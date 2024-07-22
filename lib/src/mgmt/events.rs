#![allow(missing_docs)]
//! The events that will be sent by a [`ProfileManager`](super::ProfileManager)

use crate::{checked_types::PathScoped, client::schema::ProjectId};

pub(super) trait EventSouce {
    fn send(&self, event: ProgressEvent);

    fn send_err(&self, err: crate::Error);
}

#[derive(Debug)]
pub enum ProgressEvent {
    Status(String),
    Download(DownloadProgress),
    Installed { file: PathScoped, is_new: bool, typ: InstallType },
    Deleted(PathScoped),
    Error(crate::Error),
}

#[derive(Debug, Clone, Copy)]
pub enum InstallType {
    Mod,
    Override,
    Other,
}

#[derive(Debug)]
pub enum DownloadProgress {
    Start { project: DownloadId, title: String, length: u64 },
    Progress(DownloadId, u64),
    Success(DownloadId),
    Fail(DownloadId, crate::Error),
}

impl From<DownloadProgress> for ProgressEvent {
    fn from(val: DownloadProgress) -> Self {
        Self::Download(val)
    }
}

/// A value meant to uniquely identify a download for event tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DownloadId(u64);
impl From<u64> for DownloadId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}
impl From<&ProjectId> for DownloadId {
    fn from(pid: &ProjectId) -> Self {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        pid.hash(&mut hasher);
        Self(hasher.finish())
    }
}
