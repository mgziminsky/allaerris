use std::borrow::Borrow;

/// A minecraft [version](Self::version) and [release date](Self::release_date)
/// that is unique by `version` and sorts by `release date` descending
#[derive(Debug, Clone)]
pub struct GameVersion {
    pub version: String,
    pub release_date: String,
}

// Compare by version
impl Eq for GameVersion {}
impl PartialEq for GameVersion {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}
impl std::hash::Hash for GameVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.version.hash(state);
    }
}

// Sort by date descending
impl PartialOrd for GameVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for GameVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.release_date.cmp(&self.release_date)
    }
}

/// Allow looking up in a set by string
impl Borrow<str> for GameVersion {
    fn borrow(&self) -> &str {
        &self.version
    }
}
