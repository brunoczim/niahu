use failure_derive::Fail as Failure;
use std::{fmt, path::PathBuf};

pub use failure::{Error, Fail, Fallible};

#[derive(Debug, Failure)]
pub struct WithPath {
    pub path: PathBuf,
    pub error: failure::Error,
}

impl fmt::Display for WithPath {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: {}", self.path.display(), self.error)
    }
}

#[derive(Debug, Failure)]
#[fail(display = "Arquivo inválido ou corrompido")]
pub struct InvalidFile;
