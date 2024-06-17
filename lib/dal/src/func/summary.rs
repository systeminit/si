use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::func::FuncKind;
use crate::{
    DalContext, Func, FuncError, FuncId, Schema, SchemaError, SchemaId, SchemaVariant,
    SchemaVariantError, SchemaVariantId,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncSummaryError {
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
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
    /// Returns the [summaries](FuncSummary) for all [`Funcs`](Func) in the workspace.
    ///
    /// This includes [`Funcs`](Func) that are "free floating" and not used by any [`Schema`] at present.
    pub async fn list(ctx: &DalContext) -> FuncSummaryResult<Vec<Self>> {
        let funcs = Func::list(ctx).await?;
        Ok(Self::from_funcs(funcs.as_slice()))
    }

    /// Returns the [summaries](FuncSummary) that are associated to the [`Schema`] corresponding to
    /// the provided [`SchemaId`](Schema).
    pub async fn list_for_schema_id(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> FuncSummaryResult<Vec<Self>> {
        let funcs = Schema::all_funcs(ctx, schema_id).await?;
        Ok(Self::from_funcs(funcs.as_slice()))
    }

    /// Returns the [summaries](FuncSummary) that are associated with the provided [variant](SchemaVariant).
    pub async fn list_for_schema_variant_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> FuncSummaryResult<Vec<Self>> {
        let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id).await?;
        Ok(Self::from_funcs(funcs.as_slice()))
    }

    /// Converts a slice of [`Funcs`](Func) into a list of [`FuncSummaries`](FuncSummary).
    fn from_funcs(funcs: &[Func]) -> Vec<Self> {
        let customizable_kinds = [
            FuncKind::Action,
            FuncKind::Attribute,
            FuncKind::Authentication,
            FuncKind::CodeGeneration,
            FuncKind::Qualification,
        ];

        let mut func_summaries: Vec<Self> = funcs
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

        func_summaries
    }
}
