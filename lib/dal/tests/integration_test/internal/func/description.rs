use dal::func::description::{FuncDescription, FuncDescriptionContents};
use dal::{
    func::argument::{FuncArgument, FuncArgumentKind},
    DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncId, StandardModel,
};
use dal_test::test;
use dal_test::test_harness::{create_schema, create_schema_variant};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &DalContext) {
    let (func_id, response_type) = create_confirmation_func(ctx).await;
    let schema = create_schema(ctx).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;

    let contents = FuncDescriptionContents::Confirmation {
        name: "TODD HOWARD".to_string(),
        success_description: None,
        failure_description: None,
        provider: None,
    };

    let description = FuncDescription::new(ctx, func_id, *schema_variant.id(), contents.clone())
        .await
        .expect("could not create description");

    assert_eq!(func_id, *description.func_id());
    assert_eq!(schema_variant.id(), description.schema_variant_id());
    assert_eq!(
        contents,
        description
            .deserialized_contents()
            .expect("could not deserialize contents")
    );
    assert_eq!(contents.response_type(), *description.response_type());
    assert_eq!(contents.response_type(), response_type);
}

async fn create_confirmation_func(ctx: &DalContext) -> (FuncId, FuncBackendResponseType) {
    let func_backend_response_type = FuncBackendResponseType::Confirmation;
    let mut confirmation_func = Func::new(
        ctx,
        "test:confirmation",
        FuncBackendKind::JsAttribute,
        func_backend_response_type,
    )
    .await
    .expect("could not create func");

    let code = "async function exists(input) {
        if (!input.resource?.value) {
            return {
                success: false,
                recommendedActions: [\"create\"]
            }
        }
        return {
            success: true,
            recommendedActions: [],
        }
    }";

    confirmation_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    confirmation_func
        .set_handler(ctx, Some("exists"))
        .await
        .expect("set handler");

    let _confirmation_func_argument = FuncArgument::new(
        ctx,
        "resource",
        FuncArgumentKind::String,
        None,
        *confirmation_func.id(),
    )
    .await
    .expect("could not create func argument");

    (*confirmation_func.id(), func_backend_response_type)
}
