use std::collections::{HashMap, VecDeque};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    diagram::{self, summary_diagram::SummaryDiagramError},
    property_editor::values_summary::{
        PropertyEditorValuesSummary, PropertyEditorValuesSummaryError,
    },
    socket::SocketError,
    AttributeContextError, AttributePrototype, AttributePrototypeArgumentError,
    AttributePrototypeError, AttributeValue, AttributeValueError, AttributeValueId, Component,
    ComponentError, ComponentId, ComponentView, Connection, DalContext, DiagramError, Edge,
    EdgeError, InternalProvider, InternalProviderError, PropError, PropKind, SchemaVariant,
    SchemaVariantId, Socket, SocketId, StandardModel, StandardModelError,
};

use super::{ComponentResult, ComponentViewError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ComponentMigrateError {
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value view of {0} is not associated with a prop or provider!")]
    AttributeValueViewNoPropOrProvider(AttributeValueId),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component view error: {0}")]
    ComponentView(#[from] ComponentViewError),
    #[error("diagram error: {0}")]
    Diagram(#[from] DiagramError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("input socket error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("property editor value summary error: {0}")]
    PropertyEditorValuesSummary(#[from] PropertyEditorValuesSummaryError),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("summary diagram error: {0}")]
    SummaryDiagram(#[from] SummaryDiagramError),
}

pub type ComponentMigrateResult<T> = Result<T, ComponentMigrateError>;

pub async fn migrate_component_to_schema_variant(
    ctx: &DalContext,
    component_id: ComponentId,
    schema_variant_id: SchemaVariantId,
) -> ComponentMigrateResult<()> {
    // Gather up the original socket map so we can migrate to new edges
    let original_sockets: HashMap<SocketId, String> = Socket::list_for_component(ctx, component_id)
        .await?
        .iter()
        .map(|socket| (*socket.id(), socket.name().into()))
        .collect();

    let original_edges = Edge::list_for_component(ctx, component_id).await?;

    // Delete all the original edges
    for edge in &original_edges {
        edge.clone().delete_and_propagate(ctx).await?;
    }

    let original_component_view = ComponentView::new(ctx, component_id).await?.properties;

    // Respin the component, this deletes all the attribute values for the
    // component so we have to gather up the prototype info *before* we do
    // this
    let new_component = Component::respin(ctx, component_id, schema_variant_id).await?;
    let new_sockets: HashMap<String, SocketId> = Socket::list_for_component(ctx, component_id)
        .await?
        .iter()
        .map(|socket| (socket.name().into(), *socket.id()))
        .collect();

    let mut json_for_new_sv = build_empty_json_for_prop_tree(ctx, schema_variant_id).await?;
    serde_value_merge_in_place_recursive(&mut json_for_new_sv, original_component_view);

    // Call update for context on the root attribute value of the new
    // component with the constructed attribute view. We use
    // update_for_context because it will populate all the nested values
    // for the tree and recalculate all the implicit attribute values
    // from root to leaf.
    if json_for_new_sv != serde_json::Value::Null {
        let root_attribute_value = new_component.root_attribute_value(ctx).await?;
        AttributeValue::update_for_context_without_propagating_dependent_values(
            ctx,
            *root_attribute_value.id(),
            None,
            root_attribute_value.context,
            Some(json_for_new_sv),
            None,
        )
            .await?;

        restore_prototypes_and_implicit_values(ctx, component_id).await?;
    }

    // Update all the cached summaries for the new component, this part is pretty expensive
    AttributeValue::remove_dependency_summaries_for_deleted_values(ctx).await?;
    AttributeValue::update_component_dependencies(ctx, component_id).await?;
    PropertyEditorValuesSummary::create_or_update_component_entry(ctx, component_id).await?;
    diagram::summary_diagram::update_socket_summary(ctx, &new_component).await?;
    diagram::summary_diagram::component_update(
        ctx,
        &component_id,
        new_component.name(ctx).await?,
        new_component.color(ctx).await?.unwrap_or_default(),
        new_component.get_type(ctx).await?,
        new_component.resource(ctx).await?.payload.is_some(),
        None,
    )
        .await?;

    // Restore edges if matching sockets exist in the migrated component This
    // should probably use the connection annotation for matching, instead of
    // socket name?
    for edge in original_edges {
        let tail_component_id = edge.tail_component_id();
        let tail_socket_id = if tail_component_id == component_id {
            original_sockets
                .get(&edge.tail_socket_id())
                .and_then(|socket_name| new_sockets.get(socket_name))
                .copied()
        } else {
            Some(edge.tail_socket_id())
        };

        let head_component_id = edge.head_component_id();
        let head_socket_id = if head_component_id == component_id {
            original_sockets
                .get(&edge.head_socket_id())
                .and_then(|socket_name| new_sockets.get(socket_name))
                .copied()
        } else {
            Some(edge.head_socket_id())
        };

        if let (Some(tail_socket_id), Some(head_socket_id)) = (tail_socket_id, head_socket_id) {
            Connection::new(
                ctx,
                edge.tail_node_id(),
                tail_socket_id,
                edge.head_node_id(),
                head_socket_id,
                *edge.kind(),
            )
                .await?;
        }
    }

    Ok(())
}

// If a schema variant level prototype exists for this value's context,
// reset the value to use that prototype and we're done. But we also
// need to emit all the internal provider values, since those aren't
// being emitted by the update_for_context if we don't propagate
// dependent values (but we don't want to propagate dependent values
// until the prototypes are fixed)
pub async fn restore_prototypes_and_implicit_values(
    ctx: &DalContext,
    component_id: ComponentId,
) -> ComponentMigrateResult<()> {
    for value in AttributeValue::find_all_values_for_component_id(ctx, component_id)
        .await?
        .iter_mut()
    {
        let value_context = value.attribute_value.context;
        let variant_context = value_context.clone_with_component_id(ComponentId::NONE);
        if let Some(variant_prototype) = AttributePrototype::find_for_context_and_key(
            ctx,
            variant_context,
            &value.attribute_value.key,
        )
            .await?
            .into_iter()
            .next()
        {
            if value
                .attribute_value
                .context
                .is_least_specific_field_kind_internal_provider()?
            {
                let internal_provider = InternalProvider::get_by_id(
                    ctx,
                    &value.attribute_value.context.internal_provider_id(),
                )
                    .await?
                    .ok_or_else(|| {
                        AttributeValueError::InternalProviderNotFound(
                            value.attribute_value.context.internal_provider_id(),
                        )
                    })?;
                if internal_provider.is_internal_consumer() {
                    // Prototypes for implicit internal providers are set by implicit emit,
                    // but they also don't matter since we don't exec their functions
                    internal_provider
                        .implicit_emit(ctx, &mut value.attribute_value)
                        .await?;
                } else {
                    value
                        .attribute_value
                        .set_attribute_prototype(ctx, variant_prototype.id())
                        .await?;
                }
            } else {
                value
                    .attribute_value
                    .set_attribute_prototype(ctx, variant_prototype.id())
                    .await?;
            }
        }
    }

    Ok(())
}

pub async fn build_empty_json_for_prop_tree(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> ComponentResult<serde_json::Value> {
    // This should fetch the entire prop tree in the correct order in a single query.
    let mut result = serde_json::json!({});

    let root_prop = SchemaVariant::find_root_prop(ctx, schema_variant_id)
        .await?
        .ok_or(PropError::NotFoundAtPath("root".into(), *ctx.visibility()))?;

    let mut work_queue = VecDeque::from([root_prop]);

    while let Some(prop) = work_queue.pop_front() {
        if matches!(prop.kind(), PropKind::Object) {
            work_queue.extend(prop.child_props(ctx).await?);
        }

        let path = prop.path();
        let mut parts = path.as_parts();
        if parts.len() <= 1 {
            continue;
        }

        parts[0] = "";
        let parent_path = parts[..parts.len() - 1].join("/");
        let last_part = parts[parts.len() - 1].to_string();

        if let Some(value) = result.pointer_mut(&parent_path) {
            if let Some(object) = value.as_object_mut() {
                object.insert(
                    last_part,
                    match prop.kind() {
                        PropKind::String => serde_json::Value::Null,
                        PropKind::Boolean => serde_json::Value::Null,
                        PropKind::Integer => serde_json::Value::Null,
                        PropKind::Array => serde_json::json!([]),
                        PropKind::Map => serde_json::json!({}),
                        PropKind::Object => serde_json::json!({}),
                    },
                );
            }
        }
    }

    Ok(result)
}

pub fn serde_value_merge_in_place_recursive(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (a @ &mut serde_json::Value::Object(_), serde_json::Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                serde_value_merge_in_place_recursive(
                    a.entry(k).or_insert(serde_json::Value::Null),
                    v,
                );
            }
        }
        (a, b) => *a = b,
    }
}
