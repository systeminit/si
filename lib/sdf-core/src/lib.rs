//! Core types and utilities shared by different sdf route implementations should go here

use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

pub mod api_error;
pub mod app_state;
pub mod dal_wrapper;
pub mod nats_multiplexer;
pub mod tracking;
pub mod workspace_permissions;

/// CRDT broadcast group type, moved here because it's used in AppState
pub type BroadcastGroups = Arc<Mutex<HashMap<String, Arc<y_sync::net::BroadcastGroup>>>>;
