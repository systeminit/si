use std::collections::{HashMap, VecDeque};

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::DfsPostOrder;
use petgraph::EdgeDirection::Outgoing;
use serde::{Deserialize, Serialize};
use si_data::{NatsConn, NatsTxn, NatsTxnError, PgError, PgTxn};
use strum_macros::{Display, IntoStaticStr};
use thiserror::Error;

use crate::resolver::ResolverError::MismatchedResolverBackend;
use crate::{MinimalStorable, Prop, SchemaMap, SiStorable};
use std::option::Option::None;

const RESOLVER_BY_NAME: &str = include_str!("./queries/resolver_by_name.sql");
const RESOLVER_BINDING_VALUES_FOR_ENTITY: &str =
    include_str!("./queries/resolver_binding_values_for_entity.sql");
const SCHEMA_ALL_PROPS: &str = include_str!("./queries/schema_all_props.sql");

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
    #[error("invalid resolver response data; expected Number and received {0}")]
    InvalidNumberData(serde_json::Value),
    #[error("invalid resolver response data; expected Boolean and received {0}")]
    InvalidBooleanData(serde_json::Value),
    #[error("invalid resolver response data; expected Object and received {0}")]
    InvalidObjectData(serde_json::Value),
    #[error("invalid resolver response data; expected Array and received {0}")]
    InvalidArrayData(serde_json::Value),
    #[error("Missing prop in attribute resolution: {0} not found")]
    MissingProp(String),
    #[error("Missing an index in the graph for a node")]
    MissingGraphIndex,
    #[error("Cycle detected with node {0}! Have a dot graph: {1}")]
    CycleDetected(String, String),
    #[error("Missing resolver binding from rgraph: {0}")]
    MissingResolverBinding(String),
    #[error("mismatched resolver binding values cannot be compared for precedence: left: {0:?}, right: {1:?}")]
    MismatchedResolverBindingValue(ResolverBindingValue, ResolverBindingValue),
    #[error("Resolver Binding Value belongs neither to a schema or a prop; invalid!: {0:?}")]
    ResolverBindingValueInvalid(ResolverBindingValue),
    #[error("Missing schema root, bug!")]
    MissingSchemaRoot,
    #[error("Cannot write value to a non object")]
    CannotWriteToObject,
    #[error("Cannot write value to a non array")]
    CannotWriteToArray,
    #[error("Schema error: {0}")]
    SchemaError(String),
    #[error("Mismatched Resolver Backend Binding; Backend {0:?} does not match Prop {1:?}")]
    MismatchedResolverBackend(ResolverBackendKindBinding, Prop),
    #[error("Map has item prop, but resolver binding value is lacking a key: {0}")]
    MissingResolverBindingValueMapKey(String),
    #[error("Missing Resolver Binding Value for required Resolver Binding: {0:?})")]
    MissingResolverBindingValueForResolverBinding(String),
    #[error("Invalid relationship; rbv must have a relationship, and this one doesn't: {0:?}")]
    InvalidRelationship(ResolverBindingValue),
    #[error("missing a prop id when one is required")]
    MissingPropId,
    #[error("schema root resolvers must return objects; returned: {0}")]
    SchemaRootResolverMustBeObject(serde_json::Value),
    #[error("cannot find matching prop named {0}; output value was: {1:?}; rb was: {2:?}")]
    InvalidOutputValueMissingProp(String, serde_json::Value, ResolverBinding),
    #[error("cannot cast a number to a u64: {0}")]
    InvalidNumberNotU64(serde_json::Value),
    #[error("missing item prop for map or array: {0}")]
    MissingItemProp(String),
    #[error(
        "mismatch between function result and schema.\n\nprop result:\n{0:?}\n\nvalue:\n{1:?}"
    )]
    MismatchedFunctionResultAndSchema(Option<Prop>, serde_json::Value),
    #[error("veritech error: {0:?}")]
    VeritechError(#[from] si_veritech::client::VeritechClientError),
    #[error("function error: {0} {1}")]
    FunctionError(String, String),
}

pub type ResolverResult<T> = Result<T, ResolverError>;

#[derive(Deserialize, Serialize, Debug, Display, IntoStaticStr, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResolverBackendKind {
    String,
    Number,
    Boolean,
    Object,
    Array,
    EmptyObject,
    EmptyArray,
    Unset,
    Json,
    Js,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ResolverBackendKindBinding {
    String(ResolverBackendKindStringBinding),
    Number(ResolverBackendKindNumberBinding),
    Boolean(ResolverBackendKindBooleanBinding),
    Object(ResolverBackendKindObjectBinding),
    Array(ResolverBackendKindArrayBinding),
    EmptyObject,
    EmptyArray,
    Unset,
    Json(ResolverBackendKindJsonBinding),
    Js(ResolverBackendKindJsBinding),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBackendKindStringBinding {
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBackendKindNumberBinding {
    pub value: u64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBackendKindBooleanBinding {
    pub value: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBackendKindObjectBinding {
    pub value: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBackendKindArrayBinding {
    pub value: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBackendKindJsonBinding {
    pub value: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBackendKindJsBinding {
    pub code: String,
}

#[derive(Deserialize, Serialize, Debug, Display, IntoStaticStr, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResolverOutputKind {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Unset,
    Json,
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

    pub async fn find_by_name(txn: &PgTxn<'_>, name: impl AsRef<str>) -> ResolverResult<Self> {
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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBinding {
    pub id: String,
    pub resolver_id: String,
    pub entity_id: Option<String>,
    pub schema_id: String,
    pub prop_id: Option<String>,
    pub parent_resolver_binding_id: Option<String>,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
    pub system_id: Option<String>,
    pub backend_binding: ResolverBackendKindBinding,
    pub map_key_name: Option<String>,
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
        parent_resolver_binding_id: Option<String>,
        entity_id: Option<String>,
        system_id: Option<String>,
        change_set_id: Option<String>,
        edit_session_id: Option<String>,
        map_key_name: Option<String>,
    ) -> ResolverResult<Self> {
        let resolver_id = resolver_id.into();

        if let Some(ref prop_id) = prop_id {
            let prop = Prop::get_by_id(&txn, &prop_id)
                .await
                .map_err(|e| ResolverError::SchemaError(e.to_string()))?;
            let valid = match (&prop, &backend_binding) {
                (_, ResolverBackendKindBinding::Unset)
                | (_, ResolverBackendKindBinding::Json(_))
                | (_, ResolverBackendKindBinding::Js(_))
                | (Prop::Map(_), ResolverBackendKindBinding::EmptyObject)
                | (Prop::Map(_), ResolverBackendKindBinding::Object(_))
                | (Prop::Array(_), ResolverBackendKindBinding::Array(_))
                | (Prop::Array(_), ResolverBackendKindBinding::EmptyArray)
                | (Prop::String(_), ResolverBackendKindBinding::String(_))
                | (Prop::Boolean(_), ResolverBackendKindBinding::Boolean(_))
                | (Prop::Object(_), ResolverBackendKindBinding::Object(_))
                | (Prop::Object(_), ResolverBackendKindBinding::EmptyObject)
                | (Prop::Number(_), ResolverBackendKindBinding::Number(_)) => true,
                _ => false,
            };
            if !valid {
                return Err(MismatchedResolverBackend(
                    backend_binding.clone(),
                    prop.clone(),
                ));
            }
        }

        let backend_binding = serde_json::to_value(&backend_binding)?;
        let row = txn
            .query_one(
                "SELECT object FROM resolver_binding_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[
                    &resolver_id,
                    &schema_id,
                    &prop_id,
                    &parent_resolver_binding_id,
                    &entity_id,
                    &backend_binding,
                    &system_id,
                    &change_set_id,
                    &edit_session_id,
                    &map_key_name,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let resolver_binding: ResolverBinding = serde_json::from_value(json)?;
        resolver_binding.resolve(&txn, &nats).await?;

        Ok(resolver_binding)
    }

    #[async_recursion::async_recursion]
    pub async fn resolve(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
    ) -> ResolverResult<Option<serde_json::Value>> {
        // Resolve arguments by looking up the ResolverArgBindings
        //
        // Dispatch to the backend
        let output_value = match &self.backend_binding {
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
            ResolverBackendKindBinding::Number(context) => {
                let result = serde_json::to_value(&context.value)?;
                if !result.is_number() {
                    return Err(ResolverError::InvalidNumberData(result));
                }
                result
            }
            ResolverBackendKindBinding::Boolean(context) => {
                let result = serde_json::to_value(&context.value)?;
                if !result.is_boolean() {
                    return Err(ResolverError::InvalidNumberData(result));
                }
                result
            }
            ResolverBackendKindBinding::Object(context) => {
                if !context.value.is_object() {
                    return Err(ResolverError::InvalidObjectData(context.value.clone()));
                }
                context.value.clone()
            }
            ResolverBackendKindBinding::Array(context) => {
                if !context.value.is_array() {
                    return Err(ResolverError::InvalidArrayData(context.value.clone()));
                }
                context.value.clone()
            }
            ResolverBackendKindBinding::EmptyObject => serde_json::json!({}),
            ResolverBackendKindBinding::EmptyArray => serde_json::json!([]),
            ResolverBackendKindBinding::Unset => return Ok(None),
            ResolverBackendKindBinding::Json(context) => context.value.clone(),
            ResolverBackendKindBinding::Js(context) => {
                let conn: NatsConn = nats.connection.clone().into();
                let result =
                    si_veritech::client::run_function(&conn, "resolver", context.code.clone())
                        .await?;
                match result {
                    si_veritech::FunctionResult::Success(success) => {
                        if success.unset {
                            return Ok(None);
                        } else {
                            success.data
                        }
                    }
                    si_veritech::FunctionResult::Failure(failure) => {
                        return Err(ResolverError::FunctionError(
                            failure.error.name,
                            failure.error.message,
                        ));
                    }
                }
            }
        };

        create_rbv_and_generate_resolver_bindings_from_output_value(
            &txn,
            &nats,
            &self,
            output_value.clone(),
        )
        .await?;

        Ok(Some(output_value))
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

// NOTE: we need to remember what the arguments to the binding were at resolution time
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolverBindingValue {
    pub id: String,
    pub resolver_binding_id: String,
    pub resolver_id: String,
    pub entity_id: Option<String>,
    pub schema_id: String,
    pub prop_id: Option<String>,
    pub parent_resolver_binding_id: Option<String>,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
    pub system_id: Option<String>,
    pub output_value: serde_json::Value,
    pub obj_value: serde_json::Value,
    pub map_key_name: Option<String>,
    pub si_storable: MinimalStorable,
}

impl ResolverBindingValue {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        output_value: serde_json::Value,
        obj_value: serde_json::Value,
        resolver_binding_id: impl Into<String>,
        resolver_id: impl Into<String>,
        schema_id: String,
        prop_id: Option<String>,
        parent_resolver_binding_id: Option<String>,
        entity_id: Option<String>,
        system_id: Option<String>,
        change_set_id: Option<String>,
        edit_session_id: Option<String>,
        map_key_name: Option<String>,
    ) -> ResolverResult<Self> {
        let resolver_id = resolver_id.into();
        let resolver_binding_id = resolver_binding_id.into();

        let row = txn
            .query_one(
                "SELECT object FROM resolver_binding_value_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                &[
                    &output_value,
                    &obj_value,
                    &resolver_binding_id,
                    &resolver_id,
                    &schema_id,
                    &prop_id,
                    &parent_resolver_binding_id,
                    &entity_id,
                    &system_id,
                    &change_set_id,
                    &edit_session_id,
                    &map_key_name,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: ResolverBindingValue = serde_json::from_value(json)?;
        Ok(object)
    }

    pub fn is_schema_root(&self) -> bool {
        self.prop_id.is_none()
    }

    pub fn takes_precedence(&self, other: &ResolverBindingValue) -> ResolverResult<bool> {
        if self.schema_id != other.schema_id {
            return Err(ResolverError::MismatchedResolverBindingValue(
                self.clone(),
                other.clone(),
            ));
        }
        if self.prop_id != other.prop_id {
            return Err(ResolverError::MismatchedResolverBindingValue(
                self.clone(),
                other.clone(),
            ));
        }
        if self.entity_id.is_some() && other.entity_id.is_none() {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn reference_key(&self) -> String {
        if let Some(parent_resolver_binding_id) = &self.parent_resolver_binding_id {
            parent_resolver_binding_id.to_string()
        } else if let Some(prop_id) = &self.prop_id {
            prop_id.to_string()
        } else {
            self.schema_id.to_string()
        }
    }
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

//pub async fn execute_resolver_bindings(
//    txn: &PgTxn<'_>,
//    nats: &NatsTxn,
//    schema_id: impl AsRef<str>,
//    entity_id: impl AsRef<str>,
//) -> ResolverResult<()> {
//    let schema_id = schema_id.as_ref();
//    let entity_id = entity_id.as_ref();
//
//    let mut rgraph = DiGraph::<String, String>::new();
//    let mut resolver_bindings: HashMap<String, ResolverBinding> = HashMap::new();
//    let mut resolver_bindings_by_prop_id: HashMap<String, Vec<String>> = HashMap::new();
//    let mut resolver_bindings_to_rgraph_node_id: HashMap<String, NodeIndex<u32>> = HashMap::new();
//    let mut schema_root_rgraph_node_ids: Vec<NodeIndex<u32>> = Vec::new();
//    let mut props: HashMap<String, Prop> = HashMap::new();
//
//    // Select all the ResolverBindings that relate to this schema, properties, or entity
//    let rows = txn
//        .query(RESOLVER_BINDINGS_FOR_ENTITY, &[&schema_id, &entity_id])
//        .await?;
//
//    for row in rows.into_iter() {
//        let resolver_binding_json: serde_json::Value = row.try_get("resolver_binding")?;
//        let resolver_binding: ResolverBinding = serde_json::from_value(resolver_binding_json)?;
//        let idx = rgraph.add_node(resolver_binding.id.clone());
//        if resolver_binding.is_schema_root() {
//            schema_root_rgraph_node_ids.push(idx.clone());
//        }
//        resolver_bindings_to_rgraph_node_id.insert(resolver_binding.id.clone(), idx);
//
//        if let Some(resolver_binding_prop_id) = &resolver_binding.prop_id {
//            let resolver_binding_ids = resolver_bindings_by_prop_id
//                .entry(resolver_binding_prop_id.clone())
//                .or_insert_with(|| Vec::new());
//            resolver_binding_ids.push(resolver_binding.id.clone());
//        }
//
//        resolver_bindings.insert(resolver_binding.id.clone(), resolver_binding);
//
//        if let Some(prop_json) = row.try_get("prop").ok() {
//            let prop: Prop = serde_json::from_value(prop_json)?;
//            props.insert(prop.id().to_string(), prop);
//        }
//    }
//
//    for (resolver_binding_id, resolver_binding) in resolver_bindings.iter() {
//        if resolver_binding.is_schema_root() {
//            continue;
//        }
//        if let Some(prop_id) = &resolver_binding.prop_id {
//            let prop = props
//                .get(prop_id)
//                .ok_or_else(|| ResolverError::MissingProp(prop_id.to_string()))?;
//            let our_index = resolver_bindings_to_rgraph_node_id
//                .get(resolver_binding_id)
//                .ok_or_else(|| ResolverError::MissingGraphIndex)?;
//            if let Some(parent_id) = prop.parent_id() {
//                let parent_resolvers: &Vec<String> = resolver_bindings_by_prop_id
//                    .get(parent_id)
//                    .ok_or_else(|| ResolverError::MissingProp(parent_id.to_string()))?;
//                for parent_resolver_id in parent_resolvers.iter() {
//                    let parent_resolver_index = resolver_bindings_to_rgraph_node_id
//                        .get(parent_resolver_id)
//                        .ok_or_else(|| ResolverError::MissingGraphIndex)?;
//                    rgraph.add_edge(*parent_resolver_index, *our_index, "depends".to_string());
//                }
//            } else {
//                for schema_root_index in schema_root_rgraph_node_ids.iter() {
//                    rgraph.add_edge(*schema_root_index, *our_index, "depends".to_string());
//                }
//            }
//        }
//    }
//
//    let rgraph_sorted = toposort(&rgraph, None).map_err(|c| {
//        let dot = format!("{:?}", Dot::with_config(&rgraph, &[Config::EdgeNoLabel]));
//        ResolverError::CycleDetected(format!("{:?}", c.node_id()), dot)
//    })?;
//    for idx in rgraph_sorted {
//        let resolver_binding_id = rgraph
//            .node_weight(idx)
//            .ok_or(ResolverError::MissingGraphIndex)?;
//        let resolver_binding = resolver_bindings
//            .get(resolver_binding_id)
//            .ok_or_else(|| ResolverError::MissingResolverBinding(resolver_binding_id.clone()))?;
//        let resolver_result = resolver_binding.resolve().await?;
//        if let Some(output_value) = resolver_result {
//            let obj_value = output_value.clone(); // This will get more complex
//            let _ = ResolverBindingValue::new(
//                &txn,
//                &nats,
//                output_value,
//                obj_value,
//                &resolver_binding.id,
//                &resolver_binding.resolver_id,
//                resolver_binding.schema_id.clone(),
//                resolver_binding.prop_id.clone(),
//                resolver_binding.parent_resolver_binding_id.clone(),
//                resolver_binding.entity_id.clone(),
//                resolver_binding.system_id.clone(),
//                resolver_binding.change_set_id.clone(),
//                resolver_binding.edit_session_id.clone(),
//                resolver_binding.map_key_name.clone(),
//            )
//            .await?;
//        }
//    }
//    //println!("{:?}", Dot::with_config(&rgraph, &[Config::EdgeNoLabel]));
//    Ok(())
//}

// Given an entity id, grab the baseline values for the entity and assemble
// the object.
//
// Select all the {}
//
// Order
// - schema,[prop], entity, system, change_set, edit_session
// - schema,[prop], entity, system, change_set,
// - schema,[prop], entity, system,
// - schema,[prop], entity, change_set, edit_session
// - schema,[prop], entity, change_set,
// - schema,[prop], entity,
// - schema,[prop]
//
pub async fn get_properties_for_entity(
    txn: &PgTxn<'_>,
    schema_id: impl AsRef<str>,
    entity_id: impl AsRef<str>,
) -> ResolverResult<serde_json::Value> {
    let schema_id = schema_id.as_ref();
    let entity_id = entity_id.as_ref();

    // Select all the binding values
    let rows = txn
        .query(
            RESOLVER_BINDING_VALUES_FOR_ENTITY,
            &[&schema_id, &entity_id],
        )
        .await?;

    // Graph is rbv_id and a string for the edge type
    let mut rbvgraph = DiGraph::<String, String>::new();

    // The RBV that is the root of the schema
    let mut schema_root: Option<ResolverBindingValue> = None;

    // Key is the rbv id
    let mut selected_binding_values: HashMap<String, ResolverBindingValue> = HashMap::new();

    // A list of all the props; key is the prop id
    let mut props: HashMap<String, Prop> = HashMap::new();

    // A map of prop_ids to an rbv; for edge creation
    let mut prop_id_to_rbv_id: HashMap<String, String> = HashMap::new();

    // A map of parent_rb_ids to a vector of binding values; for edge creation
    // We're trying to get the rbv_id for a given rb_id!
    let mut rb_id_to_rbv_id: HashMap<String, String> = HashMap::new();

    // Figure out which values to keep in the working set, and which can be knocked out.
    for row in rows.into_iter() {
        let resolver_binding_value_json: serde_json::Value =
            row.try_get("resolver_binding_values")?;
        let resolver_binding_value: ResolverBindingValue =
            serde_json::from_value(resolver_binding_value_json)?;

        rb_id_to_rbv_id.insert(
            resolver_binding_value.resolver_binding_id.clone(),
            resolver_binding_value.id.clone(),
        );

        // Populate the reverse lookup by prop_id
        if let Some(prop_id) = &resolver_binding_value.prop_id {
            prop_id_to_rbv_id
                .entry(prop_id.clone())
                .or_insert(resolver_binding_value.id.clone());
        }

        if resolver_binding_value.is_schema_root() {
            // Here is where you would compare this resolver binding with any
            // previously seen for either this schema root or property. If
            // it takes precedence, it should mutate the value.
            schema_root = Some(resolver_binding_value.clone());
            selected_binding_values
                .entry(resolver_binding_value.id.clone())
                .or_insert(resolver_binding_value);
        } else {
            if let Some(prop_id) = &resolver_binding_value.prop_id {
                // Here is where you would compare this resolver binding with any
                // previously seen for either this schema root or property. If
                // it takes precedence, it should mutate the value.
                match selected_binding_values.get(prop_id) {
                    Some(existing_binding_value) => {
                        if resolver_binding_value.takes_precedence(&existing_binding_value)? {
                            selected_binding_values
                                .insert(resolver_binding_value.id.clone(), resolver_binding_value);
                        }
                    }
                    None => {
                        selected_binding_values
                            .insert(resolver_binding_value.id.clone(), resolver_binding_value);
                    }
                }
            } else {
                return Err(ResolverError::ResolverBindingValueInvalid(
                    resolver_binding_value.clone(),
                ));
            }
        }

        if let Some(prop_json) = row.try_get("prop").ok() {
            let prop: Prop = serde_json::from_value(prop_json)?;
            props.entry(prop.id().to_string()).or_insert(prop);
        }
    }

    // Key is the rbv_id, value is the result of adding the nodes to the graph.
    let mut rbv_id_to_graph_idx: HashMap<String, NodeIndex<u32>> = HashMap::new();
    for resolver_binding_value_id in selected_binding_values.keys() {
        let graph_node_idx = rbvgraph.add_node(resolver_binding_value_id.clone());
        rbv_id_to_graph_idx.insert(resolver_binding_value_id.clone(), graph_node_idx);
    }

    // Extract the schema roots index, because we're going to use it when we populate edges.
    let schema_root_id_idx = rbv_id_to_graph_idx
        .get(
            &schema_root
                .as_ref()
                .ok_or(ResolverError::MissingSchemaRoot)?
                .id,
        )
        .ok_or(ResolverError::MissingGraphIndex)?;

    // Populate the edges.
    //
    // * Edge to the schema root, if thats your parent
    // * Edge for your parent_rb relationship, if you have one
    // * Edge for your prop relationship, as a last resort
    for resolver_binding_value in selected_binding_values.values() {
        let rbv_idx = rbv_id_to_graph_idx
            .get(&resolver_binding_value.id)
            .ok_or(ResolverError::MissingGraphIndex)?;
        let (parent_idx, weight) = if resolver_binding_value.is_schema_root() {
            continue;
        } else if let Some(parent_rb_id) = &resolver_binding_value.parent_resolver_binding_id {
            let parent_rbv_id = rb_id_to_rbv_id.get(parent_rb_id).ok_or(
                ResolverError::MissingResolverBindingValueForResolverBinding(parent_rb_id.clone()),
            )?;
            let parent_idx = rbv_id_to_graph_idx
                .get(parent_rbv_id)
                .ok_or(ResolverError::MissingGraphIndex)?;
            (parent_idx, "parent_rbv".to_string())
        } else if let Some(prop_id) = &resolver_binding_value.prop_id {
            let prop = props.get(prop_id).ok_or(ResolverError::MissingPropId)?;
            if let Some(parent_prop_id) = prop.parent_id() {
                let parent_prop_prbv_id = prop_id_to_rbv_id
                    .get(parent_prop_id)
                    .ok_or(ResolverError::MissingProp(prop_id.clone()))?;
                let prop_parent_idx = rbv_id_to_graph_idx
                    .get(parent_prop_prbv_id)
                    .ok_or(ResolverError::MissingGraphIndex)?;
                (prop_parent_idx, "prop".to_string())
            } else {
                (schema_root_id_idx, "schema_root".to_string())
            }
        } else {
            return Err(ResolverError::InvalidRelationship(
                resolver_binding_value.clone(),
            ));
        };
        rbvgraph.add_edge(*parent_idx, *rbv_idx, weight);
    }
    //println!("{:?}", Dot::with_config(&rbvgraph, &[]));

    let mut dfspo = DfsPostOrder::new(&rbvgraph, *schema_root_id_idx);
    while let Some(current_idx) = dfspo.next(&rbvgraph) {
        let current_rbv_id = rbvgraph
            .node_weight(current_idx)
            .ok_or(ResolverError::MissingGraphIndex)?;

        let mut children: Vec<String> = Vec::new();
        for child_idx in rbvgraph.neighbors_directed(current_idx, Outgoing) {
            let child_rbv_id = rbvgraph
                .node_weight(child_idx)
                .ok_or(ResolverError::MissingGraphIndex)?;
            children.push(child_rbv_id.to_string());
        }

        // ADAM AND FLETCHER SAYS: Why are we sorting this? Well, the reason is that our
        // database IDs are naturally sorted by time. That means by sorting
        // this list of our children, we are certain to get them in a stable
        // order that won't change between iterations. If you monkey with any
        // of those constraints, we're pretty sure that arrays are going to break.
        // But hey - fuck around and find out, amirite?
        children.sort();
        for child_rbv_id in children.into_iter() {
            let child_rbv = selected_binding_values
                .get(&child_rbv_id)
                .ok_or(ResolverError::MissingResolverBinding(child_rbv_id.clone()))?
                .clone();

            let current_rbv = selected_binding_values.get_mut(current_rbv_id).ok_or(
                ResolverError::MissingResolverBinding(current_rbv_id.clone()),
            )?;

            let is_schema_root = current_rbv.is_schema_root();

            let child_prop = {
                let child_prop_id = child_rbv
                    .prop_id
                    .as_ref()
                    .ok_or(ResolverError::MissingPropId)?;
                let child_prop = props
                    .get(child_prop_id)
                    .ok_or_else(|| ResolverError::MissingProp(child_prop_id.clone()))?;
                child_prop
            };

            if is_schema_root {
                let obj_value = current_rbv
                    .obj_value
                    .as_object_mut()
                    .ok_or(ResolverError::CannotWriteToObject)?;
                obj_value.insert(child_prop.name().to_string(), child_rbv.obj_value);
            } else {
                let current_prop_id = current_rbv
                    .prop_id
                    .as_deref()
                    .ok_or(ResolverError::MissingPropId)?;
                let current_prop = props
                    .get(current_prop_id)
                    .ok_or(ResolverError::MissingProp(current_prop_id.to_string()))?;
                match current_prop {
                    Prop::Map(_) => {
                        let obj_value = current_rbv
                            .obj_value
                            .as_object_mut()
                            .ok_or(ResolverError::CannotWriteToObject)?;
                        let debug_foo = format!("{:?}", &child_rbv);
                        let key_name = child_rbv.map_key_name.as_ref().ok_or_else(|| {
                            ResolverError::MissingResolverBindingValueMapKey(debug_foo)
                        })?;
                        obj_value.insert(key_name.to_string(), child_rbv.obj_value);
                    }
                    Prop::Array(_) => {
                        let obj_value = current_rbv
                            .obj_value
                            .as_array_mut()
                            .ok_or(ResolverError::CannotWriteToArray)?;
                        obj_value.push(child_rbv.obj_value);
                    }
                    _ => {
                        let obj_value = current_rbv
                            .obj_value
                            .as_object_mut()
                            .ok_or(ResolverError::CannotWriteToObject)?;
                        obj_value.insert(child_prop.name().to_string(), child_rbv.obj_value);
                    }
                }
            }
        }
    }

    let schema_root_id = &schema_root
        .as_ref()
        .ok_or(ResolverError::MissingSchemaRoot)?
        .id;
    let final_value = selected_binding_values
        .remove(schema_root_id)
        .ok_or(ResolverError::MissingSchemaRoot)?;

    Ok(final_value.obj_value)
}

struct ToDoRb {
    prop: Option<Prop>,
    output_value: serde_json::Value,
    map_key_name: Option<String>,
}

pub async fn create_resolver_binding_value_from_rb(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    rb: &ResolverBinding,
    output_value: serde_json::Value,
) -> ResolverResult<()> {
    let obj_value = output_value.clone();
    let _ = ResolverBindingValue::new(
        &txn,
        &nats,
        output_value.clone(),
        obj_value,
        &rb.id,
        &rb.resolver_id,
        rb.schema_id.clone(),
        rb.prop_id.clone(),
        rb.parent_resolver_binding_id.clone(),
        rb.entity_id.clone(),
        rb.system_id.clone(),
        rb.change_set_id.clone(),
        rb.edit_session_id.clone(),
        rb.map_key_name.clone(),
    )
    .await?;
    Ok(())
}

pub async fn create_rbv_and_generate_resolver_bindings_from_output_value(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    rb: &ResolverBinding,
    incoming_output_value: serde_json::Value,
) -> ResolverResult<()> {
    match &rb.backend_binding {
        ResolverBackendKindBinding::Json(_) | ResolverBackendKindBinding::Js(_) => {
            let mut schema_map = SchemaMap::new();
            let rows = txn.query(SCHEMA_ALL_PROPS, &[&rb.schema_id]).await?;
            for row in rows.into_iter() {
                let prop_json: serde_json::Value = row.try_get("object")?;
                let prop: Prop = serde_json::from_value(prop_json)?;
                schema_map.insert(prop.id().to_string(), prop);
            }

            let prop = match rb.prop_id.as_deref() {
                Some(prop_id) => {
                    let prop = schema_map
                        .get(prop_id)
                        .ok_or(ResolverError::MissingPropId)?;
                    Some(prop.clone())
                }
                None => None,
            };

            let mut to_do_list: VecDeque<ToDoRb> = VecDeque::new();
            to_do_list.push_back(ToDoRb {
                prop,
                output_value: incoming_output_value,
                map_key_name: None,
            });

            while let Some(ToDoRb {
                prop,
                output_value,
                map_key_name,
            }) = to_do_list.pop_front()
            {
                // If you are the schema root, and this isn't then object, then fuck off
                if prop.is_none() && !output_value.is_object() {
                    return Err(ResolverError::SchemaRootResolverMustBeObject(output_value));
                }

                match (prop, output_value) {
                    (None, serde_json::Value::Object(json_object_value)) => {
                        let top_resolver = Resolver::find_by_name(&txn, "si:setEmptyObject")
                            .await
                            .expect("cannot get resolver");
                        let top_backend_binding = ResolverBackendKindBinding::EmptyObject;
                        ResolverBinding::new(
                            &txn,
                            &nats,
                            &top_resolver.id,
                            top_backend_binding,
                            rb.schema_id.clone(),
                            None,
                            None,
                            rb.entity_id.clone(),
                            rb.system_id.clone(),
                            rb.edit_session_id.clone(),
                            rb.edit_session_id.clone(),
                            map_key_name,
                        )
                        .await?;
                        for (field_key, field_json_value) in json_object_value.into_iter() {
                            let field_prop = schema_map
                                .find_prop_by_name(None, &field_key)
                                .ok_or_else(|| {
                                    ResolverError::InvalidOutputValueMissingProp(
                                        field_key.clone(),
                                        field_json_value.clone(),
                                        rb.clone(),
                                    )
                                })?;
                            to_do_list.push_back(ToDoRb {
                                prop: Some(field_prop.clone()),
                                output_value: field_json_value,
                                map_key_name: None,
                            });
                        }
                    }
                    (Some(Prop::Object(prop)), serde_json::Value::Object(json_object_value)) => {
                        let top_resolver = Resolver::find_by_name(&txn, "si:setEmptyObject")
                            .await
                            .expect("cannot get resolver");
                        let top_backend_binding = ResolverBackendKindBinding::EmptyObject;
                        ResolverBinding::new(
                            &txn,
                            &nats,
                            &top_resolver.id,
                            top_backend_binding,
                            rb.schema_id.clone(),
                            Some(prop.id.clone()),
                            None,
                            rb.entity_id.clone(),
                            rb.system_id.clone(),
                            rb.edit_session_id.clone(),
                            rb.edit_session_id.clone(),
                            map_key_name,
                        )
                        .await?;
                        for (field_key, field_json_value) in json_object_value.into_iter() {
                            let field_prop = schema_map
                                .find_prop_by_name(Some(&prop.id), &field_key)
                                .ok_or_else(|| {
                                    ResolverError::InvalidOutputValueMissingProp(
                                        field_key.to_string(),
                                        field_json_value.clone(),
                                        rb.clone(),
                                    )
                                })?;
                            to_do_list.push_back(ToDoRb {
                                prop: Some(field_prop.clone()),
                                output_value: field_json_value,
                                map_key_name: None,
                            });
                        }
                    }
                    (Some(Prop::Map(prop)), serde_json::Value::Object(json_object_value)) => {
                        let top_resolver = Resolver::find_by_name(&txn, "si:setEmptyObject")
                            .await
                            .expect("cannot get resolver");
                        let top_backend_binding = ResolverBackendKindBinding::EmptyObject;
                        ResolverBinding::new(
                            &txn,
                            &nats,
                            &top_resolver.id,
                            top_backend_binding,
                            rb.schema_id.clone(),
                            Some(prop.id.clone()),
                            None,
                            rb.entity_id.clone(),
                            rb.system_id.clone(),
                            rb.edit_session_id.clone(),
                            rb.edit_session_id.clone(),
                            map_key_name,
                        )
                        .await?;

                        let item_prop =
                            schema_map
                                .find_item_prop_for_parent(&prop.id)
                                .ok_or_else(|| {
                                    ResolverError::MissingItemProp(format!("{:?}", &prop))
                                })?;
                        for (field_key, field_json_value) in json_object_value.into_iter() {
                            to_do_list.push_back(ToDoRb {
                                prop: Some(item_prop.clone()),
                                output_value: field_json_value,
                                map_key_name: Some(field_key),
                            });
                        }
                    }
                    (Some(Prop::Array(prop)), serde_json::Value::Array(json_object_value)) => {
                        let top_resolver = Resolver::find_by_name(&txn, "si:setEmptyArray")
                            .await
                            .expect("cannot get resolver");
                        let top_backend_binding = ResolverBackendKindBinding::EmptyArray;
                        ResolverBinding::new(
                            &txn,
                            &nats,
                            &top_resolver.id,
                            top_backend_binding,
                            rb.schema_id.clone(),
                            Some(prop.id.clone()),
                            None,
                            rb.entity_id.clone(),
                            rb.system_id.clone(),
                            rb.edit_session_id.clone(),
                            rb.edit_session_id.clone(),
                            map_key_name,
                        )
                        .await?;

                        let item_prop =
                            schema_map
                                .find_item_prop_for_parent(&prop.id)
                                .ok_or_else(|| {
                                    ResolverError::MissingItemProp(format!("{:?}", &prop))
                                })?;
                        for field_json_value in json_object_value.into_iter() {
                            to_do_list.push_back(ToDoRb {
                                prop: Some(item_prop.clone()),
                                output_value: field_json_value,
                                map_key_name: None,
                            });
                        }
                    }

                    (Some(Prop::String(prop)), serde_json::Value::String(field_value)) => {
                        let resolver = Resolver::find_by_name(&txn, "si:setString")
                            .await
                            .expect("cannot get resolver");
                        let backend_binding =
                            ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
                                value: field_value,
                            });
                        ResolverBinding::new(
                            &txn,
                            &nats,
                            &resolver.id,
                            backend_binding,
                            rb.schema_id.clone(),
                            Some(prop.id.clone()),
                            None,
                            rb.entity_id.clone(),
                            rb.system_id.clone(),
                            rb.edit_session_id.clone(),
                            rb.edit_session_id.clone(),
                            map_key_name,
                        )
                        .await?;
                    }
                    (Some(Prop::Number(prop)), serde_json::Value::Number(field_json_value)) => {
                        let resolver = Resolver::find_by_name(&txn, "si:setNumber")
                            .await
                            .expect("cannot get resolver");
                        let field_value =
                            field_json_value
                                .as_u64()
                                .ok_or(ResolverError::InvalidNumberNotU64(
                                    serde_json::Value::Number(field_json_value.clone()),
                                ))?;
                        let backend_binding =
                            ResolverBackendKindBinding::Number(ResolverBackendKindNumberBinding {
                                value: field_value.clone(),
                            });
                        ResolverBinding::new(
                            &txn,
                            &nats,
                            &resolver.id,
                            backend_binding,
                            rb.schema_id.clone(),
                            Some(prop.id.clone()),
                            None,
                            rb.entity_id.clone(),
                            rb.system_id.clone(),
                            rb.edit_session_id.clone(),
                            rb.edit_session_id.clone(),
                            map_key_name,
                        )
                        .await?;
                    }
                    (Some(Prop::Boolean(prop)), serde_json::Value::Bool(field_value)) => {
                        let resolver = Resolver::find_by_name(&txn, "si:setBoolean")
                            .await
                            .expect("cannot get resolver");
                        let backend_binding = ResolverBackendKindBinding::Boolean(
                            ResolverBackendKindBooleanBinding {
                                value: field_value.clone(),
                            },
                        );
                        ResolverBinding::new(
                            &txn,
                            &nats,
                            &resolver.id,
                            backend_binding,
                            rb.schema_id.clone(),
                            Some(prop.id.clone()),
                            None,
                            rb.entity_id.clone(),
                            rb.system_id.clone(),
                            rb.edit_session_id.clone(),
                            rb.edit_session_id.clone(),
                            map_key_name,
                        )
                        .await?;
                    }
                    (prop, value) => {
                        return Err(ResolverError::MismatchedFunctionResultAndSchema(
                            prop, value,
                        ));
                    }
                }
            }
        }
        _ => {
            create_resolver_binding_value_from_rb(&txn, &nats, rb, incoming_output_value).await?;
        }
    }
    Ok(())
}

//impl DefaultStringResolver {
//    async fn resolve(obj: serde_json::Value, args: serde_json::Value, context: serde_json::Value) {}
//}
