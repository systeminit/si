use content_store::ContentHash;
use serde::{Deserialize, Serialize};
use si_pkg::ActionFuncSpecKind;
use std::default::Default;
use strum::{AsRefStr, Display, EnumDiscriminants};

use crate::workspace_snapshot::content_address::ContentAddress;
use crate::{pk, SchemaVariantId, Timestamp};

// const FIND_FOR_CONTEXT: &str = include_str!("./queries/action_prototype/find_for_context.sql");
// const FIND_FOR_CONTEXT_AND_KIND: &str =
//     include_str!("./queries/action_prototype/find_for_context_and_kind.sql");
// const FIND_FOR_FUNC: &str = include_str!("./queries/action_prototype/find_for_func.sql");
// const FIND_FOR_CONTEXT_AND_FUNC: &str =
//     include_str!("./queries/action_prototype/find_for_context_and_func.sql");

// #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct ActionPrototypeView {
//     id: ActionPrototypeId,
//     name: String,
//     display_name: Option<String>,
// }

// impl ActionPrototypeView {
//     pub async fn new(
//         ctx: &DalContext,
//         prototype: ActionPrototype,
//     ) -> ActionPrototypeResult<ActionPrototypeView> {
//         // let mut display_name = None;
//         // let func_details = Func::get_by_id(ctx, &prototype.func_id).await?;
//         // if let Some(func) = func_details {
//         //     display_name = func.display_name().map(|dname| dname.to_string())
//         // };
//         Ok(Self {
//             id: prototype.id,
//             name: prototype.name().map_or_else(
//                 || match prototype.kind() {
//                     ActionKind::Create => "create".to_owned(),
//                     ActionKind::Delete => "delete".to_owned(),
//                     ActionKind::Other => "other".to_owned(),
//                     ActionKind::Refresh => "refresh".to_owned(),
//                 },
//                 ToOwned::to_owned,
//             ),
//             display_name: Some("delete me".to_string()),
//         })
//     }
// }

// #[remain::sorted]
// #[derive(Error, Debug)]
// pub enum ActionPrototypeError {
//     #[error("component error: {0}")]
//     Component(String),
//     #[error("component not found: {0}")]
//     ComponentNotFound(ComponentId),
//     #[error(transparent)]
//     ComponentView(#[from] ComponentViewError),
//     #[error(transparent)]
//     FuncBinding(#[from] FuncBindingError),
//     #[error(transparent)]
//     FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
//     #[error("action Func {0} not found for ActionPrototype {1}")]
//     FuncNotFound(FuncId, ActionPrototypeId),
//     #[error("history event error: {0}")]
//     HistoryEvent(#[from] HistoryEventError),
//     #[error("this asset already has an action of this kind")]
//     MultipleOfSameKind,
//     #[error("nats txn error: {0}")]
//     Nats(#[from] NatsError),
//     #[error("not found with kind {0} for context {1:?}")]
//     NotFoundByKindAndContext(ActionKind, ActionPrototypeContext),
//     #[error("pg error: {0}")]
//     Pg(#[from] PgError),
//     #[error("schema not found")]
//     SchemaNotFound,
//     #[error("schema variant not found")]
//     SchemaVariantNotFound,
//     #[error("error serializing/deserializing json: {0}")]
//     SerdeJson(#[from] serde_json::Error),
//     #[error("standard model error: {0}")]
//     StandardModelError(#[from] StandardModelError),
//     #[error("transactions error: {0}")]
//     Transactions(#[from] TransactionsError),
//     #[error(transparent)]
//     WsEvent(#[from] WsEventError),
// }

// pub type ActionPrototypeResult<T> = Result<T, ActionPrototypeError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct ActionPrototypeContext {
    pub schema_variant_id: SchemaVariantId,
}

/// Describes how an [`Action`](ActionPrototype) affects the world.
#[remain::sorted]
#[derive(AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy, Hash)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ActionKind {
    /// The [`action`](ActionPrototype) creates a new "resource".
    Create,
    /// The [`action`](ActionPrototype) deletes an existing "resource".
    Delete,
    /// The [`action`](ActionPrototype) is "internal only" or has multiple effects.
    Other,
    /// The [`action`](ActionPrototype) that refreshes an existing "resource".
    Refresh,
}

impl From<ActionFuncSpecKind> for ActionKind {
    fn from(value: ActionFuncSpecKind) -> Self {
        match value {
            ActionFuncSpecKind::Create => ActionKind::Create,
            ActionFuncSpecKind::Refresh => ActionKind::Refresh,
            ActionFuncSpecKind::Other => ActionKind::Other,
            ActionFuncSpecKind::Delete => ActionKind::Delete,
        }
    }
}

impl From<&ActionKind> for ActionFuncSpecKind {
    fn from(value: &ActionKind) -> Self {
        match value {
            ActionKind::Create => ActionFuncSpecKind::Create,
            ActionKind::Refresh => ActionFuncSpecKind::Refresh,
            ActionKind::Other => ActionFuncSpecKind::Other,
            ActionKind::Delete => ActionFuncSpecKind::Delete,
        }
    }
}

pk!(ActionPrototypeId);

// An ActionPrototype joins a `FuncId` to a `SchemaVariantId` with a `ActionKind` and `name`
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ActionPrototype {
    id: ActionPrototypeId,
    kind: ActionKind,
    name: Option<String>,
    #[serde(flatten)]
    timestamp: Timestamp,
}

#[derive(Debug, PartialEq)]
pub struct ActionPrototypeGraphNode {
    id: ActionPrototypeId,
    content_address: ContentAddress,
    content: ActionPrototypeContentV1,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
#[serde(tag = "version")]
pub enum ActionPrototypeContent {
    V1(ActionPrototypeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ActionPrototypeContentV1 {
    kind: ActionKind,
    name: Option<String>,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl ActionPrototypeGraphNode {
    pub fn assemble(
        id: impl Into<ActionPrototypeId>,
        content_hash: ContentHash,
        content: ActionPrototypeContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::ActionPrototype(content_hash),
            content,
        }
    }
}

// impl ActionPrototype {
//     pub async fn find_for_context(
//         ctx: &DalContext,
//         context: ActionPrototypeContext,
//     ) -> ActionPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 FIND_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context.schema_variant_id(),
//                 ],
//             )
//             .await?;

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     pub async fn find_for_context_and_kind(
//         ctx: &DalContext,
//         kind: ActionKind,
//         context: ActionPrototypeContext,
//     ) -> ActionPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 FIND_FOR_CONTEXT_AND_KIND,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &kind.as_ref(),
//                     &context.schema_variant_id(),
//                 ],
//             )
//             .await?;

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     pub async fn find_for_func(
//         ctx: &DalContext,
//         func_id: FuncId,
//     ) -> ActionPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(FIND_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), &func_id])
//             .await?;

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     pub async fn find_for_context_and_func(
//         ctx: &DalContext,
//         context: ActionPrototypeContext,
//         func_id: FuncId,
//     ) -> ActionPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 FIND_FOR_CONTEXT_AND_FUNC,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context.schema_variant_id(),
//                     &func_id,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     standard_model_accessor!(
//         schema_variant_id,
//         Pk(SchemaVariantId),
//         ActionPrototypeResult
//     );
//     standard_model_accessor!(name, Option<String>, ActionPrototypeResult);
//     standard_model_accessor!(func_id, Pk(FuncId), ActionPrototypeResult);
//     standard_model_accessor!(kind, Enum(ActionKind), ActionPrototypeResult);

//     pub async fn set_kind_checked(
//         &mut self,
//         ctx: &DalContext,
//         kind: ActionKind,
//     ) -> ActionPrototypeResult<()> {
//         let action_prototypes = Self::find_for_context(
//             ctx,
//             ActionPrototypeContext {
//                 schema_variant_id: self.schema_variant_id(),
//             },
//         )
//         .await?;
//         for prototype in action_prototypes {
//             if *prototype.kind() == kind && kind != ActionKind::Other && prototype.id() != self.id()
//             {
//                 return Err(ActionPrototypeError::MultipleOfSameKind);
//             }
//         }
//         self.set_kind(ctx, kind).await
//     }

//     pub fn context(&self) -> ActionPrototypeContext {
//         let mut context = ActionPrototypeContext::new();
//         context.set_schema_variant_id(self.schema_variant_id);

//         context
//     }

// pub async fn run(
//     &self,
//     ctx: &DalContext,
//     component_id: ComponentId,
// ) -> ActionPrototypeResult<Option<ActionRunResult>> {
//     let component_view = ComponentView::new(ctx, component_id).await?;

//     let before = before_funcs_for_component(ctx, &component_id).await?;

//     let (_, return_value) = FuncBinding::create_and_execute(
//         ctx,
//         serde_json::to_value(component_view)?,
//         self.func_id(),
//         before,
//     )
//     .await?;

//         let mut logs = vec![];
//         for stream_part in return_value
//             .get_output_stream(ctx)
//             .await?
//             .unwrap_or_default()
//         {
//             logs.push(stream_part);
//         }

//         logs.sort_by_key(|log| log.timestamp);

//         Ok(match return_value.value() {
//             Some(value) => {
//                 let mut run_result: ActionRunResult = serde_json::from_value(value.clone())?;
//                 run_result.logs = logs.iter().map(|l| l.message.clone()).collect();

//                 let deleted_ctx = &ctx.clone_with_delete_visibility();
//                 let mut component = Component::get_by_id(deleted_ctx, &component_id)
//                     .await?
//                     .ok_or(ActionPrototypeError::ComponentNotFound(component_id))?;

//                 if component.needs_destroy() && run_result.payload.is_none() {
//                     component
//                         .set_needs_destroy(deleted_ctx, false)
//                         .await
//                         .map_err(|e| ActionPrototypeError::Component(e.to_string()))?;
//                 }

//                 if component
//                     .set_resource(ctx, run_result.clone())
//                     .await
//                     .map_err(|e| ActionPrototypeError::Component(e.to_string()))?
//                 {
//                     WsEvent::resource_refreshed(ctx, *component.id())
//                         .await?
//                         .publish_on_commit(ctx)
//                         .await?;
//                 }

//                 Some(run_result)
//             }
//             None => None,
//         })
//     }
// }
