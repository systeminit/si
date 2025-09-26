use std::{
    collections::HashMap,
    sync::Arc,
};

use futures::future::try_join_all;
use itertools::Itertools as _;
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_mv_types::{
    index::change_set::ChangeSetMvIndexVersion,
    reference::IndexReference,
};
use si_id::{
    AttributeValueId,
    ChangeSetId,
    ComponentId,
    WorkspacePk,
};
use telemetry::prelude::*;

use crate::search::{
    Error,
    Result,
    SearchQuery,
    SearchTerm,
};

/// Search for components matching the given query in the given workspace and change set.
pub async fn search(
    frigg: &frigg::FriggStore,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    query: &Arc<SearchQuery>,
) -> Result<Vec<ComponentId>> {
    // Grab the index first
    let attribute_trees = attribute_tree_mv_index(frigg, workspace_id, change_set_id).await?;

    // Spawn parallel fetch+match tasks for each component
    let match_tasks = try_join_all(attribute_trees.map(|index_ref| {
        tokio::spawn(match_component(
            frigg.clone(),
            workspace_id,
            index_ref,
            query.clone(),
        ))
    }))
    .await?;

    // Wait for them all to complete and collect the results
    match_tasks.into_iter().flatten_ok().try_collect()
}

/// Fetch the AttributeTree MV for a component and match it against the query.
#[instrument(level = "debug", skip_all, fields(id))]
async fn match_component(
    frigg: frigg::FriggStore,
    workspace_pk: WorkspacePk,
    index_ref: IndexReference,
    query: Arc<SearchQuery>,
) -> Result<Option<ComponentId>> {
    let attribute_tree = attribute_tree_mv(frigg, workspace_pk, index_ref).await?;
    if match_attribute_tree(&attribute_tree, &query) {
        Ok(Some(attribute_tree.id))
    } else {
        Ok(None)
    }
}

/// Match a component against a query or sub-query.
///
/// This is called once for each term in the query, and the results are combined according to
/// query rules (AND, OR, NOT).
fn match_attribute_tree(attribute_tree: &AttributeTreeForSearch, query: &SearchQuery) -> bool {
    match query {
        SearchQuery::MatchValue(term) => {
            term.match_str(&attribute_tree.component_name)
                || term.match_str(&attribute_tree.schema_name)
                || term.match_ulid(attribute_tree.id)
        }
        // TODO support schema:, category:, component: and id:
        SearchQuery::MatchAttr { name, terms } => attribute_tree
            .attribute_values
            .values()
            .filter(|av| match_attr_path(&av.path, name))
            .any(|av| match_attr_value(&av.value, terms)),
        SearchQuery::And(queries) => queries
            .iter()
            .all(|query| match_attribute_tree(attribute_tree, query)),
        SearchQuery::Or(queries) => queries
            .iter()
            .any(|query| match_attribute_tree(attribute_tree, query)),
        SearchQuery::Not(sub_query) => !match_attribute_tree(attribute_tree, sub_query),
        SearchQuery::All => true,
    }
}

// Match an attribute value's path against the attribute spec (i.e. Name:value,
// SecurityGroup/Name:value or /domain/Name:value)
fn match_attr_path(path: &str, pattern: &str) -> bool {
    // If it's an absolute path, match the whole thing
    if pattern.starts_with('/') {
        return path.eq_ignore_ascii_case(pattern);
    }

    // Check for relative path:
    // Name:value should match /domain/SecurityGroup/Name, but not /domain/DeadName.
    let path = path.as_bytes();
    let pattern = pattern.as_bytes();
    if path.len() > pattern.len() {
        let slash = path.len() - pattern.len() - 1;
        path[slash] == b'/' && path[(slash + 1)..].eq_ignore_ascii_case(pattern)
    } else {
        // If the pattern is longer than the actual path (plus leading slash), it can't match
        false
    }
}

fn match_attr_value(value: &serde_json::Value, terms: &[SearchTerm]) -> bool {
    // If it was attr: with no value, match anything with that attr.
    if terms.is_empty() {
        return true;
    }

    // Otherwise, match if any of the values match.
    terms.iter().any(|term| term.match_value(value))
}

/// A pared-down version of the AttributeTree MV, containing only the fields we need for searching.
/// That way we don't pay the cost of deserializing everything else.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
struct AttributeTreeForSearch {
    id: ComponentId,
    attribute_values: HashMap<AttributeValueId, AttributeValueForSearch>,
    component_name: String,
    schema_name: String,
}

/// A pared-down version of the AttributeTree MV, containing only the fields we need for searching.
/// That way we don't pay the cost of deserializing everything else.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct AttributeValueForSearch {
    // pub id: AttributeValueId,
    // pub key: Option<String>,
    path: String,
    // pub prop_id: Option<PropId>,
    value: serde_json::Value,
    // pub external_sources: Option<Vec<ExternalSource>>, // this is the detail of where the subscriptions are from
    // pub is_controlled_by_ancestor: bool, // if ancestor of prop is set by dynamic func, ID of ancestor that sets it
    // pub is_controlled_by_dynamic_func: bool, // props driven by non-dynamic funcs have a statically set value
    // pub overridden: bool, // true if this prop has a different controlling func id than the default for this asset
    // pub validation: Option<ValidationOutput>,
    // pub secret: Option<Secret>,
    // TODO remove from here and frontend. Always false right now.
    // pub has_socket_connection: bool,
    // pub is_default_source: bool,
}

async fn attribute_tree_mv_index(
    frigg: &frigg::FriggStore,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
) -> Result<impl Iterator<Item = IndexReference>> {
    // Grab the index
    // TODO don't convert to JSON and immediately convert to struct--convert straight to struct
    let Some((index, _)) = frigg
        .get_change_set_index(workspace_id, change_set_id)
        .await?
    else {
        return Err(Error::ChangeSetIndexNotFound {
            workspace_id,
            change_set_id,
        });
    };
    let all_mvs = match serde_json::from_value(index.data)? {
        ChangeSetMvIndexVersion::V1(v1_index) => v1_index.mv_list,
        ChangeSetMvIndexVersion::V2(v2_index) => v2_index.mv_list,
    };
    Ok(all_mvs.into_iter().filter(|mv| mv.kind == "AttributeTree"))
}

// Fetch a single AttributeTree MV
async fn attribute_tree_mv(
    frigg: frigg::FriggStore,
    workspace_pk: WorkspacePk,
    IndexReference { kind, id, checksum }: IndexReference,
) -> Result<AttributeTreeForSearch> {
    frigg
        .get_workspace_object_data(workspace_pk, &kind, &id, &checksum)
        .await?
        .ok_or(Error::MvNotFound(kind, id, checksum))
}
