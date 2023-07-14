use si_pkg::{
    ActionFuncSpec, AttrFuncInputSpec, AttrFuncInputSpecKind, FuncArgumentSpec, FuncSpec,
    FuncSpecBackendKind, FuncSpecBackendResponseType, LeafFunctionSpec, PkgSpec, PropSpec,
    SchemaSpec, SchemaVariantSpec, SiPkg, SocketSpec, SocketSpecKind, ValidationSpec,
    ValidationSpecKind,
};

use crate::func::argument::FuncArgumentKind;
use crate::func::intrinsics::IntrinsicFunc;
use crate::pkg::import_pkg_from_pkg;
use crate::schema::variant::leaves::LeafInputLocation;
use crate::schema::variant::leaves::LeafKind;
use crate::{builtins::schema::MigrationDriver, prop::PropPath, ActionKind, PropKind};
use crate::{BuiltinsResult, DalContext};

impl MigrationDriver {
    pub async fn migrate_test_exclusive_fallout(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let mut fallout_builder = PkgSpec::builder();

        fallout_builder
            .name("fallout")
            .version("2023-05-23")
            .created_by("System Initiative");

        let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

        let fallout_confirmation_code = "async function exists(input) {
            if (!input.resource?.payload) {
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
        let fallout_confirmation_func = FuncSpec::builder()
            .name("test:confirmationFallout")
            .code_plaintext(fallout_confirmation_code)
            .backend_kind(FuncSpecBackendKind::JsAttribute)
            .response_type(FuncSpecBackendResponseType::Confirmation)
            .handler("exists")
            .argument(
                FuncArgumentSpec::builder()
                    .name("resource")
                    .kind(FuncArgumentKind::String)
                    .build()?,
            )
            .build()?;

        let fallout_create_action_code = "async function create() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";
        let fallout_create_action_func = FuncSpec::builder()
            .name("test:createActionFallout")
            .code_plaintext(fallout_create_action_code)
            .handler("create")
            .backend_kind(FuncSpecBackendKind::JsAction)
            .response_type(FuncSpecBackendResponseType::Action)
            .build()?;

        let fallout_scaffold_func = "function createAsset() {\
                return new AssetBuilder().build();
            }";
        let fallout_authoring_schema_func = FuncSpec::builder()
            .name("test:scaffoldFalloutAsset")
            .code_plaintext(fallout_scaffold_func)
            .handler("createAsset")
            .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
            .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
            .build()?;

        let fallout_resource_payload_to_value_func_code =
            "async function translate(arg: Input): Promise<Output> {\
            return arg.payload ?? {};
        }";
        let fallout_resource_payload_to_value_func = FuncSpec::builder()
            .name("si:resourcePayloadToValue")
            .code_plaintext(fallout_resource_payload_to_value_func_code)
            .handler("translate")
            .backend_kind(FuncSpecBackendKind::JsAttribute)
            .response_type(FuncSpecBackendResponseType::Json)
            .argument(
                FuncArgumentSpec::builder()
                    .name("payload")
                    .kind(FuncArgumentKind::Object)
                    .build()?,
            )
            .build()?;

        let fallout_schema = SchemaSpec::builder()
            .name("fallout")
            .category("test exclusive")
            .category_name("fallout")
            .variant(
                SchemaVariantSpec::builder()
                    .color("#ffffff")
                    .name("v0")
                    .func_unique_id(fallout_authoring_schema_func.unique_id)
                    .domain_prop(
                        PropSpec::builder()
                            .name("name")
                            .kind(PropKind::String)
                            .func_unique_id(identity_func_spec.unique_id)
                            .input(
                                AttrFuncInputSpec::builder()
                                    .kind(AttrFuncInputSpecKind::Prop)
                                    .name("identity")
                                    .prop_path(PropPath::new(["root", "si", "name"]))
                                    .build()?,
                            )
                            .build()?,
                    )
                    .domain_prop(
                        PropSpec::builder()
                            .name("special")
                            .kind(PropKind::String)
                            .build()?,
                    )
                    .domain_prop(
                        PropSpec::builder()
                            .name("rads")
                            .kind(PropKind::Integer)
                            .validation(
                                ValidationSpec::builder()
                                    .kind(ValidationSpecKind::IntegerIsBetweenTwoIntegers)
                                    .upper_bound(1001)
                                    .lower_bound(-1)
                                    .build()?,
                            )
                            .validation(
                                ValidationSpec::builder()
                                    .kind(ValidationSpecKind::IntegerIsNotEmpty)
                                    .build()?,
                            )
                            .build()?,
                    )
                    .domain_prop(
                        PropSpec::builder()
                            .name("active")
                            .kind(PropKind::Boolean)
                            .default_value(serde_json::json!(true))
                            .build()?,
                    )
                    .socket(
                        SocketSpec::builder()
                            .name("bethesda")
                            .kind(SocketSpecKind::Output)
                            .func_unique_id(identity_func_spec.unique_id)
                            .input(
                                AttrFuncInputSpec::builder()
                                    .name("identity")
                                    .kind(AttrFuncInputSpecKind::Prop)
                                    .prop_path(PropPath::new(["root", "domain", "special"]))
                                    .build()?,
                            )
                            .build()?,
                    )
                    .socket(
                        SocketSpec::builder()
                            .name("fallout")
                            .kind(SocketSpecKind::Output)
                            .func_unique_id(identity_func_spec.unique_id)
                            .input(
                                AttrFuncInputSpec::builder()
                                    .name("identity")
                                    .kind(AttrFuncInputSpecKind::Prop)
                                    .prop_path(PropPath::new(["root"]))
                                    .build()?,
                            )
                            .build()?,
                    )
                    .leaf_function(
                        LeafFunctionSpec::builder()
                            .leaf_kind(LeafKind::Confirmation)
                            .func_unique_id(fallout_confirmation_func.unique_id)
                            .inputs(vec![LeafInputLocation::Resource.into()])
                            .build()?,
                    )
                    .action_func(
                        ActionFuncSpec::builder()
                            .kind(&ActionKind::Create)
                            .func_unique_id(fallout_create_action_func.unique_id)
                            .build()?,
                    )
                    .build()?,
            )
            .build()?;

        let fallout_spec = fallout_builder
            .func(identity_func_spec)
            .func(fallout_create_action_func)
            .func(fallout_confirmation_func)
            .func(fallout_authoring_schema_func)
            .func(fallout_resource_payload_to_value_func)
            .schema(fallout_schema)
            .build()?;

        let fallout_pkg = SiPkg::load_from_spec(fallout_spec)?;
        import_pkg_from_pkg(
            ctx,
            &fallout_pkg,
            "test:fallout",
            Some(crate::pkg::ImportOptions {
                schemas: Some(vec!["fallout".into()]),
                ..Default::default()
            }),
        )
        .await?;

        Ok(())
    }
}
