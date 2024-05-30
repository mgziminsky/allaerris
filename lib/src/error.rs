#![allow(missing_docs)]

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum Error {
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
