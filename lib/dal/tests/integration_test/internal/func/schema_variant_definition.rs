use dal::{DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncBinding, StandardModel};
use dal_test::test;

#[test]
async fn execute_schema_variant_definition(ctx: &DalContext) {
    let mut func = Func::new(
        ctx,
        "schemaVariantDefinition",
        FuncBackendKind::JsSchemaVariantDefinition,
        FuncBackendResponseType::SchemaVariantDefinition,
    )
    .await
    .expect("create func");

    func.set_handler(ctx, Some("asset"))
        .await
        .expect("able to set func handler");

    let code = "function asset() {
            return {
                props: [
                    {
                        kind: 'string',
                        name: 'string_prop'
                    }
                ],
                inputSockets: [],
                outputSockets: [],
            }
        }
        ";
    func.set_code_plaintext(ctx, Some(code))
        .await
        .expect("able to set function code");

    let (_, return_value) =
        FuncBinding::create_and_execute(ctx, serde_json::Value::Null, *func.id())
            .await
            .expect("able to execute");

    assert_eq!(
        return_value.value(),
        Some(&serde_json::json!({
            "definition": {
                "props": [
                    {
                        "kind": "string",
                        "name": "string_prop",
                    }
                ],
                "inputSockets": [],
                "outputSockets": [],
            },
            "error": serde_json::Value::Null
        }))
    );
}
