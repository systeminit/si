mod batch;
mod runner;
mod with_secret;
mod with_update;

use dal::{
    Component, DalContext, DeprecatedAction, DeprecatedActionKind, DeprecatedActionPrototype,
    InputSocket, OutputSocket,
};
use dal_test::test;
use dal_test::test_harness::create_component_for_schema_name;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn prototype(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let mut action = None;
    let mut prototype = None;
    for proto in DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if proto.kind == DeprecatedActionKind::Create {
            action = Some(
                DeprecatedAction::upsert(ctx, proto.id, component.id())
                    .await
                    .expect("unable to upsert action"),
            );
            prototype = Some(proto);
            break;
        }
    }

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    assert_eq!(
        action
            .expect("no action found")
            .prototype(ctx)
            .await
            .expect("unable to find component"),
        prototype.expect("unable to find prototype")
    );
}

#[test]
async fn component(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let mut action = None;
    for prototype in DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if prototype.kind == DeprecatedActionKind::Create {
            action = Some(
                DeprecatedAction::upsert(ctx, prototype.id, component.id())
                    .await
                    .expect("unable to upsert action"),
            );
            break;
        }
    }

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    assert_eq!(
        action
            .expect("no action found")
            .component(ctx)
            .await
            .expect("unable to find component"),
        component
    );
}

#[test]
async fn get_by_id(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let mut action = None;
    for prototype in DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if prototype.kind == DeprecatedActionKind::Create {
            action = Some(
                DeprecatedAction::upsert(ctx, prototype.id, component.id())
                    .await
                    .expect("unable to upsert action"),
            );
            break;
        }
    }

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    let action = action.expect("no action found");
    assert_eq!(
        DeprecatedAction::get_by_id(ctx, action.id)
            .await
            .expect("unable to get action"),
        action
    );
}

#[test]
async fn delete(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    for prototype in DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if prototype.kind == DeprecatedActionKind::Create {
            DeprecatedAction::upsert(ctx, prototype.id, component.id())
                .await
                .expect("unable to upsert action");
            break;
        }
    }

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    let graph = DeprecatedAction::build_graph(ctx)
        .await
        .expect("unable to build graph");

    assert_eq!(graph.len(), 1);
    assert!(
        graph.values().next().expect("no graph value found").kind == DeprecatedActionKind::Create
    );

    graph
        .values()
        .next()
        .expect("no graph value found")
        .action
        .clone()
        .delete(ctx)
        .await
        .expect("unable to delete action");

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    let graph = DeprecatedAction::build_graph(ctx)
        .await
        .expect("unable to build graph");

    assert!(graph.is_empty());
}

#[test]
async fn for_component(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let mut actions = Vec::new();
    for prototype in DeprecatedActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if prototype.kind == DeprecatedActionKind::Create {
            actions.push(
                DeprecatedAction::upsert(ctx, prototype.id, component.id())
                    .await
                    .expect("unable to upsert action"),
            );
        }
    }

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    let list = DeprecatedAction::for_component(ctx, component.id())
        .await
        .expect("unable to list actions for component");
    assert_eq!(list, actions);
}

#[test]
async fn build_graph(ctx: &mut DalContext) {
    let source = create_component_for_schema_name(ctx, "fallout", "source").await;
    let source_sv_id = Component::schema_variant_id(ctx, source.id())
        .await
        .expect("find variant id for component");
    let mut source_action = None;

    for prototype in DeprecatedActionPrototype::for_variant(ctx, source_sv_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if prototype.kind == DeprecatedActionKind::Create {
            source_action = Some(
                DeprecatedAction::upsert(ctx, prototype.id, source.id())
                    .await
                    .expect("unable to upsert action"),
            );
            break;
        }
    }

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    let graph = DeprecatedAction::build_graph(ctx)
        .await
        .expect("unable to build graph");
    assert_eq!(graph.len(), 1);

    let destination = create_component_for_schema_name(ctx, "starfield", "destination").await;
    let destination_sv_id = Component::schema_variant_id(ctx, destination.id())
        .await
        .expect("find variant id for component");
    let mut destination_action = None;

    for prototype in DeprecatedActionPrototype::for_variant(ctx, destination_sv_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if prototype.kind == DeprecatedActionKind::Create {
            destination_action = Some(
                DeprecatedAction::upsert(ctx, prototype.id, destination.id())
                    .await
                    .expect("unable to upsert action"),
            );
            break;
        }
    }
    assert!(destination_action.is_some());

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    let graph = DeprecatedAction::build_graph(ctx)
        .await
        .expect("unable to build graph");
    assert_eq!(graph.len(), 2);
    assert!(graph.into_iter().all(|(_, v)| v.parents.is_empty()));

    let output_socket = OutputSocket::find_with_name(ctx, "bethesda", source_sv_id)
        .await
        .expect("could not perform find output socket")
        .expect("output socket not found");

    let input_socket = InputSocket::find_with_name(ctx, "bethesda", destination_sv_id)
        .await
        .expect("could not perform find input socket")
        .expect("input socket not found");

    Component::connect(
        ctx,
        source.id(),
        output_socket.id(),
        destination.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components");

    let conflicts = ctx.blocking_commit().await.expect("blocking commit failed");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    let graph = DeprecatedAction::build_graph(ctx)
        .await
        .expect("unable to build graph");
    assert_eq!(graph.len(), 2);
    for action in graph.values() {
        let source_action = source_action.as_ref().expect("no source action available");
        if action.action.id == source_action.id {
            assert!(action.parents.is_empty());
        } else {
            assert_eq!(action.parents.len(), 1);
            assert_eq!(action.parents[0], source_action.id);
        }
    }
}
