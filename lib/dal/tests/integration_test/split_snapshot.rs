use std::time::Instant;

use dal::{diagram::view::View, DalContext, Workspace, WorkspacePk};
use dal_test::{helpers::create_component_for_default_schema_name, test};
//use pretty_assertions_sorted::assert_eq;

#[test]
async fn create_split_snapshot_workspace(ctx: &mut DalContext) {
    let view_id = View::get_id_for_default(ctx)
        .await
        .expect("get default view");

    dbg!(&view_id);

    let start = Instant::now();
    for i in 0..1000 {
        dbg!(&i, ctx.workspace_snapshot().unwrap().subgraph_count().await);
        create_component_for_default_schema_name(ctx, "starfield", format!("{i}"), view_id)
            .await
            .expect("comp");
    }
    dbg!("1000 components legacy graph", start.elapsed());

    let workspace_pk = WorkspacePk::new();
    let workspace =
        Workspace::new_split_graph_workspace(ctx, workspace_pk, "split workspace", "token")
            .await
            .expect("failed to create new split graph workspace");

    dal_test::test_exclusive_schemas::migrate(ctx)
        .await
        .expect("failed to migrate");

    dbg!("migrated");

    let view_id = View::get_id_for_default(ctx)
        .await
        .expect("get default view");

    dbg!(&view_id);

    let start = Instant::now();
    for i in 0..1000 {
        dbg!(&i, ctx.workspace_snapshot().unwrap().subgraph_count().await);
        create_component_for_default_schema_name(ctx, "starfield", format!("{i}"), view_id)
            .await
            .expect("comp");
    }
    dbg!("1000 components split graph", start.elapsed());
}
