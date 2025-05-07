use dal::{
    AttributePrototype,
    AttributeValue,
    Component,
    ComponentId,
    DalContext,
    InputSocket,
    OutputSocket,
    attribute::prototype::argument::AttributePrototypeArgument,
    qualification::QualificationSummary,
};
use si_frontend_types::newhotness::component::{
    Component as ComponentMv,
    ComponentDiff,
};
use telemetry::prelude::*;

#[instrument(name = "dal_materialized_views.component", level = "debug", skip_all)]
pub async fn assemble(ctx: DalContext, component_id: ComponentId) -> crate::Result<ComponentMv> {
    let ctx = &ctx;
    let schema_variant = Component::schema_variant_for_component_id(ctx, component_id).await?;
    let schema = schema_variant.schema(ctx).await?;
    let has_resource = Component::resource_by_id(ctx, component_id)
        .await?
        .is_some();
    let stats = QualificationSummary::individual_stats(ctx, component_id).await?;

    // TODO(Wendy) - There is probably a better way to do this
    let input_socket_ids =
        InputSocket::list_ids_for_schema_variant(ctx, schema_variant.id()).await?;
    let mut input_count = 0;
    for input_socket_id in input_socket_ids {
        let attribute_value_id = InputSocket::component_attribute_value_for_input_socket_id(
            ctx,
            input_socket_id,
            component_id,
        )
        .await?;
        let attribute_prototype_id = AttributeValue::prototype_id(ctx, attribute_value_id).await?;
        let attribute_prototype_argument_ids =
            AttributePrototype::list_arguments_for_id(ctx, attribute_prototype_id).await?;
        for attribute_prototype_argument_id in attribute_prototype_argument_ids {
            let attribute_prototype_argument =
                AttributePrototypeArgument::get_by_id(ctx, attribute_prototype_argument_id).await?;
            if let Some(targets) = attribute_prototype_argument.targets() {
                if targets.destination_component_id == component_id {
                    input_count += 1;
                }
            }
        }
    }

    // TODO(Wendy) - There is DEFINITELY a better way to do this, this approach can't react to the output socket changing!
    let output_socket_ids =
        OutputSocket::list_ids_for_schema_variant(ctx, schema_variant.id()).await?;
    let mut output_count = 0;
    for output_socket_id in output_socket_ids {
        let attribute_prototype_argument_ids =
            OutputSocket::prototype_arguments_using_for_id(ctx, output_socket_id).await?;
        for attribute_prototype_argument_id in attribute_prototype_argument_ids {
            let attribute_prototype_argument =
                AttributePrototypeArgument::get_by_id(ctx, attribute_prototype_argument_id).await?;
            if let Some(targets) = attribute_prototype_argument.targets() {
                if targets.source_component_id == component_id {
                    output_count += 1;
                }
            }
        }
    }

    let diff_count = Component::get_diff_count(ctx, component_id).await?;

    let root_attribute_value_id = Component::root_attribute_value_id(ctx, component_id).await?;
    let domain_attribute_value_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain"]).await?;
    let secrets_attribute_value_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "secrets"]).await?;
    let si_attribute_value_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "si"]).await?;
    let resource_value_attribute_value_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "resource_value"]).await?;

    let dal_component_diff = Component::get_diff(ctx, component_id).await?;
    let diff = match dal_component_diff.diff {
        Some(code_view) => code_view.code,
        None => None,
    };
    let resource_diff = ComponentDiff {
        current: dal_component_diff.current.code,
        diff,
    };

    Ok(ComponentMv {
        id: component_id,
        name: Component::name_by_id(ctx, component_id).await?,
        schema_name: schema.name.to_owned(),
        schema_id: schema.id(),
        schema_variant_id: schema_variant.id(),
        schema_variant_name: schema_variant.display_name().to_owned(),
        schema_category: schema_variant.category().to_owned(),
        schema_variant_description: schema_variant.description().to_owned(),
        schema_variant_doc_link: schema_variant.link().to_owned(),
        has_resource,
        qualification_totals: stats,
        input_count,
        // FIXME(nick): the output count will be out of date given the output socket is not
        // relevant to the component's merkle tree hash. We will need to find (likely through
        // derivation or inference rather than an MV) a way to calculate this efficiently and
        // accurately.
        output_count,
        diff_count,
        root_attribute_value_id,
        domain_attribute_value_id,
        secrets_attribute_value_id,
        si_attribute_value_id,
        resource_value_attribute_value_id,
        resource_diff,
    })
}
