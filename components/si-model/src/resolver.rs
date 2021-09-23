use std::collections::HashMap;

use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgError, PgTxn};
use strum_macros::{Display, IntoStaticStr};
use thiserror::Error;

use crate::{Entity, MinimalStorable, Prop, SiStorable};

const RESOLVER_BY_NAME: &str = include_str!("./queries/resolver_by_name.sql");
const RESOLVER_BINDINGS_FOR_ENTITY: &str =
    include_str!("./queries/resolver_bindings_for_entity.sql");

#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("invalid resolver response data; expected String and received {0}")]
    InvalidStringData(serde_json::Value),
    #[error("Missing prop in attribute resolution: {0} not found")]
    MissingProp(String),
    #[error("Missing an index in the graph for a node")]
    MissingGraphIndex,
    #[error("Cycle detected with node {0}! Have a dot graph: {1}")]
    CycleDetected(String, String),
    #[error("Missing resolver binding from rgraph: {0}")]
    MissingResolverBinding(String),
}

pub type ResolverResult<T> = Result<T, ResolverError>;

#[derive(Deserialize, Serialize, Debug, Display, IntoStaticStr, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResolverBackendKind {
    String,
    EmptyObject,
    EmptyArray,
    Unset,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ResolverBackendKindBinding {
    String(ResolverBackendKindStringBinding),
    EmptyObject,
    EmptyArray,
    Unset,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBackendKindStringBinding {
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Display, IntoStaticStr, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResolverOutputKind {
    String,
    Object,
    Array,
    Unset,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Resolver {
    pub id: String,
    pub backend: ResolverBackendKind,
    pub name: String,
    pub description: String,
    pub output_kind: ResolverOutputKind,
    pub si_storable: MinimalStorable,
}

impl Resolver {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        description: impl Into<String>,
        backend: ResolverBackendKind,
        output_kind: ResolverOutputKind,
    ) -> ResolverResult<Self> {
        let name = name.into();
        let description = description.into();
        let backend: &str = backend.into();
        let output_kind: &str = output_kind.into();
        let row = txn
            .query_one(
                "SELECT object FROM resolver_create_v1($1, $2, $3, $4)",
                &[&name, &description, &backend, &output_kind],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Resolver = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_by_name(txn: &PgTxn<'_>, name: impl AsRef<str>) -> ResolverResult<Self> {
        let name = name.as_ref();
        let row = txn.query_one(RESOLVER_BY_NAME, &[&name]).await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ResolverArgKind {
    String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ResolverArgKindBinding {
    String(ResolverArgKindBindingString),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolverArgKindBindingString {
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolverArg {
    pub id: String,
    pub name: String,
    pub kind: ResolverArgKind,
    pub description: String,
    pub si_storable: MinimalStorable,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBinding {
    pub id: String,
    pub resolver_id: String,
    pub entity_id: Option<String>,
    pub schema_id: String,
    pub prop_id: Option<String>,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
    pub system_id: Option<String>,
    pub backend_binding: ResolverBackendKindBinding,
    pub si_storable: MinimalStorable,
}

impl ResolverBinding {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        resolver_id: impl Into<String>,
        backend_binding: ResolverBackendKindBinding,
        schema_id: String,
        prop_id: Option<String>,
        entity_id: Option<String>,
        system_id: Option<String>,
        change_set_id: Option<String>,
        edit_session_id: Option<String>,
    ) -> ResolverResult<Self> {
        let resolver_id = resolver_id.into();

        let backend_binding = serde_json::to_value(&backend_binding)?;
        let row = txn
            .query_one(
                "SELECT object FROM resolver_binding_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &resolver_id,
                    &schema_id,
                    &prop_id,
                    &entity_id,
                    &backend_binding,
                    &system_id,
                    &change_set_id,
                    &edit_session_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: ResolverBinding = serde_json::from_value(json)?;
        Ok(object)
    }

    // TODO: Tomorrow morning, get started on how we create default resolver functions
    // as needed. In particular, we need to create the default resolver for the entire
    // schema first, which should just return a raw object. It would probably be best
    // if that resolver actually existed, becasue then we could always use the same
    // behavior for returning what we want. (ie: the user could override it)
    pub async fn resolve(&self) -> ResolverResult<Option<serde_json::Value>> {
        // Resolve arguments by looking up the ResolverArgBindings
        //
        // Dispatch to the backend
        let result = match &self.backend_binding {
            ResolverBackendKindBinding::String(context) => {
                let result = serde_json::to_value(&context.value)?;
                // You can be damn sure this is a string, really - because
                // the inner type there is a string. But hey - better safe
                // than sorry!
                if !result.is_string() {
                    return Err(ResolverError::InvalidStringData(result));
                }
                result
            }
            ResolverBackendKindBinding::EmptyObject => serde_json::json!({}),
            ResolverBackendKindBinding::EmptyArray => serde_json::json!([]),
            ResolverBackendKindBinding::Unset => return Ok(None),
        };

        Ok(Some(result))
    }

    pub fn is_schema_root(&self) -> bool {
        self.prop_id.is_none()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolverArgBinding {
    pub id: String,
    pub resolve_id: String,
    pub resolver_binding_id: String,
    pub resolver_arg_id: String,
    pub entity_id: String,
    pub system_id: String,
    pub prop_id: String,
    pub binding: ResolverArgKindBinding,
    pub si_storable: SiStorable,
}

// This is here because eventually you will be resolving attributes other
// than just properties!
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedAttributes {
    pub properties: serde_json::Value,
}

// Select all the resolver bindings for the schema + entity + context
//   - Order by the schema root first, then based on prop id path
// Run the resolver binding for the schema id
//   - it generates an empty object by default
//   - or it could return a full object
//
// { # schema "fancypants"
//   foo: "bar"
// }
//
// ResolverBinding schema "fancypants" -> {}
// ResolverBinding schema "fancypants" prop "foo" ({}) -> { foo: "bar" }
//
// { # schema "frobnob"
//   foo: {
//      bar: "baz"
//   }
// }
//
// let mut acc = {};
// ResolverBinding schema "frobnob" -> {} | acc = {}
// ResolverBinding schema "frobnob" prop "foo" ({}) -> { foo: {} }  | acc = { foo: {} }
// ResolverBinding schema "frobnob" prop "bar" path ["foo"] ({}) -> { bar: "baz" } | acc = {
// foo: { bar: "baz" }
// return acc
//
// { # schema "frobnob"
//   foo: { }
// }
//
// let mut acc = {};
// ResolverBinding schema "frobnob" -> {} | acc = {}
// ResolverBinding schema "frobnob" prop "foo" ({}) -> { foo: {} }  | acc = { foo: {} }
//   ResolverBinding schema "frobnob" prop "bar" path ["foo"] ({}) -> null | acc = {}
//   ResolverBinding schema "frobnob" prop "bar" path ["foo"] ({}) -> null | acc = {}
//   null
//
// foo: {}
// return acc
//
// A resolver binding result stores the return value of the resolve call, sorted by time.
//  - It also stores a reference to the json values of the inputs.
//  - If the currently resolved inputs match, then we can return the last resolved value.
//
// Arrays in the schema have a key. By default the key is the index of the array, but it
// can also be any scalar property of the array. So when we set the actual path/position
// for an array, we track it by key. So any dependent things use the key to decide
// what should be updated.
//
// So for a containers array with a key field of image, the full path to a property is
// 'spec containers [bar] image', and we store a map of the key to the current index
// on the array.
//
//  k8sDeployment
//      spec
//          containers < RB -> [{image: bar, image: baz}]
//              - image: bar
//              - image: baz
//

pub async fn resolve_attributes(
    txn: &PgTxn<'_>,
    schema_id: impl AsRef<str>,
    entity_id: impl AsRef<str>,
) -> ResolverResult<ResolvedAttributes> {
    let schema_id = schema_id.as_ref();
    let entity_id = entity_id.as_ref();

    let mut rgraph = DiGraph::<String, String>::new();
    let mut resolver_bindings: HashMap<String, ResolverBinding> = HashMap::new();
    let mut resolver_bindings_by_prop_id: HashMap<String, Vec<String>> = HashMap::new();
    let mut resolver_bindings_to_rgraph_node_id: HashMap<String, NodeIndex<u32>> = HashMap::new();
    let mut schema_root_rgraph_node_ids: Vec<NodeIndex<u32>> = Vec::new();
    let mut props: HashMap<String, Prop> = HashMap::new();

    // Select all the ResolverBindings that relate to this schema, properties, or entity
    let rows = txn
        .query(RESOLVER_BINDINGS_FOR_ENTITY, &[&schema_id, &entity_id])
        .await?;

    for row in rows.into_iter() {
        let resolver_binding_json: serde_json::Value = row.try_get("resolver_binding")?;
        let resolver_binding: ResolverBinding = serde_json::from_value(resolver_binding_json)?;
        println!("resolver_binding: {:?}", resolver_binding);
        let idx = rgraph.add_node(resolver_binding.id.clone());
        if resolver_binding.is_schema_root() {
            schema_root_rgraph_node_ids.push(idx.clone());
        }
        resolver_bindings_to_rgraph_node_id.insert(resolver_binding.id.clone(), idx);

        if let Some(resolver_binding_prop_id) = &resolver_binding.prop_id {
            let resolver_binding_ids = resolver_bindings_by_prop_id
                .entry(resolver_binding_prop_id.clone())
                .or_insert_with(|| Vec::new());
            resolver_binding_ids.push(resolver_binding.id.clone());
        }

        resolver_bindings.insert(resolver_binding.id.clone(), resolver_binding);

        if let Some(prop_json) = row.try_get("prop").ok() {
            let prop: Prop = serde_json::from_value(prop_json)?;
            props.insert(prop.id().to_string(), prop);
        }
    }

    for (resolver_binding_id, resolver_binding) in resolver_bindings.iter() {
        if resolver_binding.is_schema_root() {
            continue;
        }
        if let Some(prop_id) = &resolver_binding.prop_id {
            let prop = props
                .get(prop_id)
                .ok_or_else(|| ResolverError::MissingProp(prop_id.to_string()))?;
            let our_index = resolver_bindings_to_rgraph_node_id
                .get(resolver_binding_id)
                .ok_or_else(|| ResolverError::MissingGraphIndex)?;
            if let Some(parent_id) = prop.parent_id() {
                let parent_resolvers: &Vec<String> = resolver_bindings_by_prop_id
                    .get(parent_id)
                    .ok_or_else(|| ResolverError::MissingProp(parent_id.to_string()))?;
                for parent_resolver_id in parent_resolvers.iter() {
                    let parent_resolver_index = resolver_bindings_to_rgraph_node_id
                        .get(parent_resolver_id)
                        .ok_or_else(|| ResolverError::MissingGraphIndex)?;
                    rgraph.add_edge(*parent_resolver_index, *our_index, "depends".to_string());
                }
            } else {
                for schema_root_index in schema_root_rgraph_node_ids.iter() {
                    rgraph.add_edge(*schema_root_index, *our_index, "depends".to_string());
                }
            }
        }
    }

    let rgraph_sorted = toposort(&rgraph, None).map_err(|c| {
        let dot = format!("{:?}", Dot::with_config(&rgraph, &[Config::EdgeNoLabel]));
        ResolverError::CycleDetected(format!("{:?}", c.node_id()), dot)
    })?;
    println!("{:?}", rgraph_sorted);
    for idx in rgraph_sorted {
        let resolver_binding_id = rgraph
            .node_weight(idx)
            .ok_or(ResolverError::MissingGraphIndex)?;
        let resolver_binding = resolver_bindings
            .get(resolver_binding_id)
            .ok_or_else(|| ResolverError::MissingResolverBinding(resolver_binding_id.clone()))?;
        let result = resolver_binding.resolve().await?;
        println!("Resolved {:?} to {:?}", resolver_binding, result);
    }

    println!("{:?}", Dot::with_config(&rgraph, &[Config::EdgeNoLabel]));

    // Populate them in a DAG
    // Topological sort the order
    // Call the resolve function on each binding
    todo!()
}

//impl DefaultStringResolver {
//    async fn resolve(obj: serde_json::Value, args: serde_json::Value, context: serde_json::Value) {}
//}
