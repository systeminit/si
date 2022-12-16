use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::binding::FuncBindingId;
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::schema::variant::SchemaVariantError;
use crate::socket::{Socket, SocketArity, SocketEdgeKind, SocketError, SocketId, SocketKind};
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    standard_model_has_many, AttributePrototype, AttributePrototypeError, ComponentId, DiagramKind,
    FuncId, HistoryEventError, InternalProviderId, SchemaVariant, StandardModel,
    StandardModelError, Timestamp, Visibility, WriteTenancy,
};
use crate::{
    AttributeContext, AttributeContextBuilderError, AttributeContextError, AttributePrototypeId,
    DalContext, SchemaId, SchemaVariantId,
};

const LIST_FOR_ATTRIBUTE_PROTOTYPE_WITH_TAIL_COMPONENT_ID: &str = include_str!(
    "../queries/external_provider_list_for_attribute_prototype_with_tail_component_id.sql"
);
const FIND_FOR_SCHEMA_VARIANT_AND_NAME: &str =
    include_str!("../queries/external_provider_find_for_schema_variant_and_name.sql");
const FIND_FOR_SOCKET: &str = include_str!("../queries/external_provider_find_for_socket.sql");
const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/external_provider_list_for_schema_variant.sql");
const LIST_FROM_INTERNAL_PROVIDER_USE: &str =
    include_str!("../queries/external_provider_list_from_internal_provider_use.sql");

#[derive(Error, Debug)]
pub enum ExternalProviderError {
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),

    #[error("unexpected: attribute prototype field is empty")]
    EmptyAttributePrototype,
    #[error("not found for id: {0}")]
    NotFound(ExternalProviderId),
    #[error("schema id mismatch: {0} (self) and {1} (provided)")]
    SchemaMismatch(SchemaId, SchemaId),
    #[error("schema variant id mismatch: {0} (self) and {1} (provided)")]
    SchemaVariantMismatch(SchemaVariantId, SchemaVariantId),
    #[error("schema variant error: {0}")]
    SchemaVariant(String),
}

pub type ExternalProviderResult<T> = Result<T, ExternalProviderError>;

pk!(ExternalProviderPk);
pk!(ExternalProviderId);

impl_standard_model! {
    model: ExternalProvider,
    pk: ExternalProviderPk,
    id: ExternalProviderId,
    table_name: "external_providers",
    history_event_label_base: "external_provider",
    history_event_message_name: "External Provider"
}

/// This provider can only provide data to external [`SchemaVariants`](crate::SchemaVariant). It can
/// only consume data within its own [`SchemaVariant`](crate::SchemaVariant).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ExternalProvider {
    pk: ExternalProviderPk,
    id: ExternalProviderId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    visibility: Visibility,
    #[serde(flatten)]
    timestamp: Timestamp,

    /// Indicates which [`Schema`](crate::Schema) this provider belongs to.
    schema_id: SchemaId,
    /// Indicates which [`SchemaVariant`](crate::SchemaVariant) this provider belongs to.
    schema_variant_id: SchemaVariantId,
    /// Indicates which transformation function should be used for "emit".
    attribute_prototype_id: Option<AttributePrototypeId>,

    /// Name for [`Self`] that can be used for identification.
    name: String,
    /// Definition of the data type (e.g. "JSONSchema" or "Number").
    type_definition: Option<String>,
}

impl ExternalProvider {
    /// This function will also create an _output_ [`Socket`](crate::Socket).
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(ctx, name))]
    pub async fn new_with_socket(
        ctx: &DalContext,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        name: impl AsRef<str>,
        type_definition: Option<String>,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        func_binding_return_value_id: FuncBindingReturnValueId,
        arity: SocketArity,
        diagram_kind: DiagramKind,
    ) -> ExternalProviderResult<(Self, Socket)> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM external_provider_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &schema_id,
                    &schema_variant_id,
                    &name,
                    &type_definition,
                ],
            )
            .await?;

        let mut external_provider: ExternalProvider =
            standard_model::finish_create_from_row(ctx, row).await?;

        let attribute_context = AttributeContext::builder()
            .set_external_provider_id(external_provider.id)
            .to_context()?;
        let attribute_prototype = AttributePrototype::new(
            ctx,
            func_id,
            func_binding_id,
            func_binding_return_value_id,
            attribute_context,
            None,
            None,
        )
        .await?;
        external_provider
            .set_attribute_prototype_id(ctx, Some(*attribute_prototype.id()))
            .await?;

        let socket = Socket::new(
            ctx,
            name,
            SocketKind::Provider,
            &SocketEdgeKind::ConfigurationOutput,
            &arity,
            &diagram_kind,
        )
        .await?;
        socket
            .set_external_provider(ctx, external_provider.id())
            .await?;

        let variant = SchemaVariant::get_by_id(ctx, external_provider.schema_variant_id())
            .await?
            .ok_or_else(|| {
                ExternalProviderError::SchemaVariant(
                    SchemaVariantError::NotFound(*external_provider.schema_variant_id())
                        .to_string(),
                )
            })?;
        variant
            .add_socket(ctx, socket.id())
            .await
            .map_err(|err| ExternalProviderError::SchemaVariant(err.to_string()))?;

        Ok((external_provider, socket))
    }

    // Immutable fields.
    standard_model_accessor_ro!(schema_id, SchemaId);
    standard_model_accessor_ro!(schema_variant_id, SchemaVariantId);

    // Mutable fields.
    standard_model_accessor!(name, String, ExternalProviderResult);
    standard_model_accessor!(type_definition, Option<String>, ExternalProviderResult);
    standard_model_accessor!(
        attribute_prototype_id,
        Option<Pk(AttributePrototypeId)>,
        ExternalProviderResult
    );

    // This is a 1-1 relationship, so the Vec<Socket> should be 1
    standard_model_has_many!(
        lookup_fn: sockets,
        table: "socket_belongs_to_external_provider",
        model_table: "sockets",
        returns: Socket,
        result: ExternalProviderResult,
    );

    /// Find all [`Self`] for a given [`SchemaVariant`](crate::SchemaVariant).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ExternalProviderResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_SCHEMA_VARIANT,
                &[ctx.read_tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Find [`Self`] with a provided [`SocketId`](crate::Socket).
    #[instrument(skip_all)]
    pub async fn find_for_socket(
        ctx: &DalContext,
        socket_id: SocketId,
    ) -> ExternalProviderResult<Option<Self>> {
        let row = ctx
            .txns()
            .pg()
            .query_opt(
                FIND_FOR_SOCKET,
                &[ctx.read_tenancy(), ctx.visibility(), &socket_id],
            )
            .await?;
        Ok(standard_model::object_option_from_row_option(row)?)
    }

    /// Find [`Self`] with a provided name, which is not only the name of [`Self`], but also of the
    /// associated _output_ [`Socket`](crate::Socket).
    #[instrument(skip_all)]
    pub async fn find_for_schema_variant_and_name(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        name: impl AsRef<str>,
    ) -> ExternalProviderResult<Option<Self>> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_opt(
                FIND_FOR_SCHEMA_VARIANT_AND_NAME,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &schema_variant_id,
                    &name,
                ],
            )
            .await?;
        Ok(standard_model::object_option_from_row_option(row)?)
    }

    /// Find all [`Self`] for a given [`AttributePrototypeId`](crate::AttributePrototype).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_for_attribute_prototype_with_tail_component_id(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        tail_component_id: ComponentId,
    ) -> ExternalProviderResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_ATTRIBUTE_PROTOTYPE_WITH_TAIL_COMPONENT_ID,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &attribute_prototype_id,
                    &tail_component_id,
                ],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Find all [`Self`] that have
    /// [`AttributePrototypeArguments`](crate::AttributePrototypeArgument) referencing the provided
    /// [`InternalProviderId`](crate::InternalProvider).
    #[tracing::instrument(skip(ctx))]
    pub async fn list_from_internal_provider_use(
        ctx: &DalContext,
        internal_provider_id: InternalProviderId,
    ) -> ExternalProviderResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FROM_INTERNAL_PROVIDER_USE,
                &[ctx.read_tenancy(), ctx.visibility(), &internal_provider_id],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }
}
