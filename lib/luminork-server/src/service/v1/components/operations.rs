//! Core component operation logic shared between single and bulk handlers.
//!
//! Functions in this module contain all business logic and transactional operations
//! (audit logs, WsEvents) but deliberately exclude:
//! - HTTP layer concerns (commits, PostHog tracking, response building)
//! - These are the caller's responsibility

use std::collections::HashMap;

use dal::{
    AttributeValue,
    Component,
    ComponentId,
    DalContext,
    Prop,
    Schema,
    SchemaVariant,
    WsEvent,
    attribute::attributes::AttributeSources,
    cached_module::CachedModule,
    diagram::view::View,
    prop::PropPath,
};
use si_events::audit_log::AuditLogKind;
use si_id::ViewId;

use super::{
    ComponentReference,
    ComponentViewV1,
    ComponentsError,
    ComponentsResult,
    get_component::into_front_end_type,
    resolve_component_reference,
    update_component::{
        SecretPropKey,
        resolve_secret_id,
    },
};

/// Core logic for creating a component.
///
/// Includes all business logic, audit logging, and transactional operations.
/// Does NOT commit or track analytics - those are the caller's responsibility.
///
/// # Arguments
/// * `component_list` - Pre-fetched list of component IDs for manager resolution.
///   Pass empty slice if `managed_by` is empty (lazy optimization).
#[allow(clippy::too_many_arguments)]
pub(super) async fn create_component_core(
    ctx: &DalContext,
    name: String,
    schema_name: String,
    view_name: Option<String>,
    resource_id: Option<String>,
    attributes: AttributeSources,
    managed_by: ComponentReference,
    use_working_copy: Option<bool>,
    component_list: &[ComponentId],
) -> ComponentsResult<ComponentViewV1> {
    // Find schema by name (try cached module first, then installed schemas)
    let schema_id =
        match CachedModule::find_latest_for_schema_name(ctx, schema_name.as_str()).await? {
            Some(module) => module.schema_id,
            None => match Schema::get_by_name_opt(ctx, schema_name.as_str()).await? {
                Some(schema) => schema.id(),
                None => return Err(ComponentsError::SchemaNameNotFound(schema_name)),
            },
        };

    // Get or install default variant
    let mut variant_id = Schema::get_or_install_default_variant(ctx, schema_id).await?;

    // Use working copy variant if requested
    if use_working_copy.unwrap_or(false) {
        match SchemaVariant::get_unlocked_for_schema(ctx, schema_id).await? {
            Some(unlocked_variant) => {
                variant_id = unlocked_variant.id();
            }
            None => {
                return Err(ComponentsError::NoWorkingCopy(schema_id));
            }
        }
    }

    let variant = SchemaVariant::get_by_id(ctx, variant_id).await?;

    // Resolve or create view
    let view_id: ViewId = if let Some(view_name) = view_name {
        if let Some(view) = View::find_by_name(ctx, view_name.as_str()).await? {
            view.id()
        } else {
            let view = View::new(ctx, view_name.as_str()).await?;
            view.id()
        }
    } else {
        View::get_id_for_default(ctx).await?
    };

    // Create the component
    let mut component = Component::new(ctx, name, variant_id, view_id).await?;
    let comp_name = component.name(ctx).await?;
    let initial_geometry = component.geometry(ctx, view_id).await?;
    component
        .set_geometry(
            ctx,
            view_id,
            0,
            0,
            initial_geometry.width(),
            initial_geometry.height(),
        )
        .await?;

    // Write audit log for creation (transactional, queued)
    ctx.write_audit_log(
        AuditLogKind::CreateComponent {
            name: comp_name.clone(),
            component_id: component.id(),
            schema_variant_id: variant_id,
            schema_variant_name: variant.display_name().to_string(),
        },
        comp_name.clone(),
    )
    .await?;

    // Apply attributes if provided
    if !attributes.is_empty() {
        dal::update_attributes(ctx, component.id(), attributes.clone()).await?;
    }

    // Set resource_id if provided
    if let Some(resource_id) = resource_id {
        let resource_prop_path = ["root", "si", "resourceId"];
        let resource_prop_id =
            Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(resource_prop_path)).await?;

        let av_for_resource_id =
            Component::attribute_value_for_prop_id(ctx, component.id(), resource_prop_id).await?;

        AttributeValue::update(
            ctx,
            av_for_resource_id,
            Some(serde_json::to_value(resource_id)?),
        )
        .await?;
    }

    // Set up management relationship if specified
    if !managed_by.is_empty() {
        let manager_component_id =
            resolve_component_reference(ctx, &managed_by, component_list).await?;

        Component::manage_component(ctx, manager_component_id, component.id()).await?;
    }

    // Write audit log for update (attributes/relationships changed)
    ctx.write_audit_log(
        AuditLogKind::UpdateComponent {
            component_id: component.id(),
            component_name: comp_name.clone(),
        },
        comp_name.clone(),
    )
    .await?;

    // Return assembled view
    Ok(ComponentViewV1::assemble(ctx, component.id()).await?)
}

/// Core logic for updating a component.
///
/// Includes all business logic, audit logging, WsEvent emission, and transactional operations.
/// Does NOT commit or track analytics - those are the caller's responsibility.
pub(super) async fn update_component_core(
    ctx: &DalContext,
    component_id: ComponentId,
    name: Option<String>,
    resource_id: Option<String>,
    secrets: HashMap<SecretPropKey, serde_json::Value>,
    attributes: AttributeSources,
) -> ComponentsResult<ComponentViewV1> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let old_name = component.name(ctx).await?;

    // Update name if provided
    if let Some(name) = name {
        component.set_name(ctx, name.as_str()).await?;

        // Write audit log for rename (transactional, queued)
        ctx.write_audit_log(
            AuditLogKind::RenameComponent {
                component_id,
                old_name,
                new_name: name.clone(),
            },
            name.clone(),
        )
        .await?;
    }

    let schema_variant = component.schema_variant(ctx).await?;
    let variant_id = schema_variant.id;
    let is_secret_defining = SchemaVariant::is_secret_defining(ctx, variant_id).await?;

    // Apply attributes if provided
    if !attributes.is_empty() {
        dal::update_attributes(ctx, component_id, attributes.clone()).await?;
    }

    // Handle secrets (only for secret-defining components)
    if !is_secret_defining && !secrets.is_empty() {
        return Err(ComponentsError::NotSecretDefiningComponent(component_id));
    }

    if is_secret_defining {
        for (key, value) in secrets.into_iter() {
            let prop_id = key.prop_id(ctx, variant_id).await?;
            let secret_id = resolve_secret_id(ctx, &value).await?;
            let attribute_value_id =
                Component::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;
            dal::Secret::attach_for_attribute_value(ctx, attribute_value_id, Some(secret_id))
                .await?;
        }
    }

    // Set resource_id if provided
    if let Some(resource_id) = resource_id {
        let resource_prop_path = ["root", "si", "resourceId"];
        let resource_prop_id =
            Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(resource_prop_path)).await?;

        let av_for_resource_id =
            Component::attribute_value_for_prop_id(ctx, component.id(), resource_prop_id).await?;

        AttributeValue::update(
            ctx,
            av_for_resource_id,
            Some(serde_json::to_value(resource_id)?),
        )
        .await?;
    }

    // Get updated component and emit WsEvent (transactional, queued)
    let updated_component = Component::get_by_id(ctx, component_id).await?;
    let new_name = updated_component.name(ctx).await?;
    WsEvent::component_updated(
        ctx,
        into_front_end_type(ctx, updated_component.clone()).await?,
    )
    .await?
    .publish_on_commit(ctx)
    .await?;

    // Write audit log for update (transactional, queued)
    ctx.write_audit_log(
        AuditLogKind::UpdateComponent {
            component_id: updated_component.id(),
            component_name: new_name.clone(),
        },
        new_name.clone(),
    )
    .await?;

    // Return assembled view
    Ok(ComponentViewV1::assemble(ctx, component_id).await?)
}
