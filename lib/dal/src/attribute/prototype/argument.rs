//! An [`AttributePrototypeArgument`] represents an argument name and how to dynamically derive
//! the corresponding value. [`AttributePrototype`](crate::AttributePrototype) can have multiple
//! arguments.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data_pg::PgError;
use telemetry::prelude::*;

use crate::{
    func::argument::FuncArgumentId, impl_standard_model, pk,
    provider::internal::InternalProviderId, standard_model, standard_model_accessor,
    AttributePrototypeId, ComponentId, DalContext, ExternalProviderId, HistoryEventError,
    StandardModel, StandardModelError, Tenancy, Timestamp, TransactionsError, Visibility,
};

const LIST_FOR_ATTRIBUTE_PROTOTYPE: &str =
    include_str!("../../queries/attribute_prototype_argument/list_for_attribute_prototype.sql");
const LIST_FOR_FUNC_ARGUMENT_ID: &str =
    include_str!("../../queries/attribute_prototype_argument/list_for_func_argument.sql");
const FIND_FOR_PROVIDERS_AND_COMPONENTS: &str = include_str!(
    "../../queries/attribute_prototype_argument/find_for_providers_and_components.sql"
);

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributePrototypeArgumentError {
    #[error("cannot update set field to become unset: {0}")]
    CannotFlipSetFieldToUnset(&'static str),
    #[error("cannot update unset field to become set: {0}")]
    CannotFlipUnsetFieldToSet(&'static str),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("required value fields must be set, found at least one unset required value field")]
    RequiredValueFieldsUnset,
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
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
    tenancy: Tenancy,
    #[serde(flatten)]
    visibility: Visibility,
    #[serde(flatten)]
    timestamp: Timestamp,

    /// Indicates the [`AttributePrototype`](crate::AttributePrototype) that [`Self`] is used as
    /// an argument for.
    attribute_prototype_id: AttributePrototypeId,
    /// Where to find the name and type of the "key" for a given argument.
    func_argument_id: FuncArgumentId,
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
    /// Create a new [`AttributePrototypeArgument`] for _intra_ [`Component`](crate::Component) use.
    pub async fn new_for_intra_component(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        func_argument_id: FuncArgumentId,
        internal_provider_id: InternalProviderId,
    ) -> AttributePrototypeArgumentResult<Self> {
        // Ensure the value fields are what we expect.
        let external_provider_id = ExternalProviderId::NONE;
        let tail_component_id = ComponentId::NONE;
        let head_component_id = ComponentId::NONE;
        if internal_provider_id == InternalProviderId::NONE {
            return Err(AttributePrototypeArgumentError::RequiredValueFieldsUnset);
        }

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                    &func_argument_id,
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
    pub async fn new_for_inter_component(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        func_argument_id: FuncArgumentId,
        head_component_id: ComponentId,
        tail_component_id: ComponentId,
        external_provider_id: ExternalProviderId,
    ) -> AttributePrototypeArgumentResult<Self> {
        // Ensure the value fields are what we expect.
        if external_provider_id == ExternalProviderId::NONE
            || tail_component_id == ComponentId::NONE
            || head_component_id == ComponentId::NONE
        {
            return Err(AttributePrototypeArgumentError::RequiredValueFieldsUnset);
        }

        // For inter component connections, the internal provider id field must be unset.
        let internal_provider_id = InternalProviderId::NONE;

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                    &func_argument_id,
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
    pub async fn new_explicit_internal_to_explicit_internal_inter_component(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        func_argument_id: FuncArgumentId,
        head_component_id: ComponentId,
        tail_component_id: ComponentId,
        internal_provider_id: InternalProviderId,
    ) -> AttributePrototypeArgumentResult<Self> {
        // Ensure the value fields are what we expect.
        if internal_provider_id == InternalProviderId::NONE
            || tail_component_id == ComponentId::NONE
            || head_component_id == ComponentId::NONE
        {
            return Err(AttributePrototypeArgumentError::RequiredValueFieldsUnset);
        }

        // For inter component connections, the internal provider id field must be unset.
        let external_provider_id = ExternalProviderId::NONE;

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                    &func_argument_id,
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
    pub async fn new_external_to_external_inter_component(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        func_argument_id: FuncArgumentId,
        head_component_id: ComponentId,
        tail_component_id: ComponentId,
        external_provider_id: ExternalProviderId,
    ) -> AttributePrototypeArgumentResult<Self> {
        // Ensure the value fields are what we expect.
        if external_provider_id == ExternalProviderId::NONE
            || tail_component_id == ComponentId::NONE
            || head_component_id == ComponentId::NONE
        {
            return Err(AttributePrototypeArgumentError::RequiredValueFieldsUnset);
        }

        // For inter component connections, the internal provider id field must be unset.
        let internal_provider_id = InternalProviderId::NONE;

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                    &func_argument_id,
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
    standard_model_accessor!(
        func_argument_id,
        Pk(FuncArgumentId),
        AttributePrototypeArgumentResult
    );
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
        &mut self,
        ctx: &DalContext,
        internal_provider_id: InternalProviderId,
    ) -> AttributePrototypeArgumentResult<()> {
        if self.internal_provider_id != InternalProviderId::NONE
            && internal_provider_id == InternalProviderId::NONE
        {
            return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
                "InternalProviderId",
            ));
        };
        if self.internal_provider_id == InternalProviderId::NONE
            && internal_provider_id != InternalProviderId::NONE
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
        if self.external_provider_id != ExternalProviderId::NONE
            && external_provider_id == ExternalProviderId::NONE
        {
            return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
                "ExternalProviderId",
            ));
        }
        if self.external_provider_id == ExternalProviderId::NONE
            && external_provider_id != ExternalProviderId::NONE
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
        if self.tail_component_id != ComponentId::NONE && tail_component_id == ComponentId::NONE {
            return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
                "tail ComponentId",
            ));
        }
        if self.tail_component_id == ComponentId::NONE && tail_component_id != ComponentId::NONE {
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
        if self.head_component_id != ComponentId::NONE && head_component_id == ComponentId::NONE {
            return Err(AttributePrototypeArgumentError::CannotFlipUnsetFieldToSet(
                "head ComponentId",
            ));
        }
        if self.head_component_id == ComponentId::NONE && head_component_id != ComponentId::NONE {
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
        self.internal_provider_id == InternalProviderId::NONE
    }

    /// List all [`AttributePrototypeArguments`](Self) for a given
    /// [`AttributePrototype`](crate::AttributePrototype).
    pub async fn list_for_attribute_prototype(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeArgumentResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_FOR_ATTRIBUTE_PROTOTYPE,
                &[ctx.tenancy(), ctx.visibility(), &attribute_prototype_id],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// List all [`AttributePrototypeArguments`](Self) for a given [`FuncArgument`](crate::func::argument::FuncArgument).
    pub async fn list_by_func_argument_id(
        ctx: &DalContext,
        func_argument_id: FuncArgumentId,
    ) -> AttributePrototypeArgumentResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_FOR_FUNC_ARGUMENT_ID,
                &[ctx.tenancy(), ctx.visibility(), &func_argument_id],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn find_for_providers_and_components(
        ctx: &DalContext,
        external_provider_id: &ExternalProviderId,
        internal_provider_id: &InternalProviderId,
        tail_component: &ComponentId,
        head_component: &ComponentId,
    ) -> AttributePrototypeArgumentResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                FIND_FOR_PROVIDERS_AND_COMPONENTS,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    external_provider_id,
                    internal_provider_id,
                    tail_component,
                    head_component,
                ],
            )
            .await?;

        Ok(standard_model::object_option_from_row_option(row)?)
    }
}
