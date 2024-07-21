use serde::{de, ser::SerializeStruct, Deserialize, Serialize};
use serde_value::ValueVisitor;
use thiserror::Error;

use crate::{
    client::schema::{ProjectId, VersionId},
    StdResult,
};


/// Represents a type that with a [project ID](ProjectId) and an optional
/// [version ID](VersionId). When both are present, they MUST belong to the same
/// service
pub trait VersionedProject {
    /// The [project id](ProjectId)
    fn project(&self) -> &ProjectId;

    /// The [version id](VersionId)
    fn version(&self) -> Option<&VersionId>;
}

/// [Project ID](ProjectId) with an optional [version ID](VersionId) from the
/// same service
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectWithVersion {
    pub(crate) project: ProjectId,
    pub(crate) version: Option<VersionId>,
}

/// Error when a [`ProjectWithVersion`] is created with values belonging to
/// different services
#[derive(Error, Debug, Copy, Clone)]
#[error("project and version can not belong to different services")]
pub struct ServiceMismatchError;

impl ProjectWithVersion {
    /// # Errors
    ///
    /// Will error if `version` is not [`None`] and is not for the same service
    /// as `project`
    pub fn new(project: ProjectId, version: Option<VersionId>) -> StdResult<Self, ServiceMismatchError> {
        match (&project, &version) {
            (_, None)
            | (ProjectId::Forge(_), Some(VersionId::Forge(_)))
            | (ProjectId::Modrinth(_), Some(VersionId::Modrinth(_)))
            | (ProjectId::Github(_), Some(VersionId::Github(_))) => (),
            _ => return Err(ServiceMismatchError),
        }
        Ok(Self { project, version })
    }
}
impl From<ProjectId> for ProjectWithVersion {
    fn from(pid: ProjectId) -> Self {
        Self {
            project: pid,
            version: None,
        }
    }
}

const VERS_FIELD: &str = "version";
impl Serialize for ProjectWithVersion {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let svc = match self.project {
            ProjectId::Forge(_) => "forge",
            ProjectId::Modrinth(_) => "modrinth",
            ProjectId::Github(_) => "github",
        };
        let mut ser = serializer.serialize_struct("ProjectWithVersion", 1 + self.version.as_ref().map_or(0, |_| 1))?;
        match &self.project {
            ProjectId::Forge(id) => ser.serialize_field(svc, id),
            ProjectId::Modrinth(id) => ser.serialize_field(svc, id),
            ProjectId::Github((owner, repo)) => ser.serialize_field(svc, &format_args!("{owner}/{repo}")),
        }?;
        if let Some(version) = &self.version {
            match version {
                VersionId::Forge(id) => ser.serialize_field(VERS_FIELD, id),
                VersionId::Modrinth(id) => ser.serialize_field(VERS_FIELD, id),
                VersionId::Github(id) => ser.serialize_field(VERS_FIELD, id),
            }?;
        }
        ser.end()
    }
}

impl<'de> Deserialize<'de> for ProjectWithVersion {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(variant_identifier, rename_all = "lowercase")]
        enum Field {
            Forge,
            Modrinth,
            Github,
            Version,
        }
        const FIELDS: &[&str; 4] = &["forge", "modrinth", "github", VERS_FIELD];
        if let serde_value::Value::Map(fields) = deserializer.deserialize_struct("ProjectWithVersion", FIELDS, ValueVisitor)? {
            macro_rules! into {
                ($val:expr) => {
                    $val.deserialize_into().map_err(|e| e.to_error())?
                };
            }
            macro_rules! assign {
                ($var:ident = $val:ident, $ty:path) => {
                    assign!($var = $ty(into!($val)), "project id")
                };
                ($var:ident = $val:expr, $err:literal) => {{
                    if $var.is_some() {
                        return Err(de::Error::duplicate_field($err));
                    }
                    $var = Some($val);
                }};
            }

            let mut project = None;
            let mut version = None;
            for (name, val) in fields {
                let name: Field = into!(name);
                match name {
                    Field::Forge => assign!(project = val, ProjectId::Forge),
                    Field::Modrinth => assign!(project = val, ProjectId::Modrinth),
                    Field::Github => assign!(project = val, ProjectId::Github),
                    Field::Version => assign!(version = val, "version"),
                }
            }
            let project = project.ok_or(de::Error::missing_field("project id"))?;
            let version = if let Some(val) = version {
                Some(match project {
                    ProjectId::Forge(_) => VersionId::Forge(into!(val)),
                    ProjectId::Modrinth(_) => VersionId::Modrinth(into!(val)),
                    ProjectId::Github(_) => VersionId::Github(into!(val)),
                })
            } else {
                None
            };

            Self::new(project, version).map_err(de::Error::custom)
        } else {
            Err(de::Error::custom("only struct/map objects are supported"))
        }
    }
}


#[cfg(test)]
mod tests {
    use github::models::AssetId;
    use serde_test::{assert_tokens, Token::*};

    use super::*;

    #[test]
    fn serde_version() {
        let val = ProjectWithVersion {
            project: ProjectId::Forge(44),
            version: Option::Some(VersionId::Forge(88)),
        };
        assert_tokens(&val, &[
            Struct {
                name: "ProjectWithVersion",
                len: 2,
            },
            Str("forge"),
            U64(44),
            Str(VERS_FIELD),
            U64(88),
            StructEnd,
        ]);
    }

    #[test]
    fn serde_no_version() {
        let val = ProjectWithVersion {
            project: ProjectId::Forge(44),
            version: Option::None,
        };
        assert_tokens(&val, &[
            Struct {
                name: "ProjectWithVersion",
                len: 1,
            },
            Str("forge"),
            U64(44),
            StructEnd,
        ]);
    }

    #[test]
    fn service_mismatch() {
        ProjectWithVersion::new(ProjectId::Forge(23), Option::Some(VersionId::Github(AssetId(65))))
            .expect_err("project and version from different services should error");
    }
}
