//! An [`AttributePrototypeArgument`] represents an argument name and its corresponding
//! [`InternalProvider`](crate::InternalProvider). An
//! [`AttributePrototype`](crate::AttributePrototype) can have multiple arguments.

use serde::{Deserialize, Serialize};
use si_data::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::provider::internal::InternalProviderId;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    standard_model_belongs_to, AttributePrototype, AttributePrototypeId, DalContext,
    HistoryEventError, InternalProvider, StandardModel, StandardModelError, Timestamp, Visibility,
    WriteTenancy,
};

const LIST_FOR_ATTRIBUTE_PROTOTYPE: &str =
    include_str!("../../queries/attribute_prototype_argument_list_for_attribute_prototype.sql");
const LIST_FOR_INTERNAL_PROVIDER: &str =
    include_str!("../../queries/attribute_prototype_argument_list_for_internal_provider.sql");

#[derive(Error, Debug)]
pub enum AttributePrototypeArgumentError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),

    #[error("attribute prototype error: {0}")]
    AttributePrototype(String),
    #[error(
        "attribute prototype argument for attribute prototype ({0}) with name ({1}) already exists"
    )]
    ArgumentWithNameForPrototypeAlreadyExists(AttributePrototypeId, String),
    #[error("attribute prototype argument does not belong to any attribute prototype")]
    NoAttributePrototypeRelationship,
}

pub type AttributePrototypeArgumentResult<T> = Result<T, AttributePrototypeArgumentError>;

pk!(AttributePrototypeArgumentPk);
pk!(AttributePrototypeArgumentId);

/// Contains a "key" and "value" for an argument that can be dynamically used
/// for [`AttributePrototypes`](crate::AttributePrototype).
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

    /// The "key" for a given argument. This field is immutable.
    name: String,
    /// The "value" for a given argument.
    internal_provider_id: InternalProviderId,
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
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        name: String,
        internal_provider_id: InternalProviderId,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM attribute_prototype_argument_create_v1($1, $2, $3, $4)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &internal_provider_id,
                ],
            )
            .await?;
        let object: AttributePrototypeArgument =
            standard_model::finish_create_from_row(ctx, row).await?;
        object
            .set_attribute_prototype(ctx, attribute_prototype_id)
            .await?;
        Ok(object)
    }

    standard_model_accessor_ro!(name, String);
    standard_model_accessor!(
        internal_provider_id,
        Pk(InternalProviderId),
        AttributePrototypeArgumentResult
    );
    standard_model_belongs_to!(
        lookup_fn: attribute_prototype,
        set_fn: set_attribute_prototype_unchecked,
        unset_fn: unset_attribute_prototype,
        table: "attribute_prototype_argument_belongs_to_attribute_prototype",
        model_table: "attribute_prototypes",
        belongs_to_id: AttributePrototypeId,
        returns: AttributePrototype,
        result: AttributePrototypeArgumentResult,
    );

    /// Sets [`Self`] to belong to a provided [`AttributePrototypeId`](crate::AttributePrototype).
    /// This only works if there are no existing arguments that share the same name.
    pub async fn set_attribute_prototype(
        &self,
        ctx: &DalContext<'_, '_>,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributePrototypeArgumentResult<()> {
        for argument in Self::list_for_attribute_prototype(ctx, attribute_prototype_id).await? {
            if argument.name == self.name {
                return Err(
                    AttributePrototypeArgumentError::ArgumentWithNameForPrototypeAlreadyExists(
                        attribute_prototype_id,
                        self.name.clone(),
                    ),
                );
            }
        }
        self.set_attribute_prototype_unchecked(ctx, &attribute_prototype_id)
            .await?;
        Ok(())
    }

    /// Get the [`InternalProvider`] via the corresponding field on [`Self`].
    pub async fn internal_provider(
        &self,
        ctx: &DalContext<'_, '_>,
    ) -> AttributePrototypeArgumentResult<Option<InternalProvider>> {
        Ok(InternalProvider::get_by_id(ctx, &self.internal_provider_id).await?)
    }

    /// Since the "name" field is immutable for [`Self`], this function creates a new [`Self`] using
    /// the new "name" and deletes the old [`Self`]. The sole, meaningful difference between the old
    /// and new argument should be the "name" field.
    pub async fn replace_with_new_name(
        mut self,
        ctx: &DalContext<'_, '_>,
        new_name: String,
    ) -> AttributePrototypeArgumentResult<Self> {
        let found_attribute_prototype = self
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributePrototypeArgumentError::NoAttributePrototypeRelationship)?;

        // We need to delete immediately before creation because the creation function will check
        // for a "name" collision.
        self.delete(ctx).await?;

        let new_argument = Self::new(
            ctx,
            new_name,
            self.internal_provider_id,
            *found_attribute_prototype.id(),
        )
        .await?;
        Ok(new_argument)
    }

    /// Find all [`Self`] for a given [`AttributePrototype`](crate::AttributePrototype).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_attribute_prototype(
        ctx: &DalContext<'_, '_>,
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

    /// Find all [`Self`] for a given [`InternalProvider`](crate::InternalProvider).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_internal_provider(
        ctx: &DalContext<'_, '_>,
        internal_provider_id: InternalProviderId,
    ) -> AttributePrototypeArgumentResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_INTERNAL_PROVIDER,
                &[ctx.read_tenancy(), ctx.visibility(), &internal_provider_id],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }
}
