#![allow(missing_docs)]

pub use std::error::Error as StdError;
use std::fmt::Display;

use itertools::Itertools;

use crate::{client::schema::ProjectId, config::ModLoader};

pub type Result<T> = std::result::Result<T, Error>;
pub type StdResult<T, E> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub struct Error {
    #[source]
    kind: ErrorKind,
}
impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}
impl<E: Into<ErrorKind>> From<E> for Error {
    fn from(source: E) -> Self {
        Self { kind: source.into() }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[non_exhaustive]
pub enum ErrorKind {
    // Api Errors
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
    #[error("Project `{0}` does not belong to requested API client")]
    WrongService(String),
    #[error("Operation not supported by this API")]
    Unsupported,
    #[error("All clients in the Multi client failed `{}` operation:\n\t{}", .0, .1.iter().join("\n\t"))]
    Multi(&'static str, Vec<Error>),

    // Config
    #[error("No profiles have been registered")]
    NoProfiles,
    #[error("Requested profile not recognized")]
    UnknownProfile,
    #[error("Profile path must be non-empty and absolute")]
    PathInvalid,

    // Management
    #[error("The developer of `{0}` has denied third party applications from downloading it")]
    DistributionDenied(String),
    #[error("No compatible version found for project `{0}`")]
    MissingVersion(ProjectId),
    #[error("Failed to download file: {0}")]
    DownloadFailed(url::Url),
    #[error("Server install not supported for `{}` loader", .0.as_str())]
    ServerUnsupported(ModLoader),

    // External API - From is manually implemented
    Modrinth(modrinth::Error),
    Forge(curseforge::Error),
    GitHub(github::Error),

    // Forwarded lib errors
    Reqwest(#[from] reqwest::Error),
    IO(#[from] std::io::Error),

    Unexpected(#[from] anyhow::Error),

    #[cfg(test)]
    #[error("Stub fn used for tests")]
    TestStub,
}
