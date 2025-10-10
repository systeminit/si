#![allow(clippy::expect_used)]

use color_eyre::eyre::eyre;
use dal::{
    Component,
    ComponentId,
    DalContext,
    attribute::attributes,
    diagram::{
        geometry::Geometry,
        view::View,
    },
    management::prototype::ManagementPrototype,
};
use si_id::ViewId;

use super::schema::variant::SchemaVariantKey;
use crate::{
    Result,
    expected::ExpectComponent,
    helpers::{
        attribute::value,
        func::{
            self,
            FuncKey,
        },
        schema::variant,
    },
};

/// Lookup a component by name or id
pub async fn id(ctx: &DalContext, key: impl ComponentKey) -> Result<ComponentId> {
    ComponentKey::id(ctx, key).await
}

/// Create a component with the given name and schema variant
pub async fn create(
    ctx: &DalContext,
    variant: impl SchemaVariantKey,
    name: impl AsRef<str>,
) -> Result<ComponentId> {
    let variant_id = variant::id(ctx, variant).await?;
    let view_id = View::get_id_for_default(ctx).await?;

    Ok(
        Component::new(ctx, name.as_ref().to_string(), variant_id, view_id)
            .await?
            .id(),
    )
}

/// Create a component with the given name and schema variant, and set some attributes
pub async fn create_and_set(
    ctx: &DalContext,
    variant: impl SchemaVariantKey,
    name: impl AsRef<str>,
    values: serde_json::Value,
) -> Result<ComponentId> {
    let component_id = create(ctx, variant, name).await?;
    update(ctx, component_id, values).await?;
    Ok(component_id)
}

/// Create a component with the given name and schema variant
pub async fn update(
    ctx: &DalContext,
    component: impl ComponentKey,
    values: serde_json::Value,
) -> Result<()> {
    let component_id = self::id(ctx, component).await?;
    let values = serde_json::from_value(values)?;
    attributes::update_attributes(ctx, component_id, values).await?;
    Ok(())
}

/// Remove a component from the graph (even if it has connections/sockets/children)
pub async fn remove(ctx: &DalContext, component: impl ComponentKey) -> Result<()> {
    let component_id = id(ctx, component).await?;
    Component::remove(ctx, component_id).await?;
    Ok(())
}

/// Find a management prototype by the prototype name (NOT THE FUNC NAME)
pub async fn find_management_prototype(
    ctx: &DalContext,
    component_id: ComponentId,
    prototype_name: &str,
) -> Result<ManagementPrototype> {
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;

    let management_prototype =
        ManagementPrototype::list_for_schema_and_variant_id(ctx, schema_variant_id)
            .await?
            .into_iter()
            .find(|proto| proto.name() == prototype_name)
            .expect("could not find prototype");

    Ok(management_prototype)
}

/// Execute a the management function and apply the result to the component
pub async fn execute_management_func(
    ctx: &mut DalContext,
    component: impl ComponentKey,
    func: impl FuncKey,
) -> Result<()> {
    use crate::helpers::ChangeSetTestHelpers;

    let func_id = func::id(ctx, func).await?;
    let component_id = id(ctx, component).await?;

    // Get the function name to find the management prototype
    let func = dal::Func::get_by_id(ctx, func_id).await?;

    // Find the management prototype by name
    let management_prototype = find_management_prototype(ctx, component_id, &func.name).await?;

    // Enqueue the management function job
    ChangeSetTestHelpers::enqueue_management_func_job(
        ctx,
        management_prototype.id(),
        component_id,
        None,
    )
    .await?;

    // Commit changes
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Wait for the management job to complete
    ChangeSetTestHelpers::wait_for_mgmt_job_to_run(ctx, management_prototype.id(), component_id)
        .await?;

    // Wait for dependent value update (DVU)
    dal::ChangeSet::wait_for_dvu(ctx, false).await?;

    // Update to visibility
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    Ok(())
}

/// Get the value of the component's domain attribute
pub async fn domain(ctx: &DalContext, component: impl ComponentKey) -> Result<serde_json::Value> {
    value::get(ctx, (component, "/domain")).await
}

/// Get the value of the component (including /si, /domain, etc.)
pub async fn value(ctx: &DalContext, component: impl ComponentKey) -> Result<serde_json::Value> {
    value::get(ctx, (component, "")).await
}

/// Get the single view id for the component
pub async fn view_id(ctx: &DalContext, component: impl ComponentKey) -> Result<ViewId> {
    let component_id = id(ctx, component).await?;
    let mut geometry_ids = Geometry::list_ids_by_component(ctx, component_id)
        .await?
        .into_iter();
    let geometry_id = geometry_ids
        .next()
        .ok_or(eyre!("no geometry for component"))?;
    if geometry_ids.next().is_some() {
        return Err(eyre!("multiple geometries for component"));
    }
    Ok(Geometry::get_view_id_by_id(ctx, geometry_id).await?)
}

///
/// Things that you can pass to reference components (name or id)
///
#[allow(async_fn_in_trait)]
pub trait ComponentKey {
    ///
    /// Turn this into a real ComponentId
    ///
    async fn id(ctx: &DalContext, key: Self) -> Result<ComponentId>;
}
impl ComponentKey for ComponentId {
    async fn id(_: &DalContext, key: Self) -> Result<ComponentId> {
        Ok(key)
    }
}
// "ComponentName" finds the component with that name
impl ComponentKey for &str {
    async fn id(ctx: &DalContext, key: Self) -> Result<ComponentId> {
        Ok(Component::get_by_name(ctx, key).await?)
    }
}
impl ComponentKey for ExpectComponent {
    async fn id(_: &DalContext, key: Self) -> Result<ComponentId> {
        Ok(key.id())
    }
}
impl ComponentKey for Component {
    async fn id(_: &DalContext, key: Self) -> Result<ComponentId> {
        Ok(key.id())
    }
}
