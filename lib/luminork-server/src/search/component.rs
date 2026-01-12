use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::Arc,
};

use serde::{
    Deserialize,
    Serialize,
    de::DeserializeOwned,
};
use si_frontend_mv_types::{
    component::{
        ComponentDiffStatus,
        ComponentInList,
        SchemaMembers,
    },
    index::change_set::ChangeSetMvIndexVersion,
    reference::IndexReference,
};
use si_id::{
    AttributeValueId,
    ChangeSetId,
    ComponentId,
    SchemaVariantId,
    WorkspacePk,
};
use telemetry::prelude::*;
use tokio::task::JoinSet;
use utoipa::ToSchema;

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
) -> Result<Vec<ComponentSearchResult>> {
    // Collect the MVs we care about, spawning them so we fetch in parallel
    let mut components_in_list = JoinSet::new();
    let mut attribute_trees = HashMap::new();
    let mut schema_members = JoinSet::new();
    for IndexReference { kind, id, checksum } in
        mv_index(frigg, workspace_id, change_set_id).await?
    {
        match kind.as_str() {
            "AttributeTree" => {
                attribute_trees.insert(
                    id.parse::<ComponentId>()?,
                    tokio::spawn(fetch_mv::<AttributeTreeForSearch>(
                        frigg.clone(),
                        workspace_id,
                        kind,
                        id,
                        checksum,
                    )),
                );
            }
            "ComponentInList" => {
                components_in_list.spawn(fetch_mv::<ComponentInList>(
                    frigg.clone(),
                    workspace_id,
                    kind,
                    id,
                    checksum,
                ));
            }
            "SchemaMembers" => {
                schema_members.spawn(fetch_mv::<SchemaMembers>(
                    frigg.clone(),
                    workspace_id,
                    kind,
                    id,
                    checksum,
                ));
            }
            _ => {}
        }
    }

    // Figure out which schema variants are fully upgraded based on schema_members
    let latest_variant_ids = Arc::new(latest_variant_ids(schema_members).await?);

    // Actually search through each component
    let mut results = vec![];
    while let Some(component_in_list) = components_in_list.join_next().await {
        let component_in_list = component_in_list??;
        let attribute_tree = match attribute_trees.remove(&component_in_list.id) {
            Some(attribute_tree) => Some(attribute_tree.await??),
            None => None,
        };
        dbg!("[search] matching component {:?}", component_in_list.id);
        // Match the component against the query
        if let Some(result) = match_component(
            component_in_list,
            attribute_tree,
            latest_variant_ids.clone(),
            query.clone(),
        )? {
            results.push(result);
        }
    }
    Ok(results)
}

/// Component data in search results.
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ComponentSearchResult {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79AA")]
    pub id: ComponentId,
    #[schema(value_type = String, example = "MyInstance")]
    pub name: String,
    pub schema: ComponentSearchResultSchema,
}

/// The schema for a component in search results.
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ComponentSearchResultSchema {
    #[schema(value_type = String, example = "AWS::EC2::Instance")]
    pub name: String,
}

/// Fetch the AttributeTree MV for a component and match it against the query.
#[instrument(level = "debug", skip_all, fields(id))]
fn match_component(
    component_in_list: ComponentInList,
    attribute_tree: Option<AttributeTreeForSearch>,
    latest_variant_ids: Arc<HashSet<SchemaVariantId>>,
    query: Arc<SearchQuery>,
) -> Result<Option<ComponentSearchResult>> {
    // TODO decide just how tolerant to be here
    if match_query(
        &component_in_list,
        attribute_tree.as_ref(),
        &latest_variant_ids,
        &query,
    ) {
        Ok(Some(ComponentSearchResult {
            id: component_in_list.id,
            name: component_in_list.name,
            schema: ComponentSearchResultSchema {
                name: component_in_list.schema_name,
            },
        }))
    } else {
        Ok(None)
    }
}

/// Match a component against a query or sub-query.
///
/// This is called once for each term in the query, and the results are combined according to
/// query rules (AND, OR, NOT).
fn match_query(
    component_in_list: &ComponentInList,
    attribute_tree: Option<&AttributeTreeForSearch>,
    latest_variant_ids: &HashSet<SchemaVariantId>,
    query: &SearchQuery,
) -> bool {
    match query {
        SearchQuery::MatchValue(term) => {
            term.match_str(&component_in_list.name)
                || term.match_str(&component_in_list.schema_name)
                || term.match_ulid(component_in_list.id)
                || term.match_str(&component_in_list.schema_category)
        }
        SearchQuery::MatchAttr { name, terms } => {
            let special_match = match name.as_str() {
                "name" => terms.iter().any(|t| t.match_str(&component_in_list.name)),
                "schema" => terms
                    .iter()
                    .any(|t| t.match_str(&component_in_list.schema_name)),
                "id" => terms.iter().any(|t| t.match_ulid(component_in_list.id)),
                "category" => terms
                    .iter()
                    .any(|t| t.match_str(&component_in_list.schema_category)),
                // A component is upgradeable if its schema variant ID is not the latest
                "isupgradeable" | "isupgradable" | "upgradeable" | "upgradable" => {
                    let is_latest =
                        latest_variant_ids.contains(&component_in_list.schema_variant_id);
                    terms.iter().any(|t| match t.as_str() {
                        "false" => is_latest,
                        _ => !is_latest,
                    })
                }
                // diff/hasdiff/diffstatus match whether the component has a diff
                "diff" | "hasdiff" | "diffstatus" => {
                    terms.iter().any(|t| match t.as_str() {
                        "true" | "" => component_in_list.diff_status != ComponentDiffStatus::None,
                        "added" => component_in_list.diff_status == ComponentDiffStatus::Added,
                        // TODO probably want to distinguish these cases!
                        "removed" | "deleted" => {
                            component_in_list.diff_status == ComponentDiffStatus::Removed
                        }
                        "modified" | "changed" => {
                            component_in_list.diff_status == ComponentDiffStatus::Modified
                        }
                        _ => component_in_list.diff_status == ComponentDiffStatus::None,
                    })
                }
                _ => false,
            };

            // Look for any attributes in the actual tree
            special_match
                || attribute_tree.is_some_and(|attribute_tree| {
                    attribute_tree
                        .attribute_values
                        .values()
                        .filter(|av| match_attr_path(&av.path, name))
                        .any(|av| match_attr_value(&av.value, terms))
                })
        }
        SearchQuery::And(queries) => queries
            .iter()
            .all(|query| match_query(component_in_list, attribute_tree, latest_variant_ids, query)),
        SearchQuery::Or(queries) => queries
            .iter()
            .any(|query| match_query(component_in_list, attribute_tree, latest_variant_ids, query)),
        SearchQuery::Not(sub_query) => !match_query(
            component_in_list,
            attribute_tree,
            latest_variant_ids,
            sub_query,
        ),
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

async fn mv_index(
    frigg: &frigg::FriggStore,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
) -> Result<Vec<IndexReference>> {
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
    Ok(all_mvs)
}

async fn fetch_mv<T: DeserializeOwned>(
    frigg: frigg::FriggStore,
    workspace_id: WorkspacePk,
    kind: String,
    id: String,
    checksum: String,
) -> Result<T> {
    frigg
        .get_workspace_object_data(workspace_id, &kind, &id, &checksum)
        .await?
        .ok_or_else(|| Error::MvNotFound(kind, id, checksum))
}

/// Figure out which schema variants are fully upgraded based on schema_members
async fn latest_variant_ids(
    mut schema_members: JoinSet<Result<SchemaMembers>>,
) -> Result<HashSet<SchemaVariantId>> {
    let mut latest_variant_ids = HashSet::new();
    while let Some(schema_members) = schema_members.join_next().await {
        let schema_members = schema_members??;
        let latest_variant_id = schema_members
            .editing_variant_id
            .unwrap_or(schema_members.default_variant_id);
        latest_variant_ids.insert(latest_variant_id);
    }
    Ok(latest_variant_ids)
}
