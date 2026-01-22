use base64::Engine;
use dal::{
    AttributeValue,
    Component,
    DalContext,
    Func,
    Schema,
    action::{
        Action,
        dependency_graph::ActionDependencyGraph,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
};
use dal_test::{
    Result,
    helpers::create_component_for_default_schema_name_in_default_view,
    prelude::ChangeSetTestHelpers,
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test(enable_veritech)]
async fn schema_level_action_prototype(ctx: &mut DalContext) -> Result<()> {
    let schema = Schema::get_by_name(ctx, "swifty").await?;
    let default_schema_variant_id = Schema::default_variant_id(ctx, schema.id()).await?;

    let create_action_code = "async function main() {
                return { payload: { \"when you're\": \"strange\"}, status: \"ok\" };
            }";

    let create_func = Func::new(
        ctx,
        "test:schemaCreateActionSwifty",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        dal::FuncBackendKind::JsAction,
        dal::FuncBackendResponseType::Action,
        "main".into(),
        Some(base64::engine::general_purpose::STANDARD_NO_PAD.encode(create_action_code)),
        false,
    )
    .await?;

    let schema_action_prototype = ActionPrototype::new(
        ctx,
        ActionKind::Create,
        "test:schemaCreateActionSwifty".into(),
        None,
        schema.id(),
        create_func.id,
    )
    .await?;

    let default_create = ActionPrototype::find_by_kind_for_schema_or_variant(
        ctx,
        ActionKind::Create,
        default_schema_variant_id,
    )
    .await?
    .pop()
    .expect("should have one!");

    assert_eq!(schema_action_prototype.id(), default_create.id());

    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "taylor kelce")
            .await?;

    let mut actions = Action::find_for_component_id(ctx, component.id())
        .await
        .expect("should be able to find actions");

    assert_eq!(1, actions.len());

    let action_id = actions.pop().expect("should have one action");

    let prototype_id = Action::prototype_id(ctx, action_id).await?;
    let func_id = ActionPrototype::func_id(ctx, prototype_id).await?;

    assert_eq!(schema_action_prototype.id(), prototype_id);
    assert_eq!(create_func.id, func_id);

    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;
    assert!(action_graph.contains_value(action_id));

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    let component_on_head = Component::get_by_id(ctx, component.id()).await?;
    let payload_id = component_on_head
        .attribute_values_for_prop(ctx, &["root", "resource", "payload"])
        .await?
        .pop()
        .expect("vivre");

    let payload = AttributeValue::view(ctx, payload_id)
        .await?
        .expect("a value should exist");

    assert_eq!(serde_json::json!({"when you're": "strange"}), payload);

    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    // Now, we remove the schema level prototype and confirm the variant level one takes precedence
    ActionPrototype::remove(ctx, schema_action_prototype.id()).await?;

    let schema_variant_create_id = ActionPrototype::for_variant(ctx, default_schema_variant_id)
        .await?
        .into_iter()
        .filter(|proto| proto.kind == ActionKind::Create)
        .map(|proto| proto.id())
        .next()
        .expect("should have a create");

    let default_create = ActionPrototype::find_by_kind_for_schema_or_variant(
        ctx,
        ActionKind::Create,
        default_schema_variant_id,
    )
    .await?
    .pop()
    .expect("should have one!");

    assert_eq!(schema_variant_create_id, default_create.id());

    let second =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "1984").await?;

    let mut actions = Action::find_for_component_id(ctx, second.id())
        .await
        .expect("should be able to find actions");

    assert_eq!(1, actions.len());

    let action_id = actions.pop().expect("should have one action");

    let prototype_id = Action::prototype_id(ctx, action_id).await?;

    assert_eq!(schema_variant_create_id, prototype_id);

    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;
    assert!(action_graph.contains_value(action_id));

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    Ok(())
}
