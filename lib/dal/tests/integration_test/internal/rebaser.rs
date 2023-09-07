//! For all tests in this module, provide "SI_TEST_BUILTIN_SCHEMAS=none" as an environment variable.

use dal::change_set_pointer::ChangeSetPointer;
use dal::content::hash::ContentHash;
use dal::workspace_snapshot::node_weight::{ContentAddress, NodeWeight};
use dal::{DalContext, Tenancy, Visibility, WorkspacePk, WorkspaceSnapshot};
use dal_test::test;
use rebaser_client::Client;

#[test]
async fn simple_rebase(ctx: &mut DalContext) {
    ctx.update_visibility(Visibility::new_head(false));
    ctx.update_tenancy(Tenancy::new(WorkspacePk::NONE));
    let ctx = &ctx;

    let mut client = Client::new().await.expect("could not build client");

    let mut base_change_set = ChangeSetPointer::new(ctx, "main")
        .await
        .expect("could not create change set");
    let base_change_set = &mut base_change_set;
    let mut snapshot = WorkspaceSnapshot::initial(ctx, &base_change_set)
        .await
        .expect("could not create workspace snapshot");

    // Add a new node.
    snapshot
        .add_node(
            NodeWeight::new_content(
                base_change_set,
                base_change_set
                    .generate_ulid()
                    .expect("cannot generate ulid"),
                ContentAddress::Schema(ContentHash::from("lacy - olivia rodrigo")),
            )
            .expect("could not create node weight"),
        )
        .expect("could not add node");

    snapshot.write(ctx).await.expect("could not write snapshot");
    base_change_set
        .update_pointer(ctx, snapshot.id)
        .await
        .expect("could not update pointer");

    // Create another change set and update.
    let mut forked_change_set = ChangeSetPointer::new(ctx, "fork")
        .await
        .expect("could not create change set");
    let forked_change_set = &mut forked_change_set;
    snapshot
        .add_node(
            NodeWeight::new_content(
                forked_change_set,
                forked_change_set
                    .generate_ulid()
                    .expect("cannot generate ulid"),
                ContentAddress::Schema(ContentHash::from("i'm the one - victoria monét")),
            )
            .expect("could not create node weight"),
        )
        .expect("could not add node");
    snapshot.write(ctx).await.expect("could not write snapshot");
    forked_change_set
        .update_pointer(ctx, snapshot.id)
        .await
        .expect("could not update pointer");

    // Rebase!
    let response = client
        .send_management_open_change_set(base_change_set.id.into())
        .await
        .expect("could not send management");

    // TODO(nick): do something useful with this.
    dbg!(response);

    ctx.blocking_commit().await.expect("could not do this");

    let response = client
        .send_with_reply(
            base_change_set.id.into(),
            snapshot.id.into(),
            forked_change_set.id.into(),
        )
        .await
        .expect("could not send");

    // TODO(nick): do something useful with this.
    dbg!(response);

    // TODO(nick): move cleanup to the test harness.
    let _ = client
        .send_management_close_change_set(base_change_set.id.into())
        .await;
}
