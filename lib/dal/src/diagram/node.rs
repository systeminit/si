use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

use crate::change_status::ChangeStatus;
use crate::diagram::DiagramResult;
use crate::schema::SchemaUiMenu;
use crate::socket::{SocketArity, SocketEdgeKind};
use crate::{
    ActorView, ChangeSetPk, Component, ComponentStatus, DalContext, DiagramError,
    HistoryActorTimestamp, Node, NodePosition, ResourceView, SchemaVariant, StandardModel,
};

#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketDirection {
    Input,
    Output,
    Bidirectional,
}

#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum NodeSide {
    Left,
    Right,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SocketView {
    id: String,
    label: String,
    #[serde(rename = "type")]
    ty: String,
    direction: SocketDirection,
    max_connections: Option<usize>,
    is_required: Option<bool>,
    node_side: NodeSide,
}

impl SocketView {
    pub async fn list(
        ctx: &DalContext,
        schema_variant: &SchemaVariant,
    ) -> DiagramResult<Vec<Self>> {
        Ok(schema_variant
            .sockets(ctx)
            .await?
            .into_iter()
            .map(|socket| {
                Self {
                    id: socket.id().to_string(),
                    label: socket.name().to_owned(),
                    ty: socket.name().to_owned(),
                    // Note: it's not clear if this mapping is correct, and there is no backend support for bidirectional sockets for now
                    direction: match socket.edge_kind() {
                        SocketEdgeKind::ConfigurationOutput => SocketDirection::Output,
                        _ => SocketDirection::Input,
                    },
                    max_connections: match socket.arity() {
                        SocketArity::Many => None,
                        SocketArity::One => Some(1),
                    },
                    is_required: Some(socket.required()),
                    node_side: match socket.edge_kind() {
                        SocketEdgeKind::ConfigurationOutput => NodeSide::Right,
                        _ => NodeSide::Left,
                    },
                }
            })
            .collect())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GridPoint {
    x: isize,
    y: isize,
}

impl GridPoint {
    pub fn x(&self) -> isize {
        self.x
    }

    pub fn y(&self) -> isize {
        self.y
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Size2D {
    width: isize,
    height: isize,
}

impl Size2D {
    pub fn width(&self) -> isize {
        self.width
    }
    pub fn height(&self) -> isize {
        self.height
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramComponentView {
    id: String,
    node_id: String,
    display_name: Option<String>,

    schema_name: String,
    schema_id: String,
    schema_variant_id: String,
    schema_variant_name: String,
    schema_category: Option<String>,

    sockets: Option<Vec<SocketView>>,
    position: GridPoint,
    size: Option<Size2D>,
    color: Option<String>,
    node_type: String,
    change_status: ChangeStatus,
    resource: ResourceView,

    created_info: HistoryEventMetadata,
    updated_info: HistoryEventMetadata,

    // TODO: get the right history event so we can show the actor
    // deleted_info: Option<HistoryEventMetadata>,
    deleted_at: Option<DateTime<Utc>>,
}

impl DiagramComponentView {
    pub async fn new(
        ctx: &DalContext,
        component: &Component,
        node: &Node,
        position: &NodePosition,
        is_modified: bool,
        schema_variant: &SchemaVariant,
    ) -> DiagramResult<Self> {
        let schema = schema_variant
            .schema(ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;

        let schema_category = SchemaUiMenu::find_for_schema(ctx, *schema.id())
            .await?
            .map(|um| um.category().to_string());

        let size = if let (Some(w), Some(h)) = (position.width(), position.height()) {
            Some(Size2D {
                height: h.parse()?,
                width: w.parse()?,
            })
        } else {
            None
        };

        let x = position.x().parse::<f64>()?;
        let y = position.y().parse::<f64>()?;

        let change_status = if node.visibility().deleted_at.is_some() {
            ChangeStatus::Deleted
        } else if node.visibility().change_set_pk != ChangeSetPk::NONE {
            ChangeStatus::Added
        } else if is_modified {
            ChangeStatus::Modified
        } else {
            ChangeStatus::Unmodified
        };

        // TODO: not really the right error...?
        let component_status = ComponentStatus::get_by_id(ctx, component.id())
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let created_info =
            HistoryEventMetadata::from_history_actor_timestamp(ctx, component_status.creation())
                .await?;
        let updated_info =
            HistoryEventMetadata::from_history_actor_timestamp(ctx, component_status.update())
                .await?;

        // TODO(theo): probably dont want to fetch this here and load totally separately, but we inherited from existing endpoints
        let resource = ResourceView::new(component.resource(ctx).await?);

        Ok(Self {
            id: component.id().to_string(),
            node_id: node.id().to_string(),
            display_name: Some(component.name(ctx).await?),

            schema_name: schema.name().to_owned(),
            schema_variant_name: schema_variant.name().to_owned(),
            schema_id: schema.id().to_string(),
            schema_variant_id: schema_variant.id().to_string(),
            schema_category,
            sockets: Some(SocketView::list(ctx, schema_variant).await?),
            position: GridPoint {
                x: x.round() as isize,
                y: y.round() as isize,
            },
            size,
            color: component
                .find_value_by_json_pointer::<String>(ctx, "/root/si/Color")
                .await?
                .or_else(|| {
                    schema_variant
                        .color()
                        .map(|color_int| format!("#{color_int:x}"))
                }),
            node_type: component
                .find_value_by_json_pointer::<String>(ctx, "/root/si/type")
                .await?
                .unwrap_or_else(|| "component".to_string()),
            change_status,
            resource,

            created_info,
            updated_info,
            deleted_at: component.visibility().deleted_at, // TODO: get deleted info instead
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn position(&self) -> &GridPoint {
        &self.position
    }

    pub fn size(&self) -> &Option<Size2D> {
        &self.size
    }
}

// TODO(theo,victor): this should probably move and be used more generally in a few places?

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEventMetadata {
    actor: ActorView,
    timestamp: DateTime<Utc>,
}

impl HistoryEventMetadata {
    async fn from_history_actor_timestamp(
        ctx: &DalContext,
        value: HistoryActorTimestamp,
    ) -> DiagramResult<Self> {
        let actor = ActorView::from_history_actor(ctx, value.actor).await?;

        Ok(Self {
            actor,
            timestamp: value.timestamp,
        })
    }
}
