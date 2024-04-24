use dal::code_view::CodeLanguage;
use dal::{Component, DalContext};
use dal_test::test;
use dal_test::test_harness::create_component_for_schema_name;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn get_code_json_lang(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    let (codegen_view, has_code) = Component::list_code_generated(ctx, component.id())
        .await
        .expect("unable to get codegen views");

    assert_eq!(codegen_view.len(), 1);
    assert!(has_code, "true");

    // This is safe as we would have failed the above test otherwise
    let codegen = codegen_view.clone().pop().unwrap();

    assert_eq!(codegen.language, CodeLanguage::Json,);
    assert_eq!(codegen.func, Some("test:generateCode".to_string()));
    assert_eq!(codegen.message, None);
    assert_eq!(
        codegen.code,
        Some("{\n  \"name\": \"shake it off\"\n}".to_string())
    );
}

#[test]
async fn get_code_yaml_and_string(ctx: &mut DalContext) {
    let component =
        create_component_for_schema_name(ctx, "katy perry", "all codegen and no actions").await;

    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    let (codegen_view, has_code) = Component::list_code_generated(ctx, component.id())
        .await
        .expect("unable to get codegen views");

    assert_eq!(codegen_view.len(), 2);
    assert!(has_code, "true");

    let string_codegen = codegen_view
        .iter()
        .find(|&f| f.func == Some("test:generateStringCode".to_string()))
        .expect("Unable to find string codegen func");

    assert_eq!(string_codegen.language, CodeLanguage::String);
    assert_eq!(string_codegen.message, None);
    assert_eq!(string_codegen.code, Some("poop canoe".to_string()));

    let yaml_codegen = codegen_view
        .iter()
        .find(|&f| f.func == Some("test:generateYamlCode".to_string()))
        .expect("Unable to find yaml codegen func");

    assert_eq!(yaml_codegen.language, CodeLanguage::Yaml);
    assert_eq!(yaml_codegen.message, None);
    assert_eq!(
        yaml_codegen.code,
        Some("name: all codegen and no actions\n".to_string())
    );
}

#[test]
async fn get_code_no_codegen_funcs(ctx: &mut DalContext) {
    let starfield_component =
        create_component_for_schema_name(ctx, "starfield", "no codegen funcs here").await;
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    let (codegen_view, has_code) = Component::list_code_generated(ctx, starfield_component.id())
        .await
        .expect("unable to get codegen views");

    assert!(codegen_view.is_empty());
    assert_eq!(has_code, false);
}
