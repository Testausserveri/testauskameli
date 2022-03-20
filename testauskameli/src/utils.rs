//! Random util functions to make life easier
use rand::{distributions::Alphanumeric, Rng};
use std::env;
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
