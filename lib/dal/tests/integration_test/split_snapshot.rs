use std::time::{Duration, Instant};

use dal::{
    diagram::view::View,
    prop::PropPath,
    workspace_snapshot::{split_snapshot::SplitSnapshot, DependentValueRoot},
    AttributeValue, Component, ComponentId, DalContext, Prop, Schema, SchemaVariant, Workspace,
    WorkspacePk,
};
use dal_test::{helpers::create_component_for_default_schema_name, test, Result};
//use pretty_assertions_sorted::assert_eq;

async fn set_av_for_prop_for_component(ctx: &DalContext, component_id: ComponentId) -> Result<()> {
    let variant_id = Component::schema_variant_id(ctx, component_id).await?;

    let rigid_designator_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new([
            "root",
            "domain",
            "possible_world_a",
            "wormhole_1",
            "wormhole_2",
            "wormhole_3",
            "rigid_designator",
        ]),
    )
    .await?;

    let rigid_designator_values =
        Component::attribute_values_for_prop_id(ctx, component_id, rigid_designator_prop_id)
            .await?;

    let rigid_designation = serde_json::json!("hesperus");

    let rigid_id = rigid_designator_values
        .first()
        .copied()
        .expect("should have a rigid_designator");

    AttributeValue::update(ctx, rigid_id, Some(rigid_designation)).await?;

    Ok(())
}

#[test]
async fn create_split_snapshot_workspace(ctx: &mut DalContext) -> Result<()> {
    const COMPONENTS_COUNT: usize = 500;

    let mut legacy_components = vec![];
    let view_id = View::get_id_for_default(ctx).await?;

    for i in 0..COMPONENTS_COUNT {
        let id =
            create_component_for_default_schema_name(ctx, "starfield", format!("{i}"), view_id)
                .await?
                .id();
        legacy_components.push(id);
    }

    // Creating all those components in one batch adds thousands of dvu roots, but that's anomalous
    DependentValueRoot::take_dependent_values(ctx).await?;

    let legacy_addr = ctx
        .workspace_snapshot()?
        .as_legacy_snapshot()?
        .write(ctx)
        .await?;

    let mut change_set = ctx.change_set()?.to_owned();
    change_set.update_pointer(ctx, legacy_addr).await?;

    ctx.update_snapshot_to_visibility().await?;

    let mut elapsed_total = Duration::ZERO;
    for id in legacy_components.iter().take(2) {
        let start = Instant::now();
        set_av_for_prop_for_component(ctx, *id).await?;
        let _ = ctx
            .workspace_snapshot()?
            .as_legacy_snapshot()?
            .write(ctx)
            .await?;
        elapsed_total += start.elapsed();
    }

    println!("elapsed total on legacy snapshot: {:?}", elapsed_total,);

    for split_max in [5_000, 10_000, 25_000, 50_000] {
        println!("\n\nnsplit_max: {}\n", split_max);
        let workspace_pk = WorkspacePk::new();
        let mut clone_ctx = ctx.clone();
        let clone_ctx_mut_ref = &mut clone_ctx;
        let _ = Workspace::new_split_graph_workspace(
            clone_ctx_mut_ref,
            workspace_pk,
            "split workspace",
            "token",
            split_max,
        )
        .await?;

        dal_test::test_exclusive_schemas::migrate(clone_ctx_mut_ref).await?;

        let view_id = View::get_id_for_default(clone_ctx_mut_ref).await?;

        let mut split_component_ids = vec![];
        for i in 0..COMPONENTS_COUNT {
            let id = create_component_for_default_schema_name(
                clone_ctx_mut_ref,
                "starfield",
                format!("{i}"),
                view_id,
            )
            .await?
            .id();
            split_component_ids.push(id);
            print!(".");
            std::io::stdout().flush().unwrap();
        }
        println!("");
        println!(
            "subgraph count: {}",
            clone_ctx_mut_ref
                .workspace_snapshot()?
                .as_split_snapshot()?
                .subgraph_count()
                .await
        );

        // Creating all those components in one batch adds thousands of dvu roots, but that's anomalous
        DependentValueRoot::take_dependent_values(clone_ctx_mut_ref).await?;

        let split_graph = clone_ctx_mut_ref
            .workspace_snapshot()?
            .as_split_snapshot()?;

        let address = split_graph.write(clone_ctx_mut_ref).await?;

        let mut change_set = clone_ctx_mut_ref.change_set()?.to_owned();
        change_set
            .update_pointer(clone_ctx_mut_ref, address)
            .await?;
        clone_ctx_mut_ref.update_snapshot_to_visibility().await?;

        let mut elapsed_total = Duration::ZERO;
        for id in split_component_ids.iter().take(1) {
            let start = Instant::now();
            set_av_for_prop_for_component(clone_ctx_mut_ref, *id).await?;
            println!("set av for prop in {:?}", start.elapsed());
            let _ = clone_ctx_mut_ref
                .workspace_snapshot()?
                .as_split_snapshot()?
                .write(clone_ctx_mut_ref)
                .await?;
            elapsed_total += start.elapsed();
            println!("elapsed: {:?}", start.elapsed());
        }
    }

    Ok(())
}
