use ferinth::{
    structures::project::{Project, ProjectType},
    Ferinth,
};
use furse::{structures::mod_structs::Mod, Furse};
use reqwest::StatusCode;

type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
#[error("{}", .0)]
pub enum Error {
    #[error("Modpack is already added to profile")]
    AlreadyAdded,
    #[error("The provided modpack does not exist")]
    DoesNotExist,
    #[error("The project is not a modpack")]
    NotAModpack,
    ModrinthError(ferinth::Error),
    CurseForgeError(furse::Error),
}

impl From<furse::Error> for Error {
    fn from(err: furse::Error) -> Self {
        if let furse::Error::ReqwestError(source) = &err {
            if Some(StatusCode::NOT_FOUND) == source.status() {
                Self::DoesNotExist
            } else {
                Self::CurseForgeError(err)
            }
        } else {
            Self::CurseForgeError(err)
        }
    }
}

impl From<ferinth::Error> for Error {
    fn from(err: ferinth::Error) -> Self {
        if let ferinth::Error::ReqwestError(source) = &err {
            if Some(StatusCode::NOT_FOUND) == source.status() {
                Self::DoesNotExist
            } else {
                Self::ModrinthError(err)
            }
        } else {
            Self::ModrinthError(err)
        }
    }
}

/// Check if the project of `project_id` exists and is a modpack
///
/// Returns the project struct
pub async fn curseforge(curseforge: &Furse, project_id: i32) -> Result<Mod> {
    let project = curseforge.get_mod(project_id).await?;

    // Check if the project is a modpack
    if project.class_id.is_some_and(|cid| cid == 4471)  {
        Ok(project)
    } else {
        Err(Error::NotAModpack)
    }
}

/// Check if the project of `project_id` exists and is a modpack
///
/// Returns the project struct
pub async fn modrinth(modrinth: &Ferinth, project_id: &str) -> Result<Project> {
    let project = modrinth.get_project(project_id).await?;

    if project.project_type == ProjectType::Modpack {
        Ok(project)
    } else {
        Err(Error::NotAModpack)
    }
}
