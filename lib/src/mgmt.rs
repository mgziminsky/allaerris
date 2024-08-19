//! Profile and system management operations related to the actual downloading
//! and installing of mods and resources

mod cache;
pub mod events;
mod lockfile;
mod modpack;
mod ops;
mod version;

use std::sync::mpsc::{self, Sender};

use self::events::ProgressEvent;
// Used by client in crate-scoped update fn
pub(crate) use self::lockfile::LockedMod;
pub use self::{cache::CACHE_DIR, ops::update::UpdateInfo};

/// Handles the actual downloading, installing, updating, etc. of the contents
/// of a [`profile`](crate::config::Profile)
#[derive(Debug, Clone)]
pub struct ProfileManager {
    channel: EventChannel,
    /// Always install/overwrite files without checking if they are already
    /// installed
    pub force: bool,
    /// Don't use cache and download files directly to profile
    pub no_cache: bool,
}

impl ProfileManager {
    /// Creates a `ProfileManager` that will send [`events`](ProgressEvent) to
    /// `sender` during processing
    #[inline]
    pub fn with_channel(sender: Sender<ProgressEvent>) -> Self {
        Self {
            channel: EventChannel(sender),
            force: false,
            no_cache: false,
        }
    }

    /// Creates a new [`ProfileManager`] with no connected
    /// [`event`](ProgressEvent) channel
    #[inline]
    pub fn new() -> Self {
        Self::with_channel(mpsc::channel().0)
    }
}

impl events::EventSouce for ProfileManager {
    #[inline]
    fn send(&self, event: ProgressEvent) {
        self.channel.send(event);
    }

    fn send_err(&self, err: crate::Error) {
        self.send(ProgressEvent::Error(err));
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone)]
struct EventChannel(Sender<ProgressEvent>);
impl EventChannel {
    #[inline]
    pub fn send(&self, event: ProgressEvent) {
        let _ = self.0.send(event);
    }
}
