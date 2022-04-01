use crate::WriteTenancy;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    edit_field::{
        value_and_visibility_diff, widget::prelude::*, EditField, EditFieldAble, EditFieldDataType,
        EditFieldError, EditFieldObjectKind, EditFields, VisibilityDiff,
    },
    impl_standard_model, pk,
    schema::builtins::{create_root_prop, RootProp},
    schema::SchemaError,
    socket::{Socket, SocketError, SocketId},
    standard_model::{self, objects_from_rows},
    standard_model_accessor, standard_model_belongs_to, standard_model_many_to_many, DalContext,
    HistoryEventError, Prop, PropError, PropId, PropKind, Schema, SchemaId, StandardModel,
    StandardModelError, Timestamp, Visibility, WsEventError,
};

#[derive(Error, Debug)]
pub enum SchemaVariantError {
    #[error(transparent)]
    EditField(#[from] EditFieldError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("schema not found: {0}")]
    NotFound(SchemaVariantId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
}

pub type SchemaVariantResult<T> = Result<T, SchemaVariantError>;

const ALL_PROPS: &str = include_str!("../queries/schema_variant_all_props.sql");

pk!(SchemaVariantPk);
pk!(SchemaVariantId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SchemaVariant {
    pk: SchemaVariantPk,
    id: SchemaVariantId,
    name: String,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: SchemaVariant,
    pk: SchemaVariantPk,
    id: SchemaVariantId,
    table_name: "schema_variants",
    history_event_label_base: "schema_variant",
    history_event_message_name: "Schema Variant"
}

impl SchemaVariant {
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        schema_id: SchemaId,
        name: impl AsRef<str>,
    ) -> SchemaVariantResult<(Self, RootProp)> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM schema_variant_create_v1($1, $2, $3)",
                &[ctx.write_tenancy(), ctx.visibility(), &name],
            )
            .await?;
        let object: SchemaVariant = standard_model::finish_create_from_row(ctx, row).await?;
        let root_prop = create_root_prop(ctx, schema_id, *object.id())
            .await
            .map_err(Box::new)?;

        object.set_schema(ctx, &schema_id).await?;

        Ok((object, root_prop))
    }

    standard_model_accessor!(name, String, SchemaVariantResult);

    standard_model_belongs_to!(
        lookup_fn: schema,
        set_fn: set_schema,
        unset_fn: unset_schema,
        table: "schema_variant_belongs_to_schema",
        model_table: "schemas",
        belongs_to_id: SchemaId,
        returns: Schema,
        result: SchemaVariantResult,
    );

    standard_model_many_to_many!(
        lookup_fn: sockets,
        associate_fn: add_socket,
        disassociate_fn: remove_socket,
        table_name: "socket_many_to_many_schema_variants",
        left_table: "sockets",
        left_id: SocketId,
        right_table: "schema_variants",
        right_id: SchemaId,
        which_table_is_this: "right",
        returns: Socket,
        result: SchemaVariantResult,
    );

    standard_model_many_to_many!(
        lookup_fn: props,
        associate_fn: add_prop,
        disassociate_fn: remove_prop,
        table_name: "prop_many_to_many_schema_variants",
        left_table: "props",
        left_id: PropId,
        right_table: "schema_variants",
        right_id: SchemaVariantId,
        which_table_is_this: "right",
        returns: Prop,
        result: SchemaVariantResult,
    );

    pub async fn all_props(&self, ctx: &DalContext<'_, '_>) -> SchemaVariantResult<Vec<Prop>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                ALL_PROPS,
                &[ctx.read_tenancy(), ctx.visibility(), self.id()],
            )
            .await?;
        let results = objects_from_rows(rows)?;
        Ok(results)
    }

    fn edit_field_object_kind() -> EditFieldObjectKind {
        EditFieldObjectKind::SchemaVariant
    }

    fn name_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> SchemaVariantResult<EditField> {
        let field_name = "name";
        let target_fn = Self::name;

        let (value, visibility_diff) = value_and_visibility_diff(
            visibility,
            Some(object),
            target_fn,
            head_object.as_ref(),
            change_set_object.as_ref(),
        )?;

        Ok(EditField::new(
            field_name,
            vec![],
            Self::edit_field_object_kind(),
            object.id,
            EditFieldDataType::String,
            Widget::Text(TextWidget::new()),
            value,
            visibility_diff,
            vec![], // TODO: actually validate to generate ValidationErrors
        ))
    }

    async fn properties_edit_field(
        ctx: &DalContext<'_, '_>,
        object: &Self,
    ) -> SchemaVariantResult<EditField> {
        let field_name = "properties";

        let mut items: Vec<EditFields> = vec![];
        for prop in object.props(ctx).await?.into_iter() {
            let edit_fields = Prop::get_edit_fields(ctx, prop.id()).await?;
            items.push(edit_fields);
        }
        Ok(EditField::new(
            field_name,
            vec![],
            EditFieldObjectKind::Prop,
            object.id,
            EditFieldDataType::Array,
            Widget::Array(items.into()),
            None,
            VisibilityDiff::None,
            vec![], // TODO: actually validate to generate ValidationErrors
        ))
    }

    async fn connections_edit_field(
        ctx: &DalContext<'_, '_>,
        object: &Self,
    ) -> SchemaVariantResult<EditField> {
        let field_name = "connections";

        let mut items: Vec<EditFields> = vec![];
        for socket in object.sockets(ctx).await?.into_iter() {
            let edit_fields = Socket::get_edit_fields(ctx, socket.id()).await?;
            items.push(edit_fields);
        }

        Ok(EditField::new(
            field_name,
            vec![],
            Self::edit_field_object_kind(),
            object.id,
            EditFieldDataType::None,
            Widget::Header(HeaderWidget::new(vec![EditField::new(
                "sockets",
                vec![field_name.to_string()],
                EditFieldObjectKind::SchemaVariant,
                object.id,
                EditFieldDataType::Array,
                Widget::Array(items.into()),
                None,
                VisibilityDiff::None,
                vec![], // TODO: actually validate to generate ValidationErrors
            )])),
            None,
            VisibilityDiff::None,
            vec![], // TODO: actually validate to generate ValidationErrors
        ))
    }
}

#[async_trait]
impl EditFieldAble for SchemaVariant {
    type Id = SchemaVariantId;
    type Error = SchemaVariantError;

    #[instrument(skip_all)]
    async fn get_edit_fields(
        ctx: &DalContext<'_, '_>,
        id: &Self::Id,
    ) -> Result<EditFields, Self::Error> {
        let object = Self::get_by_id(ctx, id)
            .await?
            .ok_or(SchemaVariantError::NotFound(*id))?;
        let head_object = if ctx.visibility().in_change_set() {
            let head_visibility = ctx.visibility().to_head();
            let ctx = ctx.clone_with_new_visibility(head_visibility);
            Self::get_by_id(&ctx, id).await?
        } else {
            None
        };
        let change_set_object = if ctx.visibility().in_change_set() {
            let change_set_visibility = ctx.visibility().to_change_set();
            let ctx = ctx.clone_with_new_visibility(change_set_visibility);
            Self::get_by_id(&ctx, id).await?
        } else {
            None
        };

        let edit_fields = vec![
            Self::name_edit_field(ctx.visibility(), &object, &head_object, &change_set_object)?,
            Self::properties_edit_field(ctx, &object).await?,
            Self::connections_edit_field(ctx, &object).await?,
        ];

        Ok(edit_fields)
    }

    #[instrument(skip_all)]
    async fn update_from_edit_field(
        ctx: &DalContext<'_, '_>,
        id: Self::Id,
        edit_field_id: String,
        value: Option<Value>,
    ) -> Result<(), Self::Error> {
        let mut object = Self::get_by_id(ctx, &id)
            .await?
            .ok_or(SchemaVariantError::NotFound(id))?;

        match edit_field_id.as_ref() {
            "name" => match value {
                Some(json_value) => {
                    let value = json_value.as_str().map(|s| s.to_string()).ok_or(
                        Self::Error::EditField(EditFieldError::InvalidValueType("string")),
                    )?;
                    object.set_name(ctx, value).await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            "properties" => {
                // TODO(fnichol): we're sticking in arbitrary default values--these become required
                // field entries on a "new item" form somewhere
                let prop = Prop::new(ctx, "TODO: name me!", PropKind::String).await?;
                prop.add_schema_variant(ctx, object.id()).await?;
            }
            "connections.sockets" => {
                // TODO(fnichol): we're sticking in arbitrary default values--these become required
                // field entries on a "new item" form somewhere
                let socket = Socket::new(
                    ctx,
                    "TODO: name me!",
                    &crate::socket::SocketEdgeKind::Deployment,
                    &crate::socket::SocketArity::One,
                )
                .await?;
                socket.add_type(ctx, object.id()).await?;
            }
            invalid => return Err(EditFieldError::invalid_field(invalid).into()),
        }

        Ok(())
    }
}
