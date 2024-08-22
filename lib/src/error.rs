#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum Error {
    /// The user can manually download the mod and place it in the `user` folder of the output directory to mitigate.
    /// However, they will have to manually update the mod.
    #[error("The developer of project has denied third party applications from downloading it")]
    DistributionDenied,
    #[error("The project has already been added")]
    AlreadyAdded,
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

    Modrinth(modrinth::Error),
    Forge(curseforge::Error),
    GitHub(github::Error),

    Reqwest(#[from] reqwest::Error),
    IO(#[from] std::io::Error),
    Serde(#[from] serde_json::Error),
}
