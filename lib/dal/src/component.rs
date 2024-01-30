//! This module contains [`Component`], which is an instance of a
//! [`SchemaVariant`](crate::SchemaVariant) and a _model_ of a "real world resource".

use content_store::{ContentHash, Store, StoreError};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map, HashMap, VecDeque};
use std::hash::Hash;
use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString, IntoEnumIterator};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;
use ulid::Ulid;

use crate::attribute::prototype::argument::value_source::ValueSource;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::value::{AttributeValueError, DependentValueGraph};
use crate::change_set_pointer::ChangeSetPointerError;
use crate::job::definition::DependentValuesUpdate;
use crate::prop::{PropError, PropPath};
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::root_prop::{component_type::ComponentType, RootPropChild};
use crate::schema::variant::SchemaVariantError;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::attribute_prototype_argument_node_weight::ArgumentTargets;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, AttributeValue, AttributeValueId, DalContext, ExternalProvider, ExternalProviderId,
    InternalProvider, InternalProviderId, Prop, PropId, PropKind, SchemaVariant, SchemaVariantId,
    Timestamp, TransactionsError, WsEvent, WsEventResult, WsPayload,
};

pub mod resource;

// pub mod code;
// pub mod diff;
// pub mod qualification;
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
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("component {0} has no attribute value for the root/si/color prop")]
    ComponentMissingColorValue(ComponentId),
    #[error("component {0} has no attribute value for the root/si/name prop")]
    ComponentMissingNameValue(ComponentId),
    #[error("component {0} has no attribute value for the root/si/type prop")]
    ComponentMissingTypeValue(ComponentId),
    #[error(
        "connection destination component {0} has no attribute value for internal provider {1}"
    )]
    DestinationComponentMissingAttributeValueForInternalProvider(ComponentId, InternalProviderId),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("external provider {0} has more than one attribute value")]
    ExternalProviderTooManyAttributeValues(ExternalProviderId),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("internal provider {0} has more than one attribute value")]
    InternalProviderTooManyAttributeValues(InternalProviderId),
    #[error("found multiple root attribute values ({0} and {1}, at minimum) for component: {2}")]
    MultipleRootAttributeValuesFound(AttributeValueId, AttributeValueId, ComponentId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("found prop id ({0}) that is not a prop")]
    PropIdNotAProp(PropId),
    #[error("root attribute value not found for component: {0}")]
    RootAttributeValueNotFound(ComponentId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found for component: {0}")]
    SchemaVariantNotFound(ComponentId),
    #[error("serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

pk!(ComponentId);

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
pub enum ComponentKind {
    Credential,
    Standard,
}

impl Default for ComponentKind {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Clone, Debug)]
pub struct IncomingConnection {
    pub attribute_prototype_argument_id: AttributePrototypeArgumentId,
    pub to_component_id: ComponentId,
    pub to_internal_provider_id: InternalProviderId,
    pub from_component_id: ComponentId,
    pub from_external_provider_id: ExternalProviderId,
}

/// A [`Component`] is an instantiation of a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Component {
    id: ComponentId,
    #[serde(flatten)]
    timestamp: Timestamp,
    kind: ComponentKind,
    needs_destroy: bool,
    x: String,
    y: String,
    width: Option<String>,
    height: Option<String>,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum ComponentContent {
    V1(ComponentContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ComponentContentV1 {
    pub timestamp: Timestamp,
    pub kind: ComponentKind,
    pub needs_destroy: bool,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
}

impl From<Component> for ComponentContentV1 {
    fn from(value: Component) -> Self {
        Self {
            timestamp: value.timestamp,
            kind: value.kind,
            needs_destroy: value.needs_destroy,
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

impl Component {
    pub fn assemble(id: ComponentId, inner: ComponentContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            kind: inner.kind,
            needs_destroy: inner.needs_destroy,
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height,
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
        component_kind: Option<ComponentKind>,
    ) -> ComponentResult<Self> {
        let name: String = name.into();
        let kind = match component_kind {
            Some(provided_kind) => provided_kind,
            None => ComponentKind::Standard,
        };

        let content = ComponentContentV1 {
            kind,
            timestamp: Timestamp::now(),
            needs_destroy: false,
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
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Component(hash))?;

        // Attach component to category and add use edge to schema variant
        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
            workspace_snapshot.add_node(node_weight)?;

            // Root --> Component Category --> Component (this)
            let component_category_id =
                workspace_snapshot.get_category_node(None, CategoryNodeKind::Component)?;
            workspace_snapshot.add_edge(
                component_category_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                id,
            )?;

            // Component (this) --> Schema Variant
            workspace_snapshot.add_edge(
                id,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                schema_variant_id,
            )?;
        }

        let mut attribute_values = vec![];

        // Create attribute values for all providers corresponding to input and output sockets.
        for internal_provider_id in
            InternalProvider::list_ids_for_schema_variant(ctx, schema_variant_id).await?
        {
            let attribute_value =
                AttributeValue::new(ctx, internal_provider_id, Some(id.into()), None, None).await?;

            attribute_values.push(attribute_value.id());
        }
        for external_provider_id in
            ExternalProvider::list_ids_for_schema_variant(ctx, schema_variant_id).await?
        {
            let attribute_value =
                AttributeValue::new(ctx, external_provider_id, Some(id.into()), None, None).await?;

            attribute_values.push(attribute_value.id());
        }

        // Walk all the props for the schema variant and create attribute values for all of them
        let root_prop_id = SchemaVariant::get_root_prop_id(ctx, schema_variant_id).await?;
        let mut work_queue = VecDeque::from([(root_prop_id, None::<AttributeValueId>, None)]);

        while let Some((prop_id, maybe_parent_attribute_value_id, key)) = work_queue.pop_front() {
            // Ensure that we are processing a prop before creating attribute values. Cache the
            // prop kind for later.
            let prop_kind = {
                let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

                workspace_snapshot
                    .get_node_weight_by_id(prop_id)?
                    .get_prop_node_weight()?
                    .kind()
            };

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

            match prop_kind {
                PropKind::Object => {
                    let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

                    if let Some(ordering_node_idx) = workspace_snapshot
                        .outgoing_targets_for_edge_weight_kind(
                            prop_id,
                            EdgeWeightKindDiscriminants::Ordering,
                        )?
                        .get(0)
                    {
                        let ordering_node_weight = workspace_snapshot
                            .get_node_weight(*ordering_node_idx)?
                            .get_ordering_node_weight()?;

                        for &child_prop_id in ordering_node_weight.order() {
                            work_queue.push_back((
                                child_prop_id.into(),
                                Some(attribute_value.id()),
                                None,
                            ));
                        }
                    } else {
                        // TODO(nick): address this better.
                        unreachable!("object props must have ordering nodes")
                    }
                }
                PropKind::Map => {
                    //
                }
                _ => {}
            }
        }

        let component = Self::assemble(id.into(), content);

        component.set_name(ctx, &name).await?;

        let component_graph = DependentValueGraph::for_values(ctx, attribute_values).await?;
        let leaf_value_ids = component_graph.independent_values();
        for leaf_value_id in &leaf_value_ids {
            AttributeValue::update_from_prototype_function(ctx, *leaf_value_id).await?;
        }
        ctx.enqueue_job(DependentValuesUpdate::new(
            ctx.access_builder(),
            *ctx.visibility(),
            leaf_value_ids,
        ))
        .await?;

        Ok(component)
    }

    pub async fn incoming_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<IncomingConnection>> {
        let mut incoming_edges = vec![];
        for (to_internal_provider_id, to_value_id) in
            self.internal_provider_attribute_values(ctx).await?
        {
            let prototype_id = AttributeValue::prototype_id(ctx, to_value_id).await?;
            for apa_id in
                AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id).await?
            {
                let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;
                if let Some(ArgumentTargets {
                    source_component_id,
                    ..
                }) = apa.targets()
                {
                    if let Some(ValueSource::ExternalProvider(from_external_provider_id)) =
                        apa.value_source(ctx).await?
                    {
                        incoming_edges.push(IncomingConnection {
                            attribute_prototype_argument_id: apa_id,
                            to_component_id: self.id(),
                            from_component_id: source_component_id,
                            to_internal_provider_id,
                            from_external_provider_id,
                        });
                    }
                }
            }
        }

        Ok(incoming_edges)
    }

    async fn get_content_with_hash(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(ContentHash, ComponentContentV1)> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
        let id: Ulid = component_id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(id)?;
        let node_weight = workspace_snapshot.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: ComponentContent = ctx
            .content_store()
            .lock()
            .await
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let ComponentContent::V1(inner) = content;

        Ok((hash, inner))
    }

    pub async fn list(ctx: &DalContext) -> ComponentResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let mut components = vec![];
        let component_category_node_id =
            workspace_snapshot.get_category_node(None, CategoryNodeKind::Component)?;

        let component_node_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            component_category_node_id,
            EdgeWeightKindDiscriminants::Use,
        )?;

        let mut node_weights = vec![];
        let mut hashes = vec![];
        for index in component_node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::Component)?;
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

                    components.push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(components)
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
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let maybe_schema_variant_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                component_id,
                EdgeWeightKindDiscriminants::Use,
            )?;

        let mut schema_variant_id: Option<SchemaVariantId> = None;
        for maybe_schema_variant_index in maybe_schema_variant_indices {
            if let NodeWeight::Content(content) =
                workspace_snapshot.get_node_weight(maybe_schema_variant_index)?
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
        let (_, content) = Self::get_content_with_hash(ctx, component_id).await?;
        Ok(Self::assemble(component_id, content))
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
                .add(&ComponentContent::V1(updated.clone()))?;

            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
            workspace_snapshot.update_content(ctx.change_set_pointer()?, id.into(), hash)?;
        }

        Ok(Self::assemble(id, updated))
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

    async fn set_color(&self, ctx: &DalContext, color: &str) -> ComponentResult<()> {
        let av_for_color = self
            .attribute_values_for_prop(ctx, &["root", "si", "color"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingNameValue(self.id()))?;

        AttributeValue::update_no_dependent_values(
            ctx,
            av_for_color,
            Some(serde_json::to_value(color)?),
        )
        .await?;

        Ok(())
    }

    async fn set_type(
        &self,
        ctx: &DalContext,
        component_type: ComponentType,
    ) -> ComponentResult<()> {
        let av_for_type = self
            .attribute_values_for_prop(ctx, &["root", "si", "type"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingNameValue(self.id()))?;

        AttributeValue::update_no_dependent_values(
            ctx,
            av_for_type,
            Some(serde_json::to_value(component_type.to_string())?),
        )
        .await?;

        Ok(())
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

    pub async fn root_attribute_value_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<AttributeValueId> {
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        let mut maybe_root_attribute_value_id = None;
        for target in workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            component_id,
            EdgeWeightKindDiscriminants::Root,
        )? {
            let target_node_weight = workspace_snapshot.get_node_weight(target)?;
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

    pub async fn external_provider_attribute_values_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<HashMap<ExternalProviderId, AttributeValueId>> {
        let mut result = HashMap::new();

        let socket_values = Self::values_for_all_providers(ctx, component_id).await?;

        for socket_value_id in socket_values {
            if let Some(external_provider_id) = AttributeValue::is_for(ctx, socket_value_id)
                .await?
                .external_provider_id()
            {
                match result.entry(external_provider_id) {
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(socket_value_id);
                    }
                    hash_map::Entry::Occupied(_) => {
                        return Err(ComponentError::ExternalProviderTooManyAttributeValues(
                            external_provider_id,
                        ))
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn external_provider_attribute_values(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<HashMap<ExternalProviderId, AttributeValueId>> {
        Self::external_provider_attribute_values_for_component_id(ctx, self.id()).await
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

    async fn values_for_all_providers(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<AttributeValueId>> {
        let mut socket_values: Vec<AttributeValueId> = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

        for socket_target in workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            component_id,
            EdgeWeightKindDiscriminants::Socket,
        )? {
            socket_values.push(
                workspace_snapshot
                    .get_node_weight(socket_target)?
                    .get_attribute_value_node_weight()?
                    .id()
                    .into(),
            );
        }

        Ok(socket_values)
    }

    pub async fn internal_provider_attribute_values_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<HashMap<InternalProviderId, AttributeValueId>> {
        let mut result = HashMap::new();

        let socket_values = Self::values_for_all_providers(ctx, component_id).await?;

        for socket_value_id in socket_values {
            if let Some(internal_provider_id) = AttributeValue::is_for(ctx, socket_value_id)
                .await?
                .internal_provider_id()
            {
                match result.entry(internal_provider_id) {
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(socket_value_id);
                    }
                    hash_map::Entry::Occupied(_) => {
                        return Err(ComponentError::InternalProviderTooManyAttributeValues(
                            internal_provider_id,
                        ))
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn internal_provider_attribute_values(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<HashMap<InternalProviderId, AttributeValueId>> {
        Self::internal_provider_attribute_values_for_component_id(ctx, self.id()).await
    }

    pub async fn connect(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_external_provider_id: ExternalProviderId,
        destination_component_id: ComponentId,
        destination_explicit_internal_provider_id: InternalProviderId,
    ) -> ComponentResult<AttributePrototypeArgumentId> {
        let destination_attribute_value_ids =
            InternalProvider::attribute_values_for_internal_provider_id(
                ctx,
                destination_explicit_internal_provider_id,
            )
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
            ComponentError::DestinationComponentMissingAttributeValueForInternalProvider(
                destination_component_id,
                destination_explicit_internal_provider_id,
            ),
        )?;

        let destination_prototype_id =
            AttributeValue::prototype_id(ctx, destination_attribute_value_id).await?;

        let attribute_prototype_argument = AttributePrototypeArgument::new_inter_component(
            ctx,
            source_component_id,
            source_external_provider_id,
            destination_component_id,
            destination_prototype_id,
        )
        .await?;

        AttributeValue::update_from_prototype_function(ctx, destination_attribute_value_id).await?;

        ctx.enqueue_job(DependentValuesUpdate::new(
            ctx.access_builder(),
            *ctx.visibility(),
            vec![destination_attribute_value_id],
        ))
        .await?;

        Ok(attribute_prototype_argument.id())
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentCreatedPayload {
    success: bool,
}

impl WsEvent {
    pub async fn component_created(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ComponentCreated(ComponentCreatedPayload { success: true }),
        )
        .await
    }
}
