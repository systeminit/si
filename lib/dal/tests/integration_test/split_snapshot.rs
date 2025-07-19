use std::time::Instant;

use dal::{
    AttributeValue,
    Component,
    ComponentId,
    DalContext,
    Prop,
    diagram::view::View,
    prop::PropPath,
    workspace_snapshot::DependentValueRoot,
};
use dal_test::{
    Result,
    helpers::create_component_for_default_schema_name,
    prelude::ChangeSetTestHelpers,
    test,
};
//use pretty_assertions_sorted::assert_eq;

#[allow(unused)]
async fn set_av_for_prop_for_component(ctx: &DalContext, component_id: ComponentId) -> Result<()> {
    let start = Instant::now();
    let variant_id = Component::schema_variant_id(ctx, component_id).await?;
    println!("got variant id in {:?}", start.elapsed());

    let start = Instant::now();
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
    println!("got prop id {:?}", start.elapsed());

    let start = Instant::now();
    let rigid_designator_values =
        Component::attribute_values_for_prop_id(ctx, component_id, rigid_designator_prop_id)
            .await?;
    println!("got av id in {:?}", start.elapsed());

    let rigid_designation = serde_json::json!("hesperus");

    let rigid_id = rigid_designator_values
        .first()
        .copied()
        .expect("should have a rigid_designator");

    let start = Instant::now();
    AttributeValue::update(ctx, rigid_id, Some(rigid_designation)).await?;
    println!("update in {:?}", start.elapsed());

    Ok(())
}

#[test]
#[ignore]
async fn create_split_snapshot_workspace(ctx: &mut DalContext) -> Result<()> {
    const COMPONENTS_COUNT: usize = 25;
    println!("test with {COMPONENTS_COUNT} components");

    let mut legacy_components = vec![];
    let view_id = View::get_id_for_default(ctx).await?;

    // dbg!(
    //     ctx.workspace_snapshot()?
    //         .as_split_snapshot()?
    //         .current_rebase_batch()
    //         .await?
    // );

    DependentValueRoot::take_dependent_values(ctx).await?;
    dbg!("done");
    dbg!(DependentValueRoot::get_dependent_value_roots(ctx).await)?;
    dbg!("here");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    // dbg!(
    //     ctx.workspace_snapshot()?
    //         .as_legacy_snapshot()?
    //         .current_rebase_batch()
    //         .await?
    // );
    // dbg!(
    //     ctx.workspace_snapshot()?
    //         .as_split_snapshot()?
    //         .current_rebase_batch()
    //         .await?
    // );

    for i in 0..COMPONENTS_COUNT {
        let id =
            create_component_for_default_schema_name(ctx, "starfield", format!("{i}"), view_id)
                .await?
                .id();
        legacy_components.push(id);
        dbg!(id);
    }
    // dbg!(
    //     ctx.workspace_snapshot()?
    //         .as_legacy_snapshot()?
    //         .current_rebase_batch()
    //         .await?
    // );
    // dbg!(
    //     ctx.workspace_snapshot()?
    //         .as_split_snapshot()?
    // );

    // Creating all those components in one batch adds thousands of dvu roots, but that's anomalous
    DependentValueRoot::take_dependent_values(ctx).await?;
    dbg!(DependentValueRoot::get_dependent_value_roots(ctx).await)?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // for component_id in legacy_components {
    //     let component = Component::get_by_id(ctx, component_id).await?;
    // }

    Ok(())

    // let legacy_addr = ctx
    //     .workspace_snapshot()?
    //     .as_legacy_snapshot()?
    //     .write(ctx)
    //     .await?;

    // let mut change_set = ctx.change_set()?.to_owned();
    // change_set.update_pointer(ctx, legacy_addr).await?;

    // ctx.update_snapshot_to_visibility().await?;

    // println!("\nTEST LEGACY\n");

    // let id = legacy_components.get(555).copied().unwrap();
    // let start = Instant::now();
    // println!("set av prop legacy");
    // set_av_for_prop_for_component(ctx, id).await?;
    // println!("set av legacy took: {:?}", start.elapsed());
    // let write_start = Instant::now();
    // let _ = ctx
    //     .workspace_snapshot()?
    //     .as_legacy_snapshot()?
    //     .write(ctx)
    //     .await?;
    // println!("legacy write took: {:?}", write_start.elapsed());

    // for split_max in [5_000, 10_000, 25_000, 50_000] {
    //     println!("\n\nnplit_max: {}\n", split_max);
    //     let workspace_pk = WorkspacePk::new();
    //     let mut clone_ctx = ctx.clone();
    //     let clone_ctx_mut_ref = &mut clone_ctx;
    //     let _ = Workspace::new_split_graph_workspace(
    //         clone_ctx_mut_ref,
    //         workspace_pk,
    //         "split workspace",
    //         "token",
    //         split_max,
    //     )
    //     .await?;

    //     dal_test::test_exclusive_schemas::migrate(clone_ctx_mut_ref).await?;

    //     let view_id = View::get_id_for_default(clone_ctx_mut_ref).await?;

    //     let mut split_component_ids = vec![];
    //     for i in 0..COMPONENTS_COUNT {
    //         let id = create_component_for_default_schema_name(
    //             clone_ctx_mut_ref,
    //             "starfield",
    //             format!("{i}"),
    //             view_id,
    //         )
    //         .await?
    //         .id();
    //         split_component_ids.push(id);
    //         print!(".");
    //         std::io::stdout().flush().unwrap();
    //     }
    //     println!("");
    //     println!(
    //         "subgraph count: {}",
    //         clone_ctx_mut_ref
    //             .workspace_snapshot()?
    //             .as_split_snapshot()?
    //             .subgraph_count()
    //             .await
    //     );

    //     // Creating all those components in one batch adds thousands of dvu roots, but that's anomalous
    //     DependentValueRoot::take_dependent_values(clone_ctx_mut_ref).await?;

    //     let split_graph = clone_ctx_mut_ref
    //         .workspace_snapshot()?
    //         .as_split_snapshot()?;

    //     let address = split_graph.write(clone_ctx_mut_ref).await?;

    //     let mut change_set = clone_ctx_mut_ref.change_set()?.to_owned();
    //     change_set
    //         .update_pointer(clone_ctx_mut_ref, address)
    //         .await?;
    //     clone_ctx_mut_ref.update_snapshot_to_visibility().await?;

    //     println!("\nTEST\n");

    //     let mut elapsed_total = Duration::ZERO;
    //     let id = split_component_ids.get(555).copied().unwrap();
    //     let start = Instant::now();
    //     set_av_for_prop_for_component(clone_ctx_mut_ref, id).await?;
    //     println!("set av for prop in {:?}", start.elapsed());
    //     let write_start = Instant::now();
    //     let _ = clone_ctx_mut_ref
    //         .workspace_snapshot()?
    //         .as_split_snapshot()?
    //         .write(clone_ctx_mut_ref)
    //         .await?;
    //     elapsed_total += start.elapsed();
    //     println!("elapsed: {:?}", write_start.elapsed());
    // }

    // Ok(())
}
