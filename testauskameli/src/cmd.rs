//! Safely run something in an async-friendly way
use std::env;
use std::ffi::{OsStr, OsString};
use std::future::Future;
use std::io::{Result as IoResult, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;

use async_process::Command as AsyncCommand;

/// Command runner
///
/// Uses `sudo` and `s6-softlimit` to run commands
/// as a different user and limit its resources from Mixu
pub struct Command {
    program: OsString,
    args: Vec<OsString>,
    proc_limit: usize,
    mem_limit: usize,
    time_limit: usize,
    file_limit: usize,
    run_user: String,
    current_dir: Option<PathBuf>,
}

/// Temp-file guard, to prevent early deletion
///
/// The [`Command`] uses **tempfile** to create temporary files
/// for where to run the code and where and how to store it. These
/// are not always used, and sometimes, only a directory may be necessary.
///
/// As a result, it is necessary to return the tempfiles and make sure
/// they are not dropped before the command is run.
///
/// Therefore, most run methods return a tuple of a future and `Files`, such as:
/// ```rust,no_run
/// let (output, _files) = Command::unlimited("haskell-runner")
///     .run_with_content(code.as_bytes(), Some("hs"));
/// ```
pub enum Files {
    /// No temporary files were necessary
    None,
    /// Only a temporary directory was created
    Dir(tempfile::TempDir),
    /// A temporary files was needed, but directory was set by outside forces
    File(tempfile::NamedTempFile),
    /// Both a directory and a file were needed
    DirAndFile(tempfile::TempDir, tempfile::NamedTempFile),
}

// AsyncCommand has not nice cock api
macro_rules! run_cmd {
    ($self:ident) => {
        AsyncCommand::new("sudo")
            .arg("-u")
            .arg($self.run_user.to_string())
            .arg("timeout")
            .arg("-s")
            .arg("KILL")
            .arg($self.time_limit.to_string())
            .arg("s6-softlimit")
            .arg("-a")
            .arg($self.mem_limit.to_string())
            .arg("-f")
            .arg($self.file_limit.to_string())
            .arg("-p")
            .arg($self.proc_limit.to_string())
            .env("KAMELI_FILELIMIT", $self.file_limit.to_string())
            .env("KAMELI_MEMLIMIT", $self.mem_limit.to_string())
            .env("KAMELI_PROCESSLIMIT", $self.proc_limit.to_string())
            .env("KAMELI_TIMELIMIT", $self.time_limit.to_string())
            .arg("env")
            .arg(&format!(
                "KAMELI_FILELIMIT={}",
                $self.file_limit.to_string()
            ))
            .arg(&format!("KAMELI_MEMLIMIT={}", $self.mem_limit.to_string()))
            .arg(&format!(
                "KAMELI_TIMELIMIT={}",
                $self.time_limit.to_string()
            ))
            .arg(&format!(
                "KAMELI_PROCESSLIMIT={}",
                $self.proc_limit.to_string()
            ))
            // this needs a better solution
            // TODO: unhardcore
            .arg(&format!(
                "GHC_ARGS={}",
                std::env::var("GHC_ARGS").unwrap_or_default()
            ))
            .arg(&$self.program)
            .args($self.args.iter())
    };
}

impl Command {
    /// Create a new [`Command`] for program. This is a restricted usecase,
    /// and limits will be imposed on the running program. Check crate repo root for env var information
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Self {
            program: program.as_ref().to_os_string(),
            args: vec![],
            proc_limit: env::var("KAMELI_PROCESSLIMIT")
                .map_or(Ok(1), |s| s.parse())
                .expect("BUG: impossible"),
            mem_limit: env::var("KAMELI_MEMLIMIT")
                .map_or(Ok(1000000000), |s| s.parse())
                .expect("BUG: impossible"),
            file_limit: env::var("KAMELI_FILELIMIT")
                .map_or(Ok(40000), |s| s.parse())
                .expect("BUG: impossible"),
            time_limit: env::var("KAMELI_TIMELIMIT")
                .map_or(Ok(10), |s| s.parse())
                .expect("BUG: impossible"),
            run_user: env::var("KAMELI_RUNUSER")
                .unwrap_or_else(|_| env::var("USER").expect("BUG: should exist")),
            current_dir: None,
        }
    }

    /// Create a new [`Command`] which is almost unlimited
    pub fn unlimited<S: AsRef<OsStr>>(program: S) -> Self {
        Self {
            program: program.as_ref().to_os_string(),
            args: vec![],
            proc_limit: 100000,
            mem_limit: 1000000000000,
            file_limit: 100000000000,
            // maybe we still want to limit time lmao
            time_limit: 100,
            run_user: env::var("KAMELI_RUNUSER")
                .unwrap_or_else(|_| env::var("USER").expect("BUG: should exist")),
            current_dir: None,
        }
    }

    /// Add an argument
    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    /// Extend arguments with a list (which can be empty)
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|s| s.as_ref().to_os_string()));
        self
    }

    /// Set, in which directory command should be executed. This
    /// makes it unnecessary to create a temporary directory and so it won't
    /// be created
    pub fn current_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.current_dir = Some(dir.as_ref().to_owned());
        self
    }

    /// Run a command without creating anything temporary and appending a file.
    /// Will run in CWD if current_dir is unset
    pub fn run(self) -> impl Future<Output = IoResult<Output>> {
        if let Some(ref dir) = self.current_dir {
            run_cmd!(self).arg(self.program).current_dir(dir).output()
        } else {
            run_cmd!(self).arg(self.program).output()
        }
    }

    /// Run on content provided as bytes (can be string or otherwise). Content will
    /// be placed into a temporary file and appendend to the command. An extension can optionally
    /// be given to the command as the last argument, which a script can use to create properly
    /// named files from the input
    pub fn run_with_content(
        self,
        content: &[u8],
        extension: Option<&str>,
    ) -> (impl Future<Output = IoResult<Output>>, Files) {
        let mut tmp_file = tempfile::NamedTempFile::new().expect("BUG: could not create temp file");
        let file_buf: PathBuf = tmp_file.as_ref().into();
        let mut perms = std::fs::metadata(&file_buf)
            .expect("BUG: could not get temp file metadata")
            .permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(&file_buf, perms)
            .expect("BUG: unable to set temp file permissions");
        tmp_file
            .write_all(content)
            .expect("BUG: failed to write file :()");

        let mut files = Files::File(tmp_file);

        let future = match self.current_dir {
            Some(dir) => run_cmd!(self)
                .current_dir(dir)
                .arg(file_buf)
                .args(extension.into_iter().collect::<Vec<_>>())
                .output(),
            None => {
                let tmp_dir = tempfile::TempDir::new().expect("BUG: could not create temp dir");
                let path_buf: PathBuf = tmp_dir.as_ref().to_owned();
                let mut perms = std::fs::metadata(&path_buf)
                    .expect("BUG: could not get temp dir metadata")
                    .permissions();
                perms.set_mode(0o777);
                std::fs::set_permissions(&path_buf, perms)
                    .expect("BUG: unable to set temp dir permissions");

                if let Files::File(file) = files {
                    files = Files::DirAndFile(tmp_dir, file);
                }

                run_cmd!(self)
                    .current_dir(&path_buf)
                    .arg(file_buf)
                    .args(extension.into_iter().collect::<Vec<_>>())
                    .output()
            }
        };

        (future, files)
    }

    /// Same as run, but appends a series of paths as arguments
    pub fn run_with_paths(
        self,
        paths: &[PathBuf],
    ) -> (impl Future<Output = IoResult<Output>>, Files) {
        let mut files = Files::None;

        let future = match self.current_dir {
            Some(ref dir) => run_cmd!(self).current_dir(dir).args(paths).output(),
            None => {
                let tmp_dir = tempfile::TempDir::new().expect("BUG: could not create temp dir");
                let buf: PathBuf = tmp_dir.as_ref().to_owned();
                let mut perms = std::fs::metadata(&buf)
                    .expect("BUG: could not get temp dir metadata")
                    .permissions();
                perms.set_mode(0o777);
                std::fs::set_permissions(&buf, perms)
                    .expect("BUG: unable to set temp dir permissions");
                files = Files::Dir(tmp_dir);

                run_cmd!(self).current_dir(buf).args(paths).output()
            }
        };

        (future, files)
    }
}
