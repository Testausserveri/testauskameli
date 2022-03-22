//! Random util functions to make life easier
use anyhow::{anyhow, Result};
use itertools::Itertools;
use rand::{distributions::Alphanumeric, Rng};
use which::which;

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;

/// Generate a random temporary file path with extension
/// This file will be stored the system tempdir, and because
/// Windows exists, consider deleting it manually when you are
/// done with it
pub fn rand_path_with_extension(extension: &str) -> PathBuf {
    let extension: Vec<char> = if let Some('.') = extension.chars().nth(0) {
        extension.chars().collect()
    } else {
        ".".chars().chain(extension.chars()).collect()
    };

    let dir = env::temp_dir();
    let mut rng = rand::thread_rng();
    dir.join(
        (0..16)
            .map(|_| rng.sample(Alphanumeric) as char)
            .chain(extension)
            .collect::<String>(),
    )
}

/// Validate program presence
pub fn needed_programs<T: AsRef<OsStr>>(binaries: &[T]) -> Result<()> {
    let errors = binaries
        .into_iter()
        .map(|x| (x, which(x)))
        .filter_map(|(x, result)| result.err().map(|_| x))
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(
            "missing binaries: {}",
            errors
                .into_iter()
                .flat_map(|x| x.as_ref().to_str())
                .join(",")
        ))
    }
}
