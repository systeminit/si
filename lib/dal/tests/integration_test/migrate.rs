use std::collections::VecDeque;

use dal::{
    AttributeValue,
    Component,
    DalContext,
    Prop,
    PropKind,
    diagram::view::View,
    prop::PropPath,
    workspace::SnapshotVersion,
    workspace_snapshot::split_snapshot::{
        SubGraphVersionDiscriminants,
        SuperGraphVersionDiscriminants,
    },
};
use dal_test::{
    Result,
    helpers::create_component_for_default_schema_name,
    test,
};

#[test]
#[ignore]
async fn migrate_to_split_graph(ctx: &mut DalContext) -> Result<()> {
    const COMPONENTS_COUNT: usize = 5;
    let view_id = View::get_id_for_default(ctx).await?;

    for i in 0..COMPONENTS_COUNT {
        let _ = create_component_for_default_schema_name(ctx, "starfield", format!("{i}"), view_id)
            .await?
            .id();
    }

    ctx.commit().await?;

    ctx.update_snapshot_to_visibility().await?;

    let snapshot_address = ctx.workspace_snapshot()?.id().await;
    let mut migrator = dal::workspace_snapshot::migrator::SnapshotGraphMigrator::new();

    let new_address = migrator.migrate_snapshot(ctx, snapshot_address).await?;

    let mut change_set = ctx.change_set()?.clone();
    change_set.update_pointer(ctx, new_address).await?;
    ctx.set_change_set(change_set)?;
    let mut workspace = ctx.get_workspace().await?;
    workspace
        .set_snapshot_versions(
            ctx,
            SnapshotVersion::Split(SuperGraphVersionDiscriminants::V1),
            Some(SubGraphVersionDiscriminants::V1),
        )
        .await?;

    ctx.update_snapshot_to_visibility().await?;
    assert!(ctx.workspace_snapshot()?.as_split_snapshot().is_ok());

    for component in Component::list(ctx).await? {
        component.view(ctx).await?;
        let schema_variant_id = Component::schema_variant_id(ctx, component.id()).await?;
        let mut prop_stack = VecDeque::new();
        let root_prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(["root"])).await?;
        prop_stack.push_back(root_prop_id);

        while let Some(prop_id) = prop_stack.pop_front() {
            let prop = Prop::get_by_id(ctx, prop_id).await?;
            match prop.kind {
                PropKind::Array | PropKind::Map | PropKind::Object => {
                    let prop_children = Prop::direct_child_prop_ids_ordered(ctx, prop_id).await?;
                    prop_stack.extend(prop_children);

                    let avs = Component::attribute_values_for_prop_id(ctx, component.id(), prop_id)
                        .await?;
                    for av_id in avs {
                        assert!(
                            AttributeValue::get_child_av_ids_in_order(ctx, av_id)
                                .await
                                .is_ok()
                        );
                    }
                }
                _ => continue,
            }
        }
    }

    Ok(())
}
