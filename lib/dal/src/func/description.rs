//! This module contains the ability to add a "description" to a [`Func`](crate::Func) for a given
//! [`SchemaVariant`](crate::SchemaVariant). This is useful in the following scenarios:
//!
//! - when the same [`Func`](crate::Func) is used between two
//!   [`SchemaVariants`](crate::SchemaVariant), but has slightly different meanings based on the
//!   context and result(s)
//! - when the [`Func`](crate::Func) has static information that is specific to the
//!   [`FuncBackendResponseType`](crate::FuncBackendResponseType) and/or
//!   [`SchemaVariant`](crate::SchemaVariant) (i.e. it doesn't belong on the [`Func`](crate::Func)
//!   itself

use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    DalContext, Func, FuncBackendResponseType, FuncError, FuncId, FuncResult, SchemaVariantId,
    StandardModel, Tenancy, Timestamp, Visibility,
};

const FIND_FOR_FUNC_AND_SCHEMA_VARIANT: &str =
    include_str!("../queries/func_description_find_for_func_and_schema_variant.sql");
const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/func_description_list_for_schema_variant.sql");

/// The contents of a [`FuncDescription`], which differ based on the [`Func's`](crate::Func)
/// [`FuncBackendResponseType`](crate::FuncBackendResponseType).
#[remain::sorted]
#[derive(
    Deserialize, Serialize, Debug, Display, AsRefStr, PartialEq, Eq, EnumIter, EnumString, Clone,
)]
pub enum FuncDescriptionContents {
    /// Corresponds to
    /// [`FuncBackendResponseType::Confirmation`](crate::FuncBackendResponseType::Confirmation).
    Confirmation {
        name: String,
        success_description: Option<String>,
        failure_description: Option<String>,
        provider: Option<String>,
    },
}

impl FuncDescriptionContents {
    /// Return the [`FuncBackendResponseType`](crate::FuncBackendResponseType) corresponding to the
    /// variant of [`self`](Self).
    pub fn response_type(&self) -> FuncBackendResponseType {
        match self {
            Self::Confirmation { .. } => FuncBackendResponseType::Confirmation,
        }
    }
}

pk!(FuncDescriptionPk);
pk!(FuncDescriptionId);

/// An additional description for a [`Func`](crate::Func) that is specific to a
/// [`SchemaVariant`](crate::SchemaVariant).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncDescription {
    pk: FuncDescriptionPk,
    id: FuncDescriptionId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,

    /// Corresponds to the [`Func`](crate::Func) that this description is linked to.
    func_id: FuncId,
    /// Scopes this description for a [`Func`](crate::Func) to a specific
    /// [`SchemaVariant`](crate::SchemaVariant).
    schema_variant_id: SchemaVariantId,
    /// Serialized [`FuncDescriptionContents`] which must be
    /// [`deserialized`](FuncDescription::deserialized_contents()) to use.
    serialized_contents: Value,
    response_type: FuncBackendResponseType,
}

impl_standard_model! {
    model: FuncDescription,
    pk: FuncDescriptionPk,
    id: FuncDescriptionId,
    table_name: "func_descriptions",
    history_event_label_base: "func_description",
    history_event_message_name: "Func Description"
}

impl FuncDescription {
    /// Create a [`FuncDescription`], which is unique for a [`FuncId`](crate::FuncId) and
    /// [`SchemaVariant`](crate::SchemaVariant).
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
        contents: FuncDescriptionContents,
    ) -> FuncResult<Self> {
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;

        // Ensure response type matches contents.
        let response_type = contents.response_type();
        if func.backend_response_type != response_type {
            return Err(FuncError::ResponseTypeMismatch(
                contents,
                func.backend_response_type,
            ));
        }

        // Serialize contents due to complex and variable shape.
        let serialized_contents = serde_json::to_value(contents)?;

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM func_description_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &schema_variant_id,
                    &serialized_contents,
                    &response_type.as_ref(),
                ],
            )
            .await?;
        let object: Self = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(serialized_contents, Json<Value>, FuncResult);
    standard_model_accessor_ro!(func_id, FuncId);
    standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);
    standard_model_accessor_ro!(response_type, FuncBackendResponseType);

    /// Find all [`Self`] for the provided [`SchemaVariantId`](crate::SchemaVariantId).
    #[instrument(skip_all)]
    pub async fn list_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> FuncResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_FOR_SCHEMA_VARIANT,
                &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Find all [`Self`] for the provided [`FuncId`](crate::FuncId).
    #[instrument(skip_all)]
    pub async fn list_for_func(ctx: &DalContext, func_id: FuncId) -> FuncResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT row_to_json(func_descriptions.*) as object
                FROM func_descriptions_v1($1, $2) AS func_descriptions
                WHERE func_descriptions.func_id = $3",
                &[ctx.tenancy(), ctx.visibility(), &func_id],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Find [`Self`] with a provided [`FuncId`](crate::FuncId) and
    /// [`SchemaVariantId`](crate::SchemaVariantId).
    #[instrument(skip_all)]
    pub async fn find_for_func_and_schema_variant(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> FuncResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                FIND_FOR_FUNC_AND_SCHEMA_VARIANT,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &schema_variant_id,
                ],
            )
            .await?;
        Ok(standard_model::object_option_from_row_option(row)?)
    }

    /// Deserializes the "serialized_contents" field into a [`FuncDescriptionContents`] object.
    pub fn deserialized_contents(&self) -> FuncResult<FuncDescriptionContents> {
        let contents: FuncDescriptionContents =
            serde_json::from_value(self.serialized_contents.clone())?;
        Ok(contents)
    }

    pub async fn set_deserialized_contents(
        &mut self,
        ctx: &DalContext,
        contents: FuncDescriptionContents,
    ) -> FuncResult<()> {
        let deserialized_contents: Value = serde_json::to_value(contents)?;
        self.set_serialized_contents(ctx, deserialized_contents)
            .await
    }
}
