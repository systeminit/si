//! These traits are the interface though which all interaction with the underlying graph should
//! occur. Using these traits should not require any knowledge of how the underlying graph is
//! implemented.

use async_trait::async_trait;

/// The "base" trait all of the various `Ext` traits for
/// [`WorkspaceSnapshot`][super::WorkspaceSnapshot] should be built upon.
#[async_trait]
pub trait WorkspaceSnapshotInterface {}
