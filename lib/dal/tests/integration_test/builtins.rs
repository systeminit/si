use dal::workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants;
use dal::{func::intrinsics::IntrinsicFunc, DalContext, Func, Schema, SchemaVariant};
use dal_test::test;
use petgraph::prelude::*;
use strum::IntoEnumIterator;

#[test]
async fn docker_image_has_one_qualfiication_map_prop(ctx: &DalContext) {
    let docker_image = Schema::list(ctx)
        .await
        .expect("list schemas")
        .iter()
        .find(|schema| schema.name() == "Docker Image")
        .expect("docker image does not exist")
        .to_owned();

    let variant = SchemaVariant::list_for_schema(ctx, docker_image.id())
        .await
        .expect("get schema variants")
        .pop()
        .expect("get default variant");

    let root_prop_id = SchemaVariant::get_root_prop_id(ctx, variant.id())
        .await
        .expect("get root prop for variant");

    let workspace_snapshot = ctx.workspace_snapshot().expect("get snap").read().await;

    let child_prop_targets = workspace_snapshot
        .outgoing_targets_for_edge_weight_kind(root_prop_id, EdgeWeightKindDiscriminants::Use)
        .expect("get all child prop targets of root");

    let qualification_props: Vec<&NodeIndex> = child_prop_targets
        .iter()
        .filter(|&child_prop_target| {
            let node_weight = workspace_snapshot
                .get_node_weight(*child_prop_target)
                .expect("get node weight")
                .get_prop_node_weight()
                .expect("should be prop")
                .to_owned();

            node_weight.name() == "qualification"
        })
        .collect();

    assert_eq!(1, qualification_props.len());
}

#[test]
async fn builtin_funcs_and_schemas_are_not_empty(ctx: &DalContext) {
    let funcs: Vec<String> = Func::list(ctx)
        .await
        .expect("list funcs should work")
        .iter()
        .map(|f| f.name.to_owned())
        .collect();

    // Check that the funcs at least contain all intrinsics.
    let intrinsics: Vec<String> = IntrinsicFunc::iter()
        .map(|intrinsic| intrinsic.name().to_owned())
        .collect();
    for intrinsic in intrinsics {
        assert!(funcs.contains(&intrinsic));
    }

    // Ensure that we have at least one schema variant for every schema and that we have at least
    // one schema.
    let schemas: Vec<Schema> = Schema::list(ctx).await.expect("could not list schemas");
    assert!(!schemas.is_empty());
    for schema in schemas {
        let schema_variants: Vec<SchemaVariant> = SchemaVariant::list_for_schema(ctx, schema.id())
            .await
            .expect("could not list schema variants");
        assert!(!schema_variants.is_empty());
    }
}
