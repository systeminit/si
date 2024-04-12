use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::func::argument::{FuncArgumentId, FuncArgumentKind};
use crate::func::associations::{FuncAssociations, FuncAssociationsError};
use crate::func::authoring::FuncAuthoringClient;
use crate::func::FuncKind;
use crate::{DalContext, Func, FuncError, FuncId, SchemaVariantError};

pub mod summary;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncViewError {
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func associations error: {0}")]
    FuncAssociations(#[from] FuncAssociationsError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
}

type FuncViewResult<T> = Result<T, FuncViewError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncArgumentView {
    pub id: FuncArgumentId,
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FuncView {
    pub id: FuncId,
    pub kind: FuncKind,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub code: Option<String>,
    pub types: String,
    pub is_builtin: bool,
    pub is_revertible: bool,
    pub associations: Option<FuncAssociations>,
}

impl FuncView {
    pub async fn assemble(ctx: &DalContext, func: &Func) -> FuncViewResult<Self> {
        let (associations, input_type) = FuncAssociations::from_func(ctx, func).await?;

        let types = [
            FuncAuthoringClient::compile_return_types(
                func.backend_response_type,
                func.backend_kind,
            ),
            &input_type,
            FuncAuthoringClient::compile_langjs_types(),
        ]
        .join("\n");

        Ok(Self {
            id: func.id.to_owned(),
            kind: func.kind,
            display_name: func.display_name.as_ref().map(Into::into),
            name: func.name.to_owned(),
            description: func.description.as_ref().map(|d| d.to_owned()),
            code: func.code_plaintext()?,
            is_builtin: func.builtin,
            is_revertible: func.is_revertible(ctx).await?,
            associations,
            types,
        })
    }
}
