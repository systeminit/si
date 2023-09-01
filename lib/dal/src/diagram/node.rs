use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use crate::change_status::ChangeStatus;
use crate::diagram::DiagramResult;
use crate::schema::SchemaUiMenu;
use crate::socket::{SocketArity, SocketEdgeKind};
use crate::{
    history_event, ActionPrototype, ActionPrototypeContext, ActionPrototypeView, ActorView,
    Component, ComponentId, ComponentStatus, ComponentType, DalContext, DiagramError,
    HistoryActorTimestamp, Node, NodeId, ResourceView, SchemaVariant, StandardModel,
};

#[remain::sorted]
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
    Bidirectional,
    Input,
    Output,
}

#[remain::sorted]
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
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub direction: SocketDirection,
    pub max_connections: Option<usize>,
    pub is_required: Option<bool>,
    pub node_side: NodeSide,
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
            .filter_map(|socket| {
                (!socket.ui_hidden()).then(|| Self {
                    id: socket.id().to_string(),
                    label: socket.human_name().unwrap_or(socket.name()).to_owned(),
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
                })
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
    id: ComponentId,
    node_id: NodeId,
    display_name: Option<String>,

    parent_node_id: Option<NodeId>,
    child_node_ids: Vec<NodeId>,

    schema_name: String,
    schema_id: String,
    schema_variant_id: String,
    schema_variant_name: String,
    schema_category: Option<String>,

    actions: Vec<ActionPrototypeView>,

    sockets: Option<Vec<SocketView>>,
    position: GridPoint,
    size: Option<Size2D>,
    color: Option<String>,
    node_type: ComponentType,
    change_status: ChangeStatus,
    resource: ResourceView,

    created_info: HistoryEventMetadata,
    updated_info: HistoryEventMetadata,

    deleted_info: Option<HistoryEventMetadata>,
}

impl DiagramComponentView {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        component: &Component,
        node: &Node,
        parent_node_id: Option<NodeId>,
        child_node_ids: Vec<NodeId>,
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

        let size = if let (Some(w), Some(h)) = (node.width(), node.height()) {
            Some(Size2D {
                height: h.parse()?,
                width: w.parse()?,
            })
        } else {
            None
        };

        let x = node.x().parse::<f64>()?;
        let y = node.y().parse::<f64>()?;

        // Change status should track the component, not the node, since node position is on the
        // node and the node will change if it is moved
        let change_status = if component.visibility().deleted_at.is_some() {
            ChangeStatus::Deleted
        } else if !component.exists_in_head(ctx).await? {
            ChangeStatus::Added
        } else if is_modified {
            ChangeStatus::Modified
        } else {
            ChangeStatus::Unmodified
        };

        let component_status = ComponentStatus::get_by_id(ctx, component.id())
            .await?
            .ok_or_else(|| DiagramError::ComponentStatusNotFound(*component.id()))?;

        let created_info =
            HistoryEventMetadata::from_history_actor_timestamp(ctx, component_status.creation())
                .await?;
        let updated_info =
            HistoryEventMetadata::from_history_actor_timestamp(ctx, component_status.update())
                .await?;

        let mut deleted_info: Option<HistoryEventMetadata> = None;
        {
            if let Some(deleted_at) = ctx.visibility().deleted_at {
                if let Some(deletion_user_pk) = component.deletion_user_pk {
                    let history_actor = history_event::HistoryActor::User(deletion_user_pk);
                    let actor = ActorView::from_history_actor(ctx, history_actor).await?;

                    deleted_info = Some(HistoryEventMetadata {
                        actor,
                        timestamp: deleted_at,
                    });
                }
            }
        }

        // TODO(theo): probably dont want to fetch this here and load totally separately, but we inherited from existing endpoints
        let resource = ResourceView::new(component.resource(ctx).await?);

        let action_prototypes = ActionPrototype::find_for_context(
            ctx,
            ActionPrototypeContext {
                schema_variant_id: *schema_variant.id(),
            },
        )
        .await?;
        let actions = action_prototypes
            .into_iter()
            .map(ActionPrototypeView::new)
            .collect();

        Ok(Self {
            id: *component.id(),
            node_id: *node.id(),
            parent_node_id,
            child_node_ids,
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
            color: component.color(ctx).await?,
            node_type: component.get_type(ctx).await?,
            change_status,
            resource,
            actions,
            created_info,
            updated_info,
            deleted_info,
        })
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn position(&self) -> &GridPoint {
        &self.position
    }

    pub fn size(&self) -> &Option<Size2D> {
        &self.size
    }

    pub fn resource(&self) -> &ResourceView {
        &self.resource
    }
}

// TODO(theo,victor): this should probably move and be used more generally in a few places?

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEventMetadata {
    pub(crate) actor: ActorView,
    pub(crate) timestamp: DateTime<Utc>,
}

impl HistoryEventMetadata {
    pub async fn from_history_actor_timestamp(
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
