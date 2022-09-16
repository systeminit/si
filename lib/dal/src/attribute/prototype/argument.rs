//! An [`AttributePrototypeArgument`] represents an argument name and how to dynamically derive
//! the corresponding value. [`AttributePrototype`](crate::AttributePrototype) can have multiple
//! arguments.

use serde::{Deserialize, Serialize};
use si_data::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::context::UNSET_ID_VALUE;
use crate::provider::internal::InternalProviderId;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, AttributePrototypeId,
    ComponentId, DalContext, ExternalProviderId, HistoryEventError, StandardModel,
    StandardModelError, Timestamp, Visibility, WriteTenancy,
};

const LIST_FOR_ATTRIBUTE_PROTOTYPE: &str =
    include_str!("../../queries/attribute_prototype_argument_list_for_attribute_prototype.sql");
const LIST_BY_NAME_FOR_ATTRIBUTE_PROTOTYPE_AND_HEAD_COMPONENT_ID: &str = include_str!(
    "../../queries/attribute_prototype_argument_list_by_name_for_attribute_prototype_and_head_component_id.sql"
);

#[derive(Error, Debug)]
pub enum AttributePrototypeArgumentError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),

    #[error("cannot update set field to become unset: {0}")]
    CannotFlipSetFieldToUnset(&'static str),
    #[error("cannot update unset field to become set: {0}")]
    CannotFlipUnsetFieldToSet(&'static str),
    #[error("required value fields must be set, found at least one unset required value field")]
    RequiredValueFieldsUnset,
}

pub type AttributePrototypeArgumentResult<T> = Result<T, AttributePrototypeArgumentError>;

pk!(AttributePrototypeArgumentPk);
pk!(AttributePrototypeArgumentId);

/// Contains a "key" and fields to derive a "value" that dynamically used as an argument for a
/// [`AttributePrototypes`](crate::AttributePrototype) function execution.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributePrototypeArgument {
    pk: AttributePrototypeArgumentPk,
    id: AttributePrototypeArgumentId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    visibility: Visibility,
    #[serde(flatten)]
    timestamp: Timestamp,

    /// Indicates the [`AttributePrototype`](crate::AttributePrototype) that [`Self`] is used as
    /// an argument for.
    attribute_prototype_id: AttributePrototypeId,
    /// The "key" for a given argument.
    name: String,
    /// Where to find the value for a given argument for _intra_ [`Component`](crate::Component)
    /// connections.
    internal_provider_id: InternalProviderId,
    /// Where to find the value for a given argument for _inter_ [`Component`](crate::Component)
    /// connections.
    external_provider_id: ExternalProviderId,
    /// For _inter_ [`Component`](crate::Component) connections, this field provides additional
    /// information to determine the _source_ of the value.
    tail_component_id: ComponentId,
    /// For _inter_ [`Component`](crate::Component) connections, this field provides additional
    /// information to determine the _destination_ of the value.
    head_component_id: ComponentId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttributePrototypeArgumentGroup {
    pub name: String,
    pub arguments: Vec<AttributePrototypeArgument>,
}

impl_standard_model! {
    model: AttributePrototypeArgument,
    pk: AttributePrototypeArgumentPk,
    id: AttributePrototypeArgumentId,
    table_name: "attribute_prototype_arguments",
    history_event_label_base: "attribute_prototype_argument",
    history_event_message_name: "Attribute Prototype Argument"
}

impl AttributePrototypeArgument {
    #[instrument(skip_all)]
    /// Create a new [`AttributePrototypeArgument`] for _intra_ [`Component`](crate::Component) use.
    pub async fn new_for_intra_component(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        name: impl AsRef<str>,
        internal_provider_id: InternalProviderId,
    ) -> AttributePrototypeArgumentResult<Self> {
        // Ensure the value fields are what we expect.
        let external_provider_id: ExternalProviderId = UNSET_ID_VALUE.into();
        let tail_component_id: ComponentId = UNSET_ID_VALUE.into();
        let head_component_id: ComponentId = UNSET_ID_VALUE.into();
        if internal_provider_id == UNSET_ID_VALUE.into() {
            return Err(AttributePrototypeArgumentError::RequiredValueFieldsUnset);
        }

        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                    &name,
                    &internal_provider_id,
                    &external_provider_id,
                    &tail_component_id,
                    &head_component_id,
                ],
            )
            .await?;
        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    /// Create a new [`AttributePrototypeArgument`] for _inter_ [`Component`](crate::Component) use.
    #[instrument(skip_all)]
    pub async fn new_for_inter_component(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        name: impl AsRef<str>,
        head_component_id: ComponentId,
        tail_component_id: ComponentId,
        external_provider_id: ExternalProviderId,
    ) -> AttributePrototypeArgumentResult<Self> {
        // Ensure the value fields are what we expect.
        let internal_provider_id: ExternalProviderId = UNSET_ID_VALUE.into();
        if external_provider_id == UNSET_ID_VALUE.into()
            || tail_component_id == UNSET_ID_VALUE.into()
            || head_component_id == UNSET_ID_VALUE.into()
        {
            return Err(AttributePrototypeArgumentError::RequiredValueFieldsUnset);
        }

        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                    &name,
                    &internal_provider_id,
                    &external_provider_id,
                    &tail_component_id,
                    &head_component_id,
                ],
            )
            .await?;
        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(
        attribute_prototype_id,
        Pk(AttributePrototypeId),
        AttributePrototypeArgumentResult
    );
    standard_model_accessor!(name, String, AttributePrototypeArgumentResult);

    // FIXME(nick): add standard model accessor wrapper that disallows updating set field
    // to become unset and vice versa.
    standard_model_accessor!(
        internal_provider_id,
        Pk(InternalProviderId),
        AttributePrototypeArgumentResult
    );
    standard_model_accessor!(
        external_provider_id,
        Pk(ExternalProviderId),
        AttributePrototypeArgumentResult
    );
    standard_model_accessor!(
        tail_component_id,
        Pk(ComponentId),
        AttributePrototypeArgumentResult
    );
    standard_model_accessor!(
        head_component_id,
        Pk(ComponentId),
        AttributePrototypeArgumentResult
    );

    /// Wraps the standard model accessor for "internal_provider_id" to ensure that a set value
    /// cannot become unset and vice versa.
    pub async fn set_internal_provider_id_safe(
        mut self,
        ctx: &DalContext,
        internal_provider_id: InternalProviderId,
    ) -> AttributePrototypeArgumentResult<()> {
        if self.internal_provider_id != UNSET_ID_VALUE.into()
            && internal_provider_id == UNSET_ID_VALUE.into()
        {
            return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
                "InternalProviderId",
            ));
        }
        if self.internal_provider_id == UNSET_ID_VALUE.into()
            && internal_provider_id != UNSET_ID_VALUE.into()
        {
            return Err(AttributePrototypeArgumentError::CannotFlipSetFieldToUnset(
                "InternalProviderId",
            ));
        }
        self.set_internal_provider_id(ctx, internal_provider_id)
            .await?;
        Ok(())
    }

    /// Wraps the standard model accessor for "external_provider_id" to ensure that a set value
    /// cannot become unset and vice versa.
    pub async fn set_external_provider_id_safe(
        mut self,
        ctx: &DalContext,
        external_provider_id: ExternalProviderId,
    ) -> AttributePrototypeArgumentResult<()> {
        if self.external_provider_id != UNSET_ID_VALUE.into()
            && external_provider_id == UNSET_ID_VALUE.into()
        {
            return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
                "ExternalProviderId",
            ));
        }
        if self.external_provider_id == UNSET_ID_VALUE.into()
            && external_provider_id != UNSET_ID_VALUE.into()
        {
            return Err(AttributePrototypeArgumentError::CannotFlipSetFieldToUnset(
                "ExternalProviderId",
            ));
        }
        self.set_external_provider_id(ctx, external_provider_id)
            .await?;
        Ok(())
    }

    /// Wraps the standard model accessor for "tail_component_id" to ensure that a set value
    /// cannot become unset and vice versa.
    pub async fn set_tail_component_id_safe(
        mut self,
        ctx: &DalContext,
        tail_component_id: ComponentId,
    ) -> AttributePrototypeArgumentResult<()> {
        if self.tail_component_id != UNSET_ID_VALUE.into()
            && tail_component_id == UNSET_ID_VALUE.into()
        {
            return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
                "tail ComponentId",
            ));
        }
        if self.tail_component_id == UNSET_ID_VALUE.into()
            && tail_component_id != UNSET_ID_VALUE.into()
        {
            return Err(AttributePrototypeArgumentError::CannotFlipSetFieldToUnset(
                "tail ComponentId",
            ));
        }
        self.set_tail_component_id(ctx, tail_component_id).await?;
        Ok(())
    }

    /// Wraps the standard model accessor for "head_component_id" to ensure that a set value
    /// cannot become unset and vice versa.
    pub async fn set_head_component_id_safe(
        mut self,
        ctx: &DalContext,
        head_component_id: ComponentId,
    ) -> AttributePrototypeArgumentResult<()> {
        if self.head_component_id != UNSET_ID_VALUE.into()
            && head_component_id == UNSET_ID_VALUE.into()
        {
            return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
                "head ComponentId",
            ));
        }
        if self.head_component_id == UNSET_ID_VALUE.into()
            && head_component_id != UNSET_ID_VALUE.into()
        {
            return Err(AttributePrototypeArgumentError::CannotFlipSetFieldToUnset(
                "head ComponentId",
            ));
        }
        self.set_head_component_id(ctx, head_component_id).await?;
        Ok(())
    }

    /// Determines if the [`InternalProviderId`](crate::InternalProvider) is unset. This function
    /// can be useful for determining how to build [`FuncBinding`](crate::FuncBinding) arguments.
    pub fn is_internal_provider_unset(&self) -> bool {
        self.internal_provider_id == UNSET_ID_VALUE.into()
    }

    /// List all [`AttributePrototypeArguments`](Self) for a given
    /// [`AttributePrototype`](crate::AttributePrototype).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_attribute_prototype(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeArgumentResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_ATTRIBUTE_PROTOTYPE,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                ],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// List all [`AttributePrototypeArguments`](Self) by name for a given
    /// [`AttributePrototype`](crate::AttributePrototype). This function should be used instead of
    /// [`Self::list_for_attribute_prototype()`] if the caller needs to group arguments that share
    /// the same "name" sharing the same name.
    #[tracing::instrument(skip(ctx))]
    pub async fn list_by_name_for_attribute_prototype_and_head_component_id(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        head_component_id: ComponentId,
    ) -> AttributePrototypeArgumentResult<Vec<AttributePrototypeArgumentGroup>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_BY_NAME_FOR_ATTRIBUTE_PROTOTYPE_AND_HEAD_COMPONENT_ID,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                    &head_component_id,
                ],
            )
            .await?;

        let mut result = Vec::new();
        for row in rows.into_iter() {
            let name: String = row.try_get("name")?;

            let arguments_json: Vec<serde_json::Value> = row.try_get("arguments")?;
            let mut arguments = Vec::new();
            for argument_json in arguments_json {
                arguments.push(serde_json::from_value(argument_json)?);
            }

            result.push(AttributePrototypeArgumentGroup { name, arguments });
        }
        Ok(result)
    }
}
