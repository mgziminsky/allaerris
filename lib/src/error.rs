#![allow(missing_docs)]

use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.kind.source()
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}
impl<E: Into<ErrorKind>> From<E> for Error {
    fn from(source: E) -> Self {
        Error { kind: source.into() }
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
#[non_exhaustive]
pub enum ErrorKind {
    // Api Errors
    #[error("The developer of project has denied third party applications from downloading it")]
    DistributionDenied,
    #[error("The project does not exist")]
    DoesNotExist,
    #[error("The project is not compatible")]
    Incompatible,
    #[error("Invalid identifier")]
    InvalidIdentifier,
    #[error("Must provide at least 1 service API wrapper")]
    NoClients,
    #[error("The project is not a {0}")]
    WrongType(&'static str),
    #[error("Project does not belong to requested API client")]
    WrongService,
    #[error("Operation not supported by this API")]
    Unsupported,

    // Config errors
    #[error("No profiles have been registered")]
    NoProfiles,
    #[error("Requested profile not recognized")]
    UnknownProfile,
    #[error("Profile path must be non-empty and absolute")]
    PathInvalid,

    // External API errors
    Modrinth(modrinth::Error),
    Forge(curseforge::Error),
    GitHub(github::Error),

    // Forwarded lib errors
    Reqwest(#[from] reqwest::Error),
    IO(#[from] std::io::Error),
    Serde(#[from] serde_json::Error),

    #[cfg(test)]
    #[error("Stub fn used for tests")]
    TestStub,
}
