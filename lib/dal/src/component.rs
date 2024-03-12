//! This module contains [`Component`], which is an instance of a
//! [`SchemaVariant`](crate::SchemaVariant) and a _model_ of a "real world resource".

use serde::{Deserialize, Serialize};
use std::collections::{hash_map, HashMap, HashSet, VecDeque};
use std::hash::Hash;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;
use ulid::Ulid;

use content_store::{ContentHash, Store, StoreError};

use crate::actor_view::ActorView;
use crate::attribute::prototype::argument::value_source::ValueSource;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::value::{AttributeValueError, DependentValueGraph};
use crate::change_set_pointer::ChangeSetPointerError;
use crate::code_view::CodeViewError;
use crate::history_event::HistoryEventMetadata;
use crate::layer_db_types::{ComponentContent, ComponentContentV1};
use crate::prop::{PropError, PropPath};
use crate::qualification::QualificationError;
use crate::schema::variant::root_prop::component_type::ComponentType;
use crate::schema::variant::SchemaVariantError;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::attribute_prototype_argument_node_weight::ArgumentTargets;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{ComponentNodeWeight, NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    func::backend::js_action::ActionRunResult, pk, ActionKind, ActionPrototype,
    ActionPrototypeError, AttributeValue, AttributeValueId, ChangeSetPk, DalContext, InputSocket,
    InputSocketId, OutputSocket, OutputSocketId, Prop, PropId, PropKind, Schema, SchemaVariant,
    SchemaVariantId, StandardModelError, Timestamp, TransactionsError, WsEvent, WsEventError,
    WsEventResult, WsPayload,
};

pub mod resource;

// pub mod code;
// pub mod diff;
mod code;
pub mod frame;
pub mod qualification;
// pub mod status;
// pub mod validation;
// pub mod view;

// pub use view::{ComponentView, ComponentViewError, ComponentViewProperties};

pub const DEFAULT_COMPONENT_X_POSITION: &str = "0";
pub const DEFAULT_COMPONENT_Y_POSITION: &str = "0";
pub const DEFAULT_COMPONENT_WIDTH: &str = "500";
pub const DEFAULT_COMPONENT_HEIGHT: &str = "500";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("code view error: {0}")]
    CodeView(#[from] CodeViewError),
    #[error("component {0} has no attribute value for the root/si/color prop")]
    ComponentMissingColorValue(ComponentId),
    #[error("component {0} has no attribute value for the root/si/name prop")]
    ComponentMissingNameValue(ComponentId),
    #[error("component {0} has no attribute value for the root/resource prop")]
    ComponentMissingResourceValue(ComponentId),
    #[error("component {0} has no attribute value for the root/si/type prop")]
    ComponentMissingTypeValue(ComponentId),
    #[error("connection destination component {0} has no attribute value for input socket {1}")]
    DestinationComponentMissingAttributeValueForInputSocket(ComponentId, InputSocketId),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("input socket {0} has more than one attribute value")]
    InputSocketTooManyAttributeValues(InputSocketId),
    #[error("component {0} missing attribute value for code")]
    MissingCodeValue(ComponentId),
    #[error("component {0} missing attribute value for qualifications")]
    MissingQualificationsValue(ComponentId),
    #[error("found multiple parents for component: {0}")]
    MultipleParentsForComponent(ComponentId),
    #[error("found multiple root attribute values ({0} and {1}, at minimum) for component: {2}")]
    MultipleRootAttributeValuesFound(AttributeValueId, AttributeValueId, ComponentId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("component not found: {0}")]
    NotFound(ComponentId),
    #[error("object prop {0} has no ordering node")]
    ObjectPropHasNoOrderingNode(PropId),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("output socket {0} has more than one attribute value")]
    OutputSocketTooManyAttributeValues(OutputSocketId),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("found prop id ({0}) that is not a prop")]
    PropIdNotAProp(PropId),
    #[error("qualification error: {0}")]
    Qualification(#[from] QualificationError),
    #[error("root attribute value not found for component: {0}")]
    RootAttributeValueNotFound(ComponentId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found for component: {0}")]
    SchemaVariantNotFound(ComponentId),
    #[error("serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("WsEvent error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

pk!(ComponentId);

#[derive(Clone, Debug)]
pub struct IncomingConnection {
    pub attribute_prototype_argument_id: AttributePrototypeArgumentId,
    pub to_component_id: ComponentId,
    pub to_input_socket_id: InputSocketId,
    pub from_component_id: ComponentId,
    pub from_output_socket_id: OutputSocketId,
    pub created_info: HistoryEventMetadata,
    pub deleted_info: Option<HistoryEventMetadata>,
}

/// A [`Component`] is an instantiation of a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Component {
    id: ComponentId,
    #[serde(flatten)]
    timestamp: Timestamp,
    to_delete: bool,
    x: String,
    y: String,
    width: Option<String>,
    height: Option<String>,
}

impl From<Component> for ComponentContentV1 {
    fn from(value: Component) -> Self {
        Self {
            timestamp: value.timestamp,
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

impl Component {
    pub fn assemble(node_weight: &ComponentNodeWeight, content: ComponentContentV1) -> Self {
        Self {
            id: node_weight.id().into(),
            timestamp: content.timestamp,
            to_delete: node_weight.to_delete(),
            x: content.x,
            y: content.y,
            width: content.width,
            height: content.height,
        }
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }

    pub fn x(&self) -> &str {
        &self.x
    }

    pub fn y(&self) -> &str {
        &self.y
    }

    pub fn width(&self) -> Option<&str> {
        self.width.as_deref()
    }

    pub fn height(&self) -> Option<&str> {
        self.height.as_deref()
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn to_delete(&self) -> bool {
        self.to_delete
    }

    pub async fn materialized_view(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Option<serde_json::Value>> {
        let schema_variant_id = Self::schema_variant_id(ctx, self.id()).await?;
        let root_prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(["root"])).await?;

        let root_value_ids = Prop::attribute_values_for_prop_id(ctx, root_prop_id).await?;
        for value_id in root_value_ids {
            let value_component_id = AttributeValue::component_id(ctx, value_id).await?;
            if value_component_id == self.id() {
                let root_value = AttributeValue::get_by_id(ctx, value_id).await?;
                return Ok(root_value.materialized_view(ctx).await?);
            }
        }

        // Should this be an error?
        Ok(None)
    }

    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        schema_variant_id: SchemaVariantId,
    ) -> ComponentResult<Self> {
        let name: String = name.into();

        let content = ComponentContentV1 {
            timestamp: Timestamp::now(),
            x: DEFAULT_COMPONENT_X_POSITION.to_string(),
            y: DEFAULT_COMPONENT_Y_POSITION.to_string(),
            width: None,
            height: None,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&ComponentContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_component(change_set, id, hash)?;

        // Attach component to category and add use edge to schema variant
        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.add_node(node_weight).await?;

        // Root --> Component Category --> Component (this)
        let component_category_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::Component)
            .await?;
        workspace_snapshot
            .add_edge(
                component_category_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                id,
            )
            .await?;

        // Component (this) --> Schema Variant
        workspace_snapshot
            .add_edge(
                id,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                schema_variant_id,
            )
            .await?;

        let mut attribute_values = vec![];

        // Create attribute values for all socket corresponding to input and output sockets.
        for input_socket_id in
            InputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?
        {
            let attribute_value =
                AttributeValue::new(ctx, input_socket_id, Some(id.into()), None, None).await?;

            attribute_values.push(attribute_value.id());
        }
        for output_socket_id in
            OutputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?
        {
            let attribute_value =
                AttributeValue::new(ctx, output_socket_id, Some(id.into()), None, None).await?;

            attribute_values.push(attribute_value.id());
        }

        // Walk all the props for the schema variant and create attribute values for all of them
        let root_prop_id = SchemaVariant::get_root_prop_id(ctx, schema_variant_id).await?;
        let mut work_queue = VecDeque::from([(root_prop_id, None::<AttributeValueId>, None)]);

        while let Some((prop_id, maybe_parent_attribute_value_id, key)) = work_queue.pop_front() {
            // If we came in with a key, we're the child of a map. We should not descend deeper
            // into it because the value should be governed by its prototype function and will
            // create child values when that function is executed
            let should_descend = key.is_none();

            // Ensure that we are processing a prop before creating attribute values. Cache the
            // prop kind for later.
            let prop_kind = workspace_snapshot
                .get_node_weight_by_id(prop_id)
                .await?
                .get_prop_node_weight()?
                .kind();

            // Create an attribute value for the prop.
            let attribute_value = AttributeValue::new(
                ctx,
                prop_id,
                Some(id.into()),
                maybe_parent_attribute_value_id,
                key,
            )
            .await?;

            attribute_values.push(attribute_value.id());

            if should_descend {
                match prop_kind {
                    PropKind::Object => {
                        let ordering_node_weight = workspace_snapshot
                            .ordering_node_for_container(prop_id)
                            .await?
                            .ok_or(ComponentError::ObjectPropHasNoOrderingNode(prop_id))?;

                        for &child_prop_id in ordering_node_weight.order() {
                            work_queue.push_back((
                                child_prop_id.into(),
                                Some(attribute_value.id()),
                                None,
                            ));
                        }
                    }
                    PropKind::Map => {
                        let element_prop_id =
                            Prop::direct_single_child_prop_id(ctx, prop_id).await?;

                        for (key, _) in Prop::prototypes_by_key(ctx, element_prop_id).await? {
                            if key.is_some() {
                                work_queue.push_back((
                                    element_prop_id,
                                    Some(attribute_value.id()),
                                    key,
                                ))
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let (node_weight, content) = Self::get_node_weight_and_content(ctx, id.into()).await?;
        let component = Self::assemble(&node_weight, content);

        component.set_name(ctx, &name).await?;

        let component_graph = DependentValueGraph::for_values(ctx, attribute_values).await?;
        let leaf_value_ids = component_graph.independent_values();
        for leaf_value_id in &leaf_value_ids {
            // Run these concurrently in a join set? They will serialize on the lock...
            AttributeValue::update_from_prototype_function(ctx, *leaf_value_id).await?;
        }
        ctx.enqueue_dependent_values_update(leaf_value_ids).await?;

        Ok(component)
    }

    pub async fn incoming_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<IncomingConnection>> {
        let mut incoming_edges = vec![];

        for (to_input_socket_id, to_value_id) in self.input_socket_attribute_values(ctx).await? {
            let prototype_id = AttributeValue::prototype_id(ctx, to_value_id).await?;
            for apa_id in AttributePrototypeArgument::list_ids_for_prototype_and_destination(
                ctx,
                prototype_id,
                self.id,
            )
            .await?
            {
                let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;

                let created_info = {
                    let history_actor = ctx.history_actor();
                    let actor = ActorView::from_history_actor(ctx, *history_actor).await?;
                    HistoryEventMetadata {
                        actor,
                        timestamp: apa.timestamp().created_at,
                    }
                };

                if let Some(ArgumentTargets {
                    source_component_id,
                    destination_component_id,
                }) = apa.targets()
                {
                    if let Some(ValueSource::OutputSocket(from_output_socket_id)) =
                        apa.value_source(ctx).await?
                    {
                        incoming_edges.push(IncomingConnection {
                            attribute_prototype_argument_id: apa_id,
                            to_component_id: destination_component_id,
                            from_component_id: source_component_id,
                            to_input_socket_id,
                            from_output_socket_id,
                            created_info,
                            deleted_info: None,
                        });
                    }
                }
            }
        }

        Ok(incoming_edges)
    }

    pub async fn parent(&self, ctx: &DalContext) -> ComponentResult<Option<ComponentId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut raw_sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                self.id,
                EdgeWeightKindDiscriminants::FrameContains,
            )
            .await?;

        let maybe_parent = if let Some(raw_parent) = raw_sources.pop() {
            if !raw_sources.is_empty() {
                return Err(ComponentError::MultipleParentsForComponent(self.id));
            }
            Some(
                workspace_snapshot
                    .get_node_weight(raw_parent)
                    .await?
                    .id()
                    .into(),
            )
        } else {
            None
        };

        Ok(maybe_parent)
    }

    async fn get_node_weight_and_content(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(ComponentNodeWeight, ComponentContentV1)> {
        let (component_node_weight, hash) =
            Self::get_node_weight_and_content_hash(ctx, component_id).await?;

        let content: ComponentContent = ctx.content_store().lock().await.get(&hash).await?.ok_or(
            WorkspaceSnapshotError::MissingContentFromStore(component_id.into()),
        )?;

        let ComponentContent::V1(inner) = content;

        Ok((component_node_weight, inner))
    }

    async fn get_node_weight_and_content_hash(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(ComponentNodeWeight, ContentHash)> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id: Ulid = component_id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(id).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;

        let hash = node_weight.content_hash();
        let component_node_weight = node_weight.get_component_node_weight()?;
        Ok((component_node_weight, hash))
    }

    pub async fn list(ctx: &DalContext) -> ComponentResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut components = vec![];
        let component_category_node_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::Component)
            .await?;

        let component_node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                component_category_node_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut node_weights = vec![];
        let mut hashes = vec![];
        for index in component_node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_component_node_weight()?;
            hashes.push(node_weight.content_hash());
            node_weights.push(node_weight);
        }

        let contents: HashMap<ContentHash, ComponentContent> = ctx
            .content_store()
            .lock()
            .await
            .get_bulk(hashes.as_slice())
            .await?;

        for node_weight in node_weights {
            match contents.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let ComponentContent::V1(inner) = content;

                    components.push(Self::assemble(&node_weight, inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(components)
    }

    pub async fn schema_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Schema> {
        let schema_variant = Self::schema_variant_for_component_id(ctx, component_id).await?;

        Ok(schema_variant.schema(ctx).await?)
    }

    pub async fn schema(&self, ctx: &DalContext) -> ComponentResult<Schema> {
        Self::schema_for_component_id(ctx, self.id).await
    }

    pub async fn schema_variant_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<SchemaVariant> {
        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;
        Ok(SchemaVariant::get_by_id(ctx, schema_variant_id).await?)
    }

    pub async fn schema_variant(&self, ctx: &DalContext) -> ComponentResult<SchemaVariant> {
        Self::schema_variant_for_component_id(ctx, self.id).await
    }

    pub async fn schema_variant_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<SchemaVariantId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let maybe_schema_variant_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(component_id, EdgeWeightKindDiscriminants::Use)
            .await?;

        let mut schema_variant_id: Option<SchemaVariantId> = None;
        for maybe_schema_variant_index in maybe_schema_variant_indices {
            if let NodeWeight::Content(content) = workspace_snapshot
                .get_node_weight(maybe_schema_variant_index)
                .await?
            {
                let content_hash_discriminants: ContentAddressDiscriminants =
                    content.content_address().into();
                if let ContentAddressDiscriminants::SchemaVariant = content_hash_discriminants {
                    // TODO(nick): consider creating a new edge weight kind to make this easier.
                    // We also should use a proper error here.
                    schema_variant_id = match schema_variant_id {
                        None => Some(content.id().into()),
                        Some(_already_found_schema_variant_id) => {
                            panic!("already found a schema variant")
                        }
                    };
                }
            }
        }
        let schema_variant_id =
            schema_variant_id.ok_or(ComponentError::SchemaVariantNotFound(component_id))?;
        Ok(schema_variant_id)
    }

    pub async fn get_by_id(ctx: &DalContext, component_id: ComponentId) -> ComponentResult<Self> {
        let (node_weight, content) = Self::get_node_weight_and_content(ctx, component_id).await?;
        Ok(Self::assemble(&node_weight, content))
    }

    pub async fn set_geometry(
        self,
        ctx: &DalContext,
        x: impl Into<String>,
        y: impl Into<String>,
        width: Option<impl Into<String>>,
        height: Option<impl Into<String>>,
    ) -> ComponentResult<Self> {
        let id: ComponentId = self.id;
        let mut component = self;

        let before = ComponentContentV1::from(component.clone());
        component.x = x.into();
        component.y = y.into();
        component.width = width.map(|w| w.into());
        component.height = height.map(|h| h.into());
        let updated = ComponentContentV1::from(component);

        if updated != before {
            let hash = ctx
                .content_store()
                .lock()
                .await
                .add(&ComponentContent::V1(updated))?;

            ctx.workspace_snapshot()?
                .update_content(ctx.change_set_pointer()?, id.into(), hash)
                .await?;
        }
        let (node_weight, content) = Self::get_node_weight_and_content(ctx, id).await?;

        Ok(Self::assemble(&node_weight, content))
    }

    async fn set_name(&self, ctx: &DalContext, name: &str) -> ComponentResult<()> {
        let av_for_name = self
            .attribute_values_for_prop(ctx, &["root", "si", "name"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingNameValue(self.id()))?;

        AttributeValue::update_no_dependent_values(
            ctx,
            av_for_name,
            Some(serde_json::to_value(name)?),
        )
        .await?;

        Ok(())
    }

    pub async fn act(&self, ctx: &DalContext, action: ActionKind) -> ComponentResult<()> {
        let schema_variant = self.schema_variant(ctx).await?;

        let action = ActionPrototype::for_variant(ctx, schema_variant.id())
            .await
            .map_err(Box::new)?
            .into_iter()
            .find(|p| p.kind == action);
        if let Some(action) = action {
            action.run(ctx, self.id()).await.map_err(Box::new)?;
        }

        Ok(())
    }

    pub async fn set_resource(
        &self,
        ctx: &DalContext,
        resource: ActionRunResult,
    ) -> ComponentResult<()> {
        let av_for_resource = self
            .attribute_values_for_prop(ctx, &["root", "resource"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingResourceValue(self.id()))?;

        AttributeValue::update(ctx, av_for_resource, Some(serde_json::to_value(resource)?)).await?;

        Ok(())
    }

    pub async fn resource(&self, ctx: &DalContext) -> ComponentResult<ActionRunResult> {
        let value_id = self
            .attribute_values_for_prop(ctx, &["root", "resource"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingResourceValue(self.id()))?;

        let av = AttributeValue::get_by_id(ctx, value_id).await?;

        Ok(match av.materialized_view(ctx).await? {
            Some(serde_value) => serde_json::from_value(serde_value)?,
            None => ActionRunResult::default(),
        })
    }

    pub async fn name(&self, ctx: &DalContext) -> ComponentResult<String> {
        let name_value_id = self
            .attribute_values_for_prop(ctx, &["root", "si", "name"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingNameValue(self.id()))?;

        let name_av = AttributeValue::get_by_id(ctx, name_value_id).await?;

        Ok(match name_av.materialized_view(ctx).await? {
            Some(serde_value) => serde_json::from_value(serde_value)?,
            None => "".into(),
        })
    }

    pub async fn color(&self, ctx: &DalContext) -> ComponentResult<Option<String>> {
        let color_value_id = self
            .attribute_values_for_prop(ctx, &["root", "si", "color"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingColorValue(self.id()))?;

        let color_av = AttributeValue::get_by_id(ctx, color_value_id).await?;

        Ok(match color_av.materialized_view(ctx).await? {
            Some(serde_value) => Some(serde_json::from_value(serde_value)?),
            None => None,
        })
    }

    pub async fn get_type(&self, ctx: &DalContext) -> ComponentResult<ComponentType> {
        let type_value_id = self
            .attribute_values_for_prop(ctx, &["root", "si", "type"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingTypeValue(self.id()))?;

        let type_value = AttributeValue::get_by_id(ctx, type_value_id)
            .await?
            .materialized_view(ctx)
            .await?
            .ok_or(ComponentError::ComponentMissingTypeValue(self.id()))?;

        Ok(serde_json::from_value(type_value)?)
    }

    pub async fn set_type(&self, ctx: &DalContext, new_type: ComponentType) -> ComponentResult<()> {
        let type_value_id = self
            .attribute_values_for_prop(ctx, &["root", "si", "type"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingTypeValue(self.id()))?;

        let value = serde_json::to_value(new_type)?;

        AttributeValue::update(ctx, type_value_id, Some(value)).await?;

        Ok(())
    }

    pub async fn root_attribute_value_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<AttributeValueId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut maybe_root_attribute_value_id = None;
        for target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(component_id, EdgeWeightKindDiscriminants::Root)
            .await?
        {
            let target_node_weight = workspace_snapshot.get_node_weight(target).await?;
            if let NodeWeight::AttributeValue(_) = target_node_weight {
                maybe_root_attribute_value_id = match maybe_root_attribute_value_id {
                    Some(already_found_root_attribute_value_id) => {
                        return Err(ComponentError::MultipleRootAttributeValuesFound(
                            target_node_weight.id().into(),
                            already_found_root_attribute_value_id,
                            component_id,
                        ));
                    }
                    None => Some(target_node_weight.id().into()),
                };
            }
        }
        maybe_root_attribute_value_id
            .ok_or(ComponentError::RootAttributeValueNotFound(component_id))
    }

    pub async fn output_socket_attribute_values_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<HashMap<OutputSocketId, AttributeValueId>> {
        let mut result = HashMap::new();

        let socket_values = Self::values_for_all_sockets(ctx, component_id).await?;

        for socket_value_id in socket_values {
            if let Some(output_socket_id) = AttributeValue::is_for(ctx, socket_value_id)
                .await?
                .output_socket_id()
            {
                match result.entry(output_socket_id) {
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(socket_value_id);
                    }
                    hash_map::Entry::Occupied(_) => {
                        return Err(ComponentError::OutputSocketTooManyAttributeValues(
                            output_socket_id,
                        ));
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn output_socket_attribute_values(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<HashMap<OutputSocketId, AttributeValueId>> {
        Self::output_socket_attribute_values_for_component_id(ctx, self.id()).await
    }

    pub async fn attribute_values_for_prop(
        &self,
        ctx: &DalContext,
        prop_path: &[&str],
    ) -> ComponentResult<Vec<AttributeValueId>> {
        let mut result = vec![];

        let schema_variant_id = Self::schema_variant_id(ctx, self.id()).await?;

        let prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(prop_path)).await?;

        for attribute_value_id in Prop::attribute_values_for_prop_id(ctx, prop_id).await? {
            let value_component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
            if value_component_id == self.id() {
                result.push(attribute_value_id)
            }
        }

        Ok(result)
    }

    async fn values_for_all_sockets(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<AttributeValueId>> {
        let mut socket_values: Vec<AttributeValueId> = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;

        for socket_target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                component_id,
                EdgeWeightKindDiscriminants::SocketValue,
            )
            .await?
        {
            socket_values.push(
                workspace_snapshot
                    .get_node_weight(socket_target)
                    .await?
                    .get_attribute_value_node_weight()?
                    .id()
                    .into(),
            );
        }

        Ok(socket_values)
    }

    pub async fn input_socket_attribute_values_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<HashMap<InputSocketId, AttributeValueId>> {
        let mut result = HashMap::new();

        let socket_values = Self::values_for_all_sockets(ctx, component_id).await?;

        for socket_value_id in socket_values {
            if let Some(input_socket_id) = AttributeValue::is_for(ctx, socket_value_id)
                .await?
                .input_socket_id()
            {
                match result.entry(input_socket_id) {
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(socket_value_id);
                    }
                    hash_map::Entry::Occupied(_) => {
                        return Err(ComponentError::InputSocketTooManyAttributeValues(
                            input_socket_id,
                        ));
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn input_socket_attribute_values(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<HashMap<InputSocketId, AttributeValueId>> {
        Self::input_socket_attribute_values_for_component_id(ctx, self.id()).await
    }

    async fn connect_inner(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_it: OutputSocketId,
        destination_component_id: ComponentId,
        destination_input_socket_id: InputSocketId,
    ) -> ComponentResult<(AttributeValueId, AttributePrototypeArgumentId)> {
        let destination_attribute_value_ids =
            InputSocket::attribute_values_for_input_socket_id(ctx, destination_input_socket_id)
                .await?;

        // filter the value ids by destination_component_id
        let mut destination_attribute_value_id: Option<AttributeValueId> = None;
        for value_id in destination_attribute_value_ids {
            let component_id = AttributeValue::component_id(ctx, value_id).await?;
            if component_id == destination_component_id {
                destination_attribute_value_id = Some(value_id);
                break;
            }
        }

        let destination_attribute_value_id = destination_attribute_value_id.ok_or(
            ComponentError::DestinationComponentMissingAttributeValueForInputSocket(
                destination_component_id,
                destination_input_socket_id,
            ),
        )?;

        let destination_prototype_id =
            AttributeValue::prototype_id(ctx, destination_attribute_value_id).await?;

        let attribute_prototype_argument = AttributePrototypeArgument::new_inter_component(
            ctx,
            source_component_id,
            source_output_socket_it,
            destination_component_id,
            destination_prototype_id,
        )
        .await?;

        AttributeValue::update_from_prototype_function(ctx, destination_attribute_value_id).await?;

        Ok((
            destination_attribute_value_id,
            attribute_prototype_argument.id(),
        ))
    }

    pub async fn connect(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_id: OutputSocketId,
        destination_component_id: ComponentId,
        destination_input_socket_id: InputSocketId,
    ) -> ComponentResult<AttributePrototypeArgumentId> {
        let (destination_attribute_value_id, attribute_prototype_argument_id) =
            Self::connect_inner(
                ctx,
                source_component_id,
                source_output_socket_id,
                destination_component_id,
                destination_input_socket_id,
            )
            .await?;

        ctx.enqueue_dependent_values_update(vec![destination_attribute_value_id])
            .await?;

        Ok(attribute_prototype_argument_id)
    }

    /// Find all matching sockets for a given source [`Component`] and a given destination [`Component`].
    ///
    /// This is useful when [`attaching`](frame::Frame::attach_child_to_parent) a child [`Component`] to a parent
    /// frame.
    pub async fn connect_all(
        ctx: &DalContext,
        source_component_id: ComponentId,
        destination_component_id: ComponentId,
    ) -> ComponentResult<()> {
        let source_schema_variant_id =
            Component::schema_variant_id(ctx, source_component_id).await?;
        let destination_schema_variant_id =
            Component::schema_variant_id(ctx, destination_component_id).await?;

        let source_sockets = OutputSocket::list(ctx, source_schema_variant_id).await?;
        let destination_sockets = InputSocket::list(ctx, destination_schema_variant_id).await?;

        let mut to_enqueue = Vec::new();

        for src_sock in source_sockets {
            let mut maybe_dest_id = None;
            for dest_candidate in &destination_sockets {
                if src_sock.fits_input(dest_candidate) {
                    // If more than one valid destination is found, skip the socket.
                    if maybe_dest_id.is_some() && maybe_dest_id != Some(dest_candidate.id()) {
                        maybe_dest_id = None;
                        break;
                    }

                    // Otherwise, this is a socket we wanna connect to!
                    maybe_dest_id = Some(dest_candidate.id());
                }
            }

            if let Some(destination_socket_id) = maybe_dest_id {
                let (attribute_value_id, _) = Self::connect_inner(
                    ctx,
                    source_component_id,
                    src_sock.id(),
                    destination_component_id,
                    destination_socket_id,
                )
                .await?;
                to_enqueue.push(attribute_value_id);
            }
        }

        // Enqueue all the values from each connection.
        ctx.enqueue_dependent_values_update(to_enqueue).await?;

        Ok(())
    }

    // Returns map of node id -> parent node ids
    pub async fn build_graph(
        ctx: &DalContext,
    ) -> ComponentResult<HashMap<ComponentId, HashSet<ComponentId>>> {
        let total_start = std::time::Instant::now();

        let components = Self::list(ctx).await?;

        let mut components_map: HashMap<ComponentId, HashSet<ComponentId>> = HashMap::new();

        for component in components {
            components_map.insert(component.id, HashSet::new());

            for incomming_connection in component.incoming_connections(ctx).await? {
                components_map
                    .entry(component.id)
                    .or_default()
                    .insert(incomming_connection.from_component_id);
            }
        }

        debug!("build graph took {:?}", total_start.elapsed());
        Ok(components_map)
    }

    async fn modify<L>(self, ctx: &DalContext, lambda: L) -> ComponentResult<Self>
    where
        L: FnOnce(&mut Self) -> ComponentResult<()>,
    {
        let mut component = self;

        let before = ComponentContentV1::from(component.clone());
        lambda(&mut component)?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let component_idx = workspace_snapshot
            .get_node_index_by_id(component.id())
            .await?;
        let component_node_weight = workspace_snapshot
            .get_node_weight(component_idx)
            .await?
            .get_component_node_weight()?;

        // The `to_delete` lives on the node itself, not in the content, so we need to be a little
        // more manual when updating that field.
        if component.to_delete != component_node_weight.to_delete() {
            let mut new_component_node_weight = component_node_weight
                .new_with_incremented_vector_clock(ctx.change_set_pointer()?)?;
            new_component_node_weight.set_to_delete(component.to_delete);
            workspace_snapshot
                .add_node(NodeWeight::Component(new_component_node_weight))
                .await?;
            workspace_snapshot.replace_references(component_idx).await?;
        }

        let updated = ComponentContentV1::from(component.clone());
        if updated != before {
            let hash = ctx
                .content_store()
                .lock()
                .await
                .add(&ComponentContent::V1(updated.clone()))?;
            workspace_snapshot
                .update_content(ctx.change_set_pointer()?, component.id.into(), hash)
                .await?;
        }

        Ok(Component::assemble(&component_node_weight, updated))
    }

    pub async fn delete(self, ctx: &DalContext) -> ComponentResult<Self> {
        self.modify(ctx, |component| {
            component.to_delete = true;
            Ok(())
        })
        .await

        // TODO: Trigger DependentValuesUpdate for all Components that get data from this
        // Component.
    }

    pub async fn set_to_delete(self, ctx: &DalContext, to_delete: bool) -> ComponentResult<Self> {
        self.modify(ctx, |component| {
            component.to_delete = to_delete;
            Ok(())
        })
        .await
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentCreatedPayload {
    success: bool,
    component_id: ComponentId,
    change_set_pk: ChangeSetPk,
}

impl WsEvent {
    pub async fn component_created(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ComponentCreated(ComponentCreatedPayload {
                success: true,
                change_set_pk: ctx.visibility().change_set_pk,
                component_id,
            }),
        )
        .await
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentUpdatedPayload {
    component_id: ComponentId,
    change_set_pk: ChangeSetPk,
}

impl WsEvent {
    pub async fn component_updated(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ComponentUpdated(ComponentUpdatedPayload {
                component_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }
}
