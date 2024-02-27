use si_pkg::{
    ActionFuncSpec, AttrFuncInputSpec, AttrFuncInputSpecKind, FuncArgumentSpec, FuncSpec,
    FuncSpecBackendKind, FuncSpecBackendResponseType, PkgSpec, PropSpec, SchemaSpec,
    SchemaVariantSpec, SchemaVariantSpecData, SiPkg, SocketSpec, SocketSpecData, SocketSpecKind,
};
use si_pkg::{FuncSpecData, SchemaSpecData};

use crate::func::argument::FuncArgumentKind;
use crate::func::intrinsics::IntrinsicFunc;
use crate::pkg::import_pkg_from_pkg;
use crate::{prop::PropPath, ActionKind, PropKind};
use crate::{BuiltinsResult, DalContext};

pub async fn migrate_test_exclusive_schema_fallout(ctx: &DalContext) -> BuiltinsResult<()> {
    let mut fallout_builder = PkgSpec::builder();

    fallout_builder
        .name("fallout")
        .version("2023-05-23")
        .created_by("System Initiative");

    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let fallout_create_action_code = "async function create() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";
    let fn_name = "test:createActionFallout";
    let fallout_create_action_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(fallout_create_action_code)
                .handler("create")
                .backend_kind(FuncSpecBackendKind::JsAction)
                .response_type(FuncSpecBackendResponseType::Action)
                .build()?,
        )
        .build()?;

    let fallout_scaffold_func = "function createAsset() {\
                return new AssetBuilder().build();
            }";
    let fn_name = "test:scaffoldFalloutAsset";
    let fallout_authoring_schema_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(fallout_scaffold_func)
                .handler("createAsset")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()?,
        )
        .build()?;

    let fallout_resource_payload_to_value_func_code =
        "async function translate(arg: Input): Promise<Output> {\
            return arg.payload ?? {};
        }";
    let fn_name = "test:resourcePayloadToValue";
    let fallout_resource_payload_to_value_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(fallout_resource_payload_to_value_func_code)
                .handler("translate")
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Json)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("payload")
                .kind(FuncArgumentKind::Object)
                .build()?,
        )
        .build()?;

    let fallout_schema = SchemaSpec::builder()
        .name("fallout")
        .data(
            SchemaSpecData::builder()
                .name("fallout")
                .category("test exclusive")
                .category_name("fallout")
                .build()
                .expect("build schema spec data"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
                .unique_id("fallout_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .name("v0")
                        .color("#ffffff")
                        .func_unique_id(&fallout_authoring_schema_func.unique_id)
                        .build()
                        .expect("fallout variant data"),
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("name")
                        .kind(PropKind::String)
                        .func_unique_id(&identity_func_spec.unique_id)
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
                        .data(
                            SocketSpecData::builder()
                                .name("bethesda")
                                .connection_annotations(serde_json::to_string(&vec!["bethesda"])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
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
                        .data(
                            SocketSpecData::builder()
                                .name("fallout")
                                .connection_annotations(serde_json::to_string(&vec!["bethesda"])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new(["root"]))
                                .build()?,
                        )
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&ActionKind::Create)
                        .func_unique_id(&fallout_create_action_func.unique_id)
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let fallout_spec = fallout_builder
        .func(identity_func_spec)
        .func(fallout_create_action_func)
        .func(fallout_authoring_schema_func)
        .func(fallout_resource_payload_to_value_func)
        .schema(fallout_schema)
        .build()?;

    let fallout_pkg = SiPkg::load_from_spec(fallout_spec)?;
    // TODO(nick): decide what to do with override schema builtin featuee flag.
    import_pkg_from_pkg(
        ctx,
        &fallout_pkg,
        Some(crate::pkg::ImportOptions {
            schemas: Some(vec!["fallout".into()]),
            ..Default::default()
        }),
        true,
    )
    .await?;

    Ok(())
}
