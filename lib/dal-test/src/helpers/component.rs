#![allow(clippy::expect_used)]

use color_eyre::eyre::eyre;
use dal::{
    Component,
    ComponentId,
    DalContext,
    InputSocket,
    OutputSocket,
    attribute::attributes,
    diagram::{
        geometry::Geometry,
        view::View,
    },
    management::{
        ManagementFuncReturn,
        ManagementOperator,
        prototype::ManagementPrototype,
    },
};
use si_id::ViewId;
use veritech_client::ManagementFuncStatus;

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

    let management_prototype = ManagementPrototype::list_for_variant_id(ctx, schema_variant_id)
        .await?
        .into_iter()
        .find(|proto| proto.name() == prototype_name)
        .expect("could not find prototype");

    Ok(management_prototype)
}

/// Execute a the management function and apply the result to the component
pub async fn execute_management_func(
    ctx: &DalContext,
    component: impl ComponentKey,
    func: impl FuncKey,
) -> Result<()> {
    let func_id = func::id(ctx, func).await?;
    let component_id = id(ctx, component).await?;
    let mut prototype_ids = ManagementPrototype::list_ids_for_func_id(ctx, func_id)
        .await?
        .into_iter();
    let prototype_id = prototype_ids
        .next()
        .ok_or(eyre!("no prototypes for func"))?;
    if prototype_ids.next().is_some() {
        return Err(eyre!("multiple prototypes for func"));
    }

    let (geometries, placeholders, run_channel, _) = ManagementPrototype::start_execution(
        ctx,
        prototype_id,
        component_id,
        Some(view_id(ctx, component_id).await?),
    )
    .await?;

    let mut execution_result = ManagementPrototype::finalize_execution(
        ctx,
        component_id,
        prototype_id,
        geometries,
        placeholders,
        run_channel,
    )
    .await?;

    let result: ManagementFuncReturn = execution_result
        .result
        .take()
        .ok_or(eyre!("no result for func"))?
        .try_into()?;

    assert_eq!(ManagementFuncStatus::Ok, result.status);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        component_id,
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;
    Ok(())
}

/// Get the value of the component's domain attribute
pub async fn domain(ctx: &DalContext, component: impl ComponentKey) -> Result<serde_json::Value> {
    value::get(ctx, (component, "/domain")).await
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
/// Connect two components by their sockets
///
pub async fn connect(
    ctx: &DalContext,
    source: (impl ComponentKey, &str),
    dest: (impl ComponentKey, &str),
) -> Result<()> {
    let source_id = id(ctx, source.0).await?;
    let source_variant_id = Component::schema_variant_id(ctx, source_id).await?;
    let source_socket =
        OutputSocket::find_with_name_or_error(ctx, source.1, source_variant_id).await?;
    let dest_id = id(ctx, dest.0).await?;
    let dest_variant_id = Component::schema_variant_id(ctx, dest_id).await?;
    let dest_socket = InputSocket::find_with_name_or_error(ctx, dest.1, dest_variant_id).await?;
    Component::connect(
        ctx,
        source_id,
        source_socket.id(),
        dest_id,
        dest_socket.id(),
    )
    .await?;
    Ok(())
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
