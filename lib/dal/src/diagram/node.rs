use chrono::Utc;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use crate::change_status::ChangeStatus;
use crate::diagram::DiagramResult;
use crate::history_event::{HistoryActorTimestamp, HistoryEventMetadata};
use crate::schema::variant::root_prop::component_type::ComponentType;
use crate::{Component, ComponentId, DalContext, ProviderArity, SchemaVariant, SchemaVariantId};

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
    Hash,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum DiagramSocketDirection {
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
pub enum DiagramNodeSide {
    Left,
    Right,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramSocketView {
    pub id: String,
    pub label: String,
    pub connection_annotations: Vec<String>,
    pub direction: DiagramSocketDirection,
    pub max_connections: Option<usize>,
    pub is_required: Option<bool>,
    pub node_side: DiagramNodeSide,
}

impl DiagramSocketView {
    pub async fn list(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> DiagramResult<Vec<Self>> {
        let mut socket_views = Vec::new();
        let (external_providers, explicit_internal_providers) =
            SchemaVariant::list_external_providers_and_explicit_internal_providers(
                ctx,
                schema_variant_id,
            )
            .await?;

        for external_provider in external_providers {
            if !external_provider.ui_hidden() {
                socket_views.push(Self {
                    id: external_provider.id().to_string(),
                    label: external_provider.name().to_owned(),
                    // todo: implement connection annotations in graph work
                    connection_annotations: vec![external_provider.name().to_owned()],
                    direction: DiagramSocketDirection::Output,
                    max_connections: match external_provider.arity() {
                        ProviderArity::Many => None,
                        ProviderArity::One => Some(1),
                    },
                    is_required: Some(external_provider.required()),
                    node_side: DiagramNodeSide::Right,
                });
            }
        }

        for explicit_internal_provider in explicit_internal_providers {
            if !explicit_internal_provider.ui_hidden() {
                socket_views.push(Self {
                    id: explicit_internal_provider.id().to_string(),
                    label: explicit_internal_provider.name().to_owned(),
                    // todo: implement connection annotations in graph work
                    connection_annotations: vec![explicit_internal_provider.name().to_owned()],
                    direction: DiagramSocketDirection::Input,
                    max_connections: match explicit_internal_provider.arity() {
                        ProviderArity::Many => None,
                        ProviderArity::One => Some(1),
                    },
                    is_required: Some(explicit_internal_provider.required()),
                    node_side: DiagramNodeSide::Left,
                });
            }
        }

        Ok(socket_views)
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
    display_name: Option<String>,

    parent_component_id: Option<ComponentId>,
    child_component_ids: Vec<ComponentId>,

    schema_name: String,
    schema_id: String,
    schema_variant_id: String,
    schema_variant_name: String,
    schema_category: Option<String>,

    sockets: Option<Vec<DiagramSocketView>>,
    position: GridPoint,
    size: Option<Size2D>,
    color: Option<String>,
    node_type: ComponentType,
    change_status: ChangeStatus,
    has_resource: bool,
    created_info: HistoryEventMetadata,
    updated_info: HistoryEventMetadata,

    deleted_info: Option<HistoryEventMetadata>,
}

impl DiagramComponentView {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        component: &Component,
        parent_component_id: Option<ComponentId>,
        child_component_ids: Vec<ComponentId>,
        _is_modified: bool,
        schema_variant: &SchemaVariant,
    ) -> DiagramResult<Self> {
        let size = if let (Some(w), Some(h)) = (component.width(), component.height()) {
            Some(Size2D {
                height: h.parse()?,
                width: w.parse()?,
            })
        } else {
            None
        };

        let x = component.x().parse::<f64>()?;
        let y = component.y().parse::<f64>()?;

        // TODO(nick): restore component status.
        // // Change status should track the component, not the node, since node position is on the
        // // node and the node will change if it is moved
        // let change_status = if component.visibility().deleted_at.is_some() {
        //     ChangeStatus::Deleted
        // } else if !component.exists_in_head(ctx).await? {
        //     ChangeStatus::Added
        // } else if is_modified {
        //     ChangeStatus::Modified
        // } else {
        //     ChangeStatus::Unmodified
        // };
        //
        // let component_status = ComponentStatus::get_by_id(ctx, component.id())
        //     .await?
        //     .ok_or_else(|| DiagramError::ComponentStatusNotFound(*component.id()))?;

        // TODO(nick): restore all of this.
        // let created_info =
        //     HistoryEventMetadata::from_history_actor_timestamp(ctx, component_status.creation())
        //         .await?;
        // let updated_info =
        //     HistoryEventMetadata::from_history_actor_timestamp(ctx, component_status.update())
        //         .await?;
        //
        // let mut deleted_info: Option<HistoryEventMetadata> = None;
        // {
        //     if let Some(deleted_at) = ctx.visibility().deleted_at {
        //         if let Some(deletion_user_pk) = component.deletion_user_pk() {
        //             let history_actor = history_event::HistoryActor::User(*deletion_user_pk);
        //             let actor = ActorView::from_history_actor(ctx, history_actor).await?;
        //
        //             deleted_info = Some(HistoryEventMetadata {
        //                 actor,
        //                 timestamp: deleted_at,
        //             });
        //         }
        //     }
        // }
        //
        // // TODO(theo): probably dont want to fetch this here and load totally separately, but we inherited from existing endpoints
        // let resource = ResourceView::new(component.resource(ctx).await?);
        //
        // let action_prototypes = ActionPrototype::find_for_context(
        //     ctx,
        //     ActionPrototypeContext {
        //         schema_variant_id: *schema_variant.id(),
        //     },
        // )
        // .await?;
        // let mut action_views: Vec<ActionPrototypeView> = Vec::new();
        // for action_prototype in action_prototypes {
        //     if *action_prototype.kind() == ActionKind::Refresh {
        //         continue;
        //     }
        //
        //     let view = ActionPrototypeView::new(ctx, action_prototype).await?;
        //     action_views.push(view);
        // }

        let schema = SchemaVariant::schema(ctx, schema_variant.id()).await?;

        // TODO(nick): replace these.
        let dummy_timestamp = HistoryActorTimestamp {
            actor: *ctx.history_actor(),
            timestamp: Utc::now(),
        };
        let dummy_metadata =
            HistoryEventMetadata::from_history_actor_timestamp(ctx, dummy_timestamp).await?;

        Ok(Self {
            id: component.id(),
            parent_component_id,
            child_component_ids,
            display_name: Some(component.name(ctx).await?),
            schema_name: schema.name().to_owned(),
            schema_variant_name: schema_variant.name().to_owned(),
            schema_id: schema.id().to_string(),
            schema_variant_id: schema_variant.id().to_string(),
            schema_category: Some(schema_variant.category().to_owned()),
            sockets: Some(DiagramSocketView::list(ctx, schema_variant.id()).await?),
            position: GridPoint {
                x: x.round() as isize,
                y: y.round() as isize,
            },
            size,
            color: component.color(ctx).await?,
            node_type: component.get_type(ctx).await?,
            change_status: ChangeStatus::Added,
            has_resource: false,
            created_info: dummy_metadata.clone(),
            updated_info: dummy_metadata,
            deleted_info: None,
        })
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }

    pub fn position(&self) -> &GridPoint {
        &self.position
    }

    pub fn size(&self) -> &Option<Size2D> {
        &self.size
    }
}
