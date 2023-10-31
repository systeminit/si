use content_store::ContentHash;
use serde::{Deserialize, Serialize};

use strum::EnumDiscriminants;
use telemetry::prelude::*;

use crate::workspace_snapshot::content_address::ContentAddress;
use crate::{func::FuncId, pk, StandardModel, Timestamp};

// pub mod context;

// const LIST_FOR_PROP: &str = include_str!("../queries/validation_prototype/list_for_prop.sql");
// const LIST_FOR_SCHEMA_VARIANT: &str =
//     include_str!("../queries/validation_prototype/list_for_schema_variant.sql");
// const LIST_FOR_FUNC: &str = include_str!("../queries/validation_prototype/list_for_func.sql");
// const FIND_FOR_CONTEXT: &str = include_str!("../queries/validation_prototype/find_for_context.sql");

pk!(ValidationPrototypeId);

// An ValidationPrototype joins a `Func` to the context in which
// the component that is created with it can use to generate a ValidationResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationPrototype {
    id: ValidationPrototypeId,
    #[serde(flatten)]
    timestamp: Timestamp,
    func_id: FuncId,
    args: serde_json::Value,
    link: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct ValidationPrototypeGraphNode {
    id: ValidationPrototypeId,
    content_address: ContentAddress,
    content: ValidationPrototypeContentV1,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
#[serde(tag = "version")]
pub enum ValidationPrototypeContent {
    V1(ValidationPrototypeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ValidationPrototypeContentV1 {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    pub func_id: FuncId,
    pub args: serde_json::Value,
    pub link: Option<String>,
}

impl ValidationPrototypeGraphNode {
    pub fn assemble(
        id: impl Into<ValidationPrototypeId>,
        content_hash: ContentHash,
        content: ValidationPrototypeContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::ValidationPrototype(content_hash),
            content,
        }
    }
}

// impl ValidationPrototype {
//     standard_model_accessor!(func_id, Pk(FuncId), ValidationPrototypeResult);
//     standard_model_accessor!(args, Json<JsonValue>, ValidationPrototypeResult);
//     standard_model_accessor!(link, Option<String>, ValidationPrototypeResult);
//     standard_model_accessor!(prop_id, Pk(PropId), ValidationPrototypeResult);
//     standard_model_accessor!(schema_id, Pk(SchemaId), ValidationPrototypeResult);
//     standard_model_accessor!(
//         schema_variant_id,
//         Pk(SchemaVariantId),
//         ValidationPrototypeResult
//     );

//     pub fn context(&self) -> ValidationPrototypeContext {
//         ValidationPrototypeContext::new_unchecked(
//             self.prop_id,
//             self.schema_variant_id,
//             self.schema_id,
//         )
//     }

//     /// List all [`ValidationPrototypes`](Self) for a given [`Prop`](crate::Prop).
//     #[instrument(skip_all)]
//     pub async fn list_for_prop(
//         ctx: &DalContext,
//         prop_id: PropId,
//     ) -> ValidationPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(LIST_FOR_PROP, &[ctx.tenancy(), ctx.visibility(), &prop_id])
//             .await?;
//         let object = objects_from_rows(rows)?;
//         Ok(object)
//     }

//     /// List all [`ValidationPrototypes`](Self) for all [`Props`](crate::Prop) in a
//     /// [`SchemaVariant`](crate::SchemaVariant).
//     ///
//     /// _You can access the [`PropId`](crate::Prop) via the [`ValidationPrototypeContext`], if
//     /// needed._
//     #[instrument(skip_all)]
//     pub async fn list_for_schema_variant(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> ValidationPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_SCHEMA_VARIANT,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;
//         let object = objects_from_rows(rows)?;
//         Ok(object)
//     }

//     /// List all [`ValidationPrototypes`](Self) for a [`Func`](crate::Func)
//     #[instrument(skip_all)]
//     pub async fn list_for_func(
//         ctx: &DalContext,
//         func_id: FuncId,
//     ) -> ValidationPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(LIST_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), &func_id])
//             .await?;

//         Ok(objects_from_rows(rows)?)
//     }

//     pub async fn find_for_context(
//         ctx: &DalContext,
//         context: ValidationPrototypeContext,
//     ) -> ValidationPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 FIND_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context.prop_id(),
//                     &context.schema_variant_id(),
//                     &context.schema_id(),
//                 ],
//             )
//             .await?;

//         Ok(objects_from_rows(rows)?)
//     }

//     pub async fn prop(&self, ctx: &DalContext) -> ValidationPrototypeResult<Prop> {
//         Prop::get_by_id(ctx, &self.prop_id())
//             .await?
//             .ok_or(ValidationPrototypeError::PropNotFound(self.prop_id()))
//     }
// }
