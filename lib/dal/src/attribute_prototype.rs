// use serde::{Deserialize, Serialize};
// use si_data::{NatsError, NatsTxn, PgError, PgTxn};
// use std::default::Default;
// use telemetry::prelude::*;
// use thiserror::Error;

// use crate::{
//     func::{
//         backend::{
//             array::FuncBackendArrayArgs, boolean::FuncBackendBooleanArgs,
//             integer::FuncBackendIntegerArgs, map::FuncBackendMapArgs,
//             prop_object::FuncBackendPropObjectArgs, string::FuncBackendStringArgs,
//         },
//         binding::{FuncBinding, FuncBindingError, FuncBindingId},
//         binding_return_value::FuncBindingReturnValue,
//         FuncId,
//     },
//     impl_standard_model, pk,
//     standard_model::{self, TypeHint},
//     standard_model_accessor, standard_model_belongs_to, ComponentId, Func, FuncBackendKind,
//     HistoryActor, HistoryEventError, IndexMap, Prop, PropError, PropId, PropKind, SchemaId,
//     SchemaVariantId, StandardModel, StandardModelError, SystemId, Tenancy, Timestamp, Visibility,
// };
// use async_recursion::async_recursion;

// pk!(AttributePrototypePk);
// pk!(AttributePrototypeId);

// #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
// pub struct AttributePrototype {
//     pk: AttributePrototypePk,
//     id: AttributePrototypeId,
//     #[serde(flatten)]
//     tenancy: Tenancy,
//     #[serde(flatten)]
//     visibility: Visibility,
//     func_id: FuncId,
//     #[serde(flatten)]
//     pub context: AttributeResolverContext,
//     #[serde(flatten)]
//     timestamp: Timestamp,
// }

// #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
// pub struct AttributeResolver {
//     pk: AttributeResolverPk,
//     id: AttributeResolverId,
//     func_id: FuncId,
//     func_binding_id: FuncBindingId,
//     pub index_map: Option<IndexMap>,
//     pub key: Option<String>,
//     #[serde(flatten)]
//     pub context: AttributeResolverContext,
//     #[serde(flatten)]
//     tenancy: Tenancy,
//     #[serde(flatten)]
//     timestamp: Timestamp,
//     #[serde(flatten)]
//     visibility: Visibility,
// }

// impl_standard_model! {
//     model: AttributeResolver,
//     pk: AttributeResolverPk,
//     id: AttributeResolverId,
//     table_name: "attribute_resolvers",
//     history_event_label_base: "attribute_resolver",
//     history_event_message_name: "Attribute Resolver"
// }
