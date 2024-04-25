use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::func::FuncKind;
use crate::{
    DalContext, Func, FuncError, FuncId, SchemaVariant, SchemaVariantError, SchemaVariantId,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncSummaryError {
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
}

type FuncSummaryResult<T> = Result<T, FuncSummaryError>;

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FuncSummary {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncKind,
    pub name: String,
    pub display_name: Option<String>,
    pub is_builtin: bool,
}

impl FuncSummary {
    /// Returns the [summaries](FuncSummary) for all [`Funcs`](Func) in the [`ChangeSet`](crate::ChangeSet).
    pub async fn list(ctx: &DalContext) -> FuncSummaryResult<Vec<Self>> {
        Self::list_inner(ctx, None).await
    }

    /// Returns the [summaries](FuncSummary) that are associated with the provided [variant](SchemaVariant).
    pub async fn list_for_schema_variant_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> FuncSummaryResult<Vec<Self>> {
        Self::list_inner(ctx, Some(schema_variant_id)).await
    }

    /// By default, this returns a list of [`Func`] [summaries](FuncSummary) for the entire
    /// workspace. If a [`SchemaVariantId`](SchemaVariant) is passed in, it will only return
    /// [summaries](FuncSummary) that are associated with the [variant](SchemaVariant).
    async fn list_inner(
        ctx: &DalContext,
        schema_variant_id: Option<SchemaVariantId>,
    ) -> FuncSummaryResult<Vec<Self>> {
        let funcs = match schema_variant_id {
            Some(provided_schema_variant_id) => {
                SchemaVariant::all_funcs(ctx, provided_schema_variant_id).await?
            }
            None => Func::list(ctx).await?,
        };

        let customizable_kinds = [
            FuncKind::Action,
            FuncKind::Attribute,
            FuncKind::Authentication,
            FuncKind::CodeGeneration,
            FuncKind::Qualification,
        ];

        let mut func_summaries: Vec<FuncSummary> = funcs
            .iter()
            .filter(|f| {
                if f.hidden {
                    false
                } else {
                    customizable_kinds.contains(&f.kind)
                }
            })
            .map(|func| Self {
                id: func.id,
                handler: func.handler.to_owned().map(|handler| handler.to_owned()),
                kind: func.kind,
                name: func.name.to_owned(),
                display_name: func.display_name.to_owned().map(Into::into),
                is_builtin: func.builtin,
            })
            .collect();

        func_summaries.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(func_summaries)
    }
}
