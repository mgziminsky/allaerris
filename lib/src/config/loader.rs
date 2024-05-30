#![allow(missing_docs)]

use std::{convert::Infallible, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    #[default]
    Unknown,

    Forge,
    Cauldron,
    LiteLoader,
    Fabric,
    Quilt,
    NeoForge,
}

impl ModLoader {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModLoader::Unknown => "",
            ModLoader::Forge => "forge",
            ModLoader::Cauldron => "cauldron",
            ModLoader::LiteLoader => "liteloader",
            ModLoader::Fabric => "fabric",
            ModLoader::Quilt => "quilt",
            ModLoader::NeoForge => "neoforge",
        }
    }

    /// Returns `true` if the mod loader is [`Unknown`](ModLoader::Unknown).
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl Display for ModLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ModLoader {
    type Err = Infallible;

    fn from_str(from: &str) -> Result<Self, Self::Err> {
        Ok(match from.trim().to_lowercase().as_str() {
            "forge" => Self::Forge,
            "cauldron" => Self::Cauldron,
            "liteloader" => Self::LiteLoader,
            "fabric" => Self::Fabric,
            "quilt" => Self::Quilt,
            "neoforge" => Self::NeoForge,
            _ => Self::Unknown,
        })
    }
}

impl<'de> Deserialize<'de> for ModLoader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = Deserialize::deserialize(deserializer)?;
        Self::from_str(val).map_err(serde::de::Error::custom)
    }
}
