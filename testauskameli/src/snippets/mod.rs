//! default provided implementations for [`MrSnippet`]

pub mod c;
pub mod echo;
pub mod haskell;
pub mod idris;
pub mod js;
pub mod lisp;
pub mod nomeme;

/// Register every service currently available with default configuration
pub fn register_all<T>(executor: &T)
where
    T: crate::Executor,
{
    executor.register(Box::new(c::C));
    executor.register(Box::new(js::JS));
    executor.register(Box::new(haskell::Haskell));
    executor.register(Box::new(idris::Idris));
    executor.register(Box::new(lisp::Lisp));
    executor.register(Box::new(nomeme::NoMeme::new()));
    executor.register(Box::new(echo::Echo));
}
