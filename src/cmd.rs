use std::env;
use std::ffi::{OsStr, OsString};
use std::future::Future;
use std::io::{Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::process::Output;

use async_process::Command as AsyncCommand;

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

pub enum Files {
    None,
    Dir(tempfile::TempDir),
    File(tempfile::NamedTempFile),
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
            .args($self.args.iter())
    };
}

impl Command {
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

    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|s| s.as_ref().to_os_string()));
        self
    }

    pub fn current_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.current_dir = Some(dir.as_ref().to_owned());
        self
    }

    pub fn run(self) -> impl Future<Output = IoResult<Output>> {
        if let Some(ref dir) = self.current_dir {
            run_cmd!(self).arg(self.program).current_dir(dir).output()
        } else {
            run_cmd!(self).arg(self.program).output()
        }
    }

    pub fn run_with_content(
        self,
        content: &[u8],
        extension: Option<&str>,
    ) -> (impl Future<Output = IoResult<Output>>, Files) {
        let mut tmp_file = tempfile::NamedTempFile::new().expect("BUG: could not create temp file");
        let file_buf: PathBuf = tmp_file.as_ref().into();
        tmp_file
            .write_all(content)
            .expect("BUG: failed to write file :()");

        let mut files = Files::File(tmp_file);

        let future = match self.current_dir {
            Some(dir) => run_cmd!(self)
                .current_dir(dir)
                .arg(self.program)
                .arg(file_buf)
                .args(extension.into_iter().collect::<Vec<_>>())
                .output(),
            None => {
                let tmp_dir = tempfile::TempDir::new().expect("BUG: could not create temp dir");
                let path_buf: PathBuf = tmp_dir.as_ref().to_owned();

                if let Files::File(file) = files {
                    files = Files::DirAndFile(tmp_dir, file);
                }

                run_cmd!(self)
                    .current_dir(&path_buf)
                    .arg(self.program)
                    .arg(file_buf)
                    .args(extension.into_iter().collect::<Vec<_>>())
                    .output()
            }
        };

        (future, files)
    }

    pub fn run_with_paths(
        self,
        paths: &[PathBuf],
    ) -> (impl Future<Output = IoResult<Output>>, Files) {
        let mut files = Files::None;

        let future = match self.current_dir {
            Some(ref dir) => run_cmd!(self)
                .current_dir(dir)
                .arg(self.program)
                .args(paths)
                .output(),
            None => {
                let tmp_dir = tempfile::TempDir::new().expect("BUG: could not create temp dir");
                let buf: PathBuf = tmp_dir.as_ref().to_owned();
                files = Files::Dir(tmp_dir);

                run_cmd!(self)
                    .current_dir(buf)
                    .arg(self.program)
                    .args(paths)
                    .output()
            }
        };

        (future, files)
    }
}
