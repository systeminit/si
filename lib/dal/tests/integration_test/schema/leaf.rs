use std::sync::Arc;

use base64::Engine;
use dal::{
    Component,
    DalContext,
    Func,
    Schema,
    attribute::value::{
        DependentValueGraph,
        dependent_value_graph::DependentValue,
    },
    func::leaf::{
        LeafInputLocation,
        LeafKind,
    },
    schema::leaf::LeafPrototype,
    workspace_snapshot::DependentValueRoot,
};
use dal_test::{
    Result,
    helpers::create_component_for_default_schema_name_in_default_view,
    prelude::ChangeSetTestHelpers,
    test,
};
use pretty_assertions_sorted::assert_eq;
use tokio::sync::RwLock;

#[test]
async fn leaf_prototype_rerun(ctx: &mut DalContext) -> Result<()> {
    let schema = Schema::get_by_name(ctx, "swifty").await?;

    let inputs = vec![LeafInputLocation::Domain];

    let leaf_qual_code = "async function main(input) {
                return {
                    result: 'success',
                    message: Math.random().toString(),
                };
            }";

    let leaf_qual_func = Func::new(
        ctx,
        "test:schemaLevelQualification",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        dal::FuncBackendKind::JsAttribute,
        dal::FuncBackendResponseType::Qualification,
        "main".into(),
        Some(base64::engine::general_purpose::STANDARD_NO_PAD.encode(leaf_qual_code)),
        false,
    )
    .await?;

    let leaf_qual_proto = LeafPrototype::new(
        ctx,
        schema.id(),
        LeafKind::Qualification,
        inputs.clone(),
        leaf_qual_func.id,
    )
    .await?;

    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "the east wing")
            .await?;
    let root_attribute_value_id = Component::root_attribute_value_id(ctx, component.id()).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let view = component.view(ctx).await?.expect("a view should exist");

    let quals_before = view
        .get("qualification")
        .and_then(|qual| qual.get(&leaf_qual_func.name))
        .cloned()
        .expect("new qualification should exist");

    let destination_map_id = leaf_qual_proto
        .resolve_output_map(ctx, root_attribute_value_id)
        .await?;

    let elem_av_id = leaf_qual_proto
        .resolve_output_element(ctx, root_attribute_value_id)
        .await?
        .expect(
            "Failed to resolve output element, but should have since qual should have executed",
        );

    ctx.add_dependent_values_and_enqueue(vec![elem_av_id])
        .await?;

    let dep_graph = DependentValueGraph::new(
        ctx,
        DependentValueRoot::get_dependent_value_roots(ctx).await?,
    )
    .await?;

    assert!(
        dep_graph.contains_value(DependentValue::OverlayDestination {
            leaf_prototype_id: leaf_qual_proto.id(),
            destination_map_id,
            destination_element_id: Some(elem_av_id),
            root_attribute_value_id,
        })
    );

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let view = component.view(ctx).await?.expect("a view should exist");

    let quals_after = view
        .get("qualification")
        .and_then(|qual| qual.get(&leaf_qual_func.name))
        .cloned()
        .expect("new qualification should exist");

    assert_ne!(quals_before, quals_after);

    Ok(())
}

#[test]
async fn leaf_prototype_tests(ctx: &mut DalContext) -> Result<()> {
    let schema = Schema::get_by_name(ctx, "swifty").await?;

    let inputs = vec![
        LeafInputLocation::Domain,
        LeafInputLocation::DeletedAt,
        LeafInputLocation::Code,
        LeafInputLocation::Secrets,
        LeafInputLocation::Resource,
    ];

    let leaf_qual_code = "async function main() {
                return {
                    result: 'success',
                    message: 'this cannot fail, it can only be failed',
                };
            }";

    let leaf_qual_func = Func::new(
        ctx,
        "test:schemaLevelQualification",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        dal::FuncBackendKind::JsAttribute,
        dal::FuncBackendResponseType::Qualification,
        "main".into(),
        Some(base64::engine::general_purpose::STANDARD_NO_PAD.encode(leaf_qual_code)),
        false,
    )
    .await?;

    let leaf_qual_proto = LeafPrototype::new(
        ctx,
        schema.id(),
        LeafKind::Qualification,
        inputs.clone(),
        leaf_qual_func.id,
    )
    .await?;

    let fetched_prototype = LeafPrototype::get_by_id(ctx, leaf_qual_proto.id()).await?;

    assert_eq!(leaf_qual_proto, fetched_prototype);

    let leaf_inputs: Vec<_> = fetched_prototype.leaf_inputs().collect();

    assert_eq!(inputs, leaf_inputs);

    let func_id = LeafPrototype::func_id(ctx, leaf_qual_proto.id()).await?;

    assert_eq!(leaf_qual_func.id, func_id);

    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "the east wing")
            .await?;
    let root_attribute_value_id = Component::root_attribute_value_id(ctx, component.id()).await?;

    let mut resolved_inputs = leaf_qual_proto
        .resolve_inputs(ctx, root_attribute_value_id)
        .await?;
    let destination_map_id = leaf_qual_proto
        .resolve_output_map(ctx, root_attribute_value_id)
        .await?;

    let dep_graph = DependentValueGraph::new(
        ctx,
        resolved_inputs
            .iter()
            .map(|id| DependentValueRoot::Unfinished(id.into()))
            .collect(),
    )
    .await?;

    assert!(
        dep_graph.contains_value(DependentValue::OverlayDestination {
            leaf_prototype_id: leaf_qual_proto.id(),
            destination_map_id,
            destination_element_id: None,
            root_attribute_value_id,
        })
    );

    let read_lock = Arc::new(RwLock::new(()));
    let mut result = LeafPrototype::execute(
        ctx,
        leaf_qual_proto.id(),
        destination_map_id,
        root_attribute_value_id,
        read_lock,
    )
    .await?;

    resolved_inputs.sort();
    result.input_attribute_value_ids.sort();
    assert_eq!(resolved_inputs, result.input_attribute_value_ids);

    let expected_qual_result = serde_json::json!({
        "result": "success",
        "message": "this cannot fail, it can only be failed",
    });

    assert_eq!(
        &expected_qual_result,
        result
            .func_run_value
            .unprocessed_value()
            .expect("it should have a value")
    );

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let view = component.view(ctx).await?.expect("a view should exist");

    let quals = view
        .get("qualification")
        .and_then(|qual| qual.get(&leaf_qual_func.name))
        .expect("new qualification should exist");

    assert_eq!(&expected_qual_result, quals);

    Ok(())
}
