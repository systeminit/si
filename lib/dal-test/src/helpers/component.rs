#![allow(clippy::expect_used)]

use color_eyre::eyre::eyre;
use dal::{
    AttributeValue,
    Component,
    ComponentId,
    DalContext,
    FuncId,
    attribute::{
        path::AttributePath,
        value::subscription::ValueSubscription,
    },
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
use crate::Result;

/// Create a component with the given name and schema variant
pub async fn create(
    ctx: &DalContext,
    variant: impl SchemaVariantKey,
    name: impl AsRef<str>,
) -> Result<ComponentId> {
    let variant_id = variant.lookup_schema_variant(ctx).await?;
    let view_id = View::get_id_for_default(ctx).await?;

    Ok(
        Component::new(ctx, name.as_ref().to_string(), variant_id, view_id)
            .await?
            .id(),
    )
}

/// Subscribe from one component's attribute to another component's attribute
pub async fn subscribe(
    ctx: &mut DalContext,
    subscriber: ComponentId,
    path: impl Into<String>,
    subscribed_to: ComponentId,
    subscribed_to_path: &str,
) -> Result<()> {
    let subscriber_root_id = Component::root_attribute_value_id(ctx, subscriber).await?;
    let subscriber_av_id = AttributePath::from_json_pointer(path)
        .resolve(ctx, subscriber_root_id)
        .await?
        .expect("attribute to exist");
    AttributeValue::subscribe(
        ctx,
        subscriber_av_id,
        ValueSubscription {
            attribute_value_id: Component::root_attribute_value_id(ctx, subscribed_to).await?,
            path: AttributePath::from_json_pointer(subscribed_to_path),
        },
    )
    .await?;
    Ok(())
}

/// Execute a the management function and apply the result to the component
pub async fn execute_management_func(
    ctx: &DalContext,
    component_id: ComponentId,
    func_id: FuncId,
) -> Result<()> {
    let mut prototype_ids = ManagementPrototype::list_ids_for_func_id(ctx, func_id)
        .await?
        .into_iter();
    let prototype_id = prototype_ids
        .next()
        .ok_or(eyre!("no prototypes for func"))?;
    if prototype_ids.next().is_some() {
        return Err(eyre!("multiple prototypes for func"));
    }
    let mut execution_result = ManagementPrototype::execute_by_id(
        ctx,
        prototype_id,
        component_id,
        Some(view_id(ctx, component_id).await?),
    )
    .await?;
    let result: ManagementFuncReturn = execution_result
        .result
        .take()
        .ok_or(eyre!("no result for func"))?
        .try_into()?;

    assert_eq!(ManagementFuncStatus::Ok, result.status);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, component_id, operations, execution_result, None)
        .await?
        .operate()
        .await?;
    Ok(())
}

/// Get the single view id for the component
pub async fn view_id(ctx: &DalContext, component_id: ComponentId) -> Result<ViewId> {
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
