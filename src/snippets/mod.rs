//! default provided implementations for [`MrSnippet`]

pub mod haskell;
pub mod nomeme;

/// Register every service currently available with default configuration
pub fn register_all<T>(executor: &T)
where
    T: crate::Executor,
{
    executor.register(Box::new(haskell::Haskell));
    executor.register(Box::new(nomeme::NoMeme::new()));
}
