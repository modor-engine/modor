use modor::{Node, RootNode};
use std::time::Duration;

/// The duration of the latest update.
///
/// The delta time is not automatically updated.
/// It can be manually set to simulate time, or be automatically updated
/// by another crate (e.g. by the graphics crate).
#[non_exhaustive]
#[derive(Default, Debug, RootNode, Node)]
pub struct Delta {
    /// Duration of the last update.
    ///
    /// Default is [`Duration::ZERO`].
    pub duration: Duration,
}
