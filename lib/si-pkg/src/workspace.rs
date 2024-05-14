//! This module defines the export format for the workspace
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceExport {
    V0(WorkspaceExportContentV0),
}

impl WorkspaceExport {
    pub fn new(content: WorkspaceExportContentV0) -> Self {
        WorkspaceExport::V0(content)
    }

    // This function should always return the latest version, updating the contents if necessary
    pub fn into_latest(self) -> WorkspaceExportContentV0 {
        let WorkspaceExport::V0(export) = self;
        export
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceExportContentV0 {
    // We store changesets keyed by the cs id they depend on, so we can import in the right order
    pub change_sets: HashMap<Ulid, Vec<WorkspaceExportChangeSetV0>>,
    pub content_store_values: Vec<u8>,
    pub metadata: WorkspaceExportMetadataV0,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceExportChangeSetV0 {
    pub id: Ulid,
    pub name: String,
    pub base_change_set_id: Option<Ulid>,
    pub workspace_snapshot_serialized_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceExportMetadataV0 {
    pub name: String,
    pub version: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub default_change_set: Ulid,
    pub default_change_set_base: Ulid,
    pub workspace_pk: Ulid,
    pub workspace_name: String,
}
