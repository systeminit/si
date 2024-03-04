use si_pkg::SchemaSpecData;
use si_pkg::{
    ActionFuncSpec, AttrFuncInputSpec, AttrFuncInputSpecKind, FuncArgumentSpec, FuncSpec,
    FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData, PkgSpec, PropSpec, SchemaSpec,
    SchemaVariantSpec, SchemaVariantSpecData, SiPkg, SocketSpec, SocketSpecData, SocketSpecKind,
};

use crate::func::argument::FuncArgumentKind;
use crate::func::intrinsics::IntrinsicFunc;
use crate::pkg::import_pkg_from_pkg;
use crate::{prop::PropPath, ActionKind};
use crate::{BuiltinsResult, DalContext, PropKind};

pub async fn migrate_test_exclusive_schema_starfield(ctx: &DalContext) -> BuiltinsResult<()> {
    let mut starfield_builder = PkgSpec::builder();

    starfield_builder
        .name("starfield")
        .version("2023-05-23")
        .created_by("System Initiative");

    let identity_func_spec = IntrinsicFunc::Identity
        .to_spec()
        .expect("create identity func spec");

    let starfield_create_action_code = "async function create() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionStarfield";
    let starfield_create_action_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(starfield_create_action_code)
                .handler("create")
                .backend_kind(FuncSpecBackendKind::JsAction)
                .response_type(FuncSpecBackendResponseType::Action)
                .build()?,
        )
        .build()?;

    let starfield_refresh_action_code =
        "async function refresh(component: Input): Promise<Output> {
              return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:refreshActionStarfield";
    let starfield_refresh_action_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .handler("refresh")
                .code_plaintext(starfield_refresh_action_code)
                .backend_kind(FuncSpecBackendKind::JsAction)
                .response_type(FuncSpecBackendResponseType::Action)
                .build()?,
        )
        .build()?;

    let fallout_entries_to_galaxies_transform_code =
            "async function falloutEntriesToGalaxies(input: Input): Promise<Output> {
          let galaxies = [];
          let entries = input.entries;

          // Force the entries arg to be an Array (and return an empty array if the arg is absent/undefined/null).
          if (entries === undefined) return galaxies;
          if (entries === null) return galaxies;
          if (!Array.isArray(entries)) entries = [entries];

          entries.filter(i => i ?? false).forEach(function (entry) {

            let name = entry.si.name;
            let rads = entry.domain.rads;
            let galaxy = {
              sun: name + \"-sun\",
              planets: rads
            };

            galaxies.push(galaxy);
          });

          return galaxies;
        }";
    let fn_name = "test:falloutEntriesToGalaxies";
    let fallout_entries_to_galaxies_transform_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(fallout_entries_to_galaxies_transform_code)
                .handler("falloutEntriesToGalaxies")
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Array)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("entries")
                .kind(FuncArgumentKind::Array)
                .element_kind(Some(FuncArgumentKind::Object.into()))
                .build()?,
        )
        .build()?;

    let starfield_scaffold_func = "function createAsset() {\
                return new AssetBuilder().build();
            }";
    let fn_name = "test:scaffoldStarfieldAsset";
    let starfield_authoring_schema_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(starfield_scaffold_func)
                .handler("createAsset")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()?,
        )
        .build()?;

    let starfield_resource_payload_to_value_func_code =
        "async function translate(arg: Input): Promise<Output> {\
            return arg.payload ?? {};
        }";
    let fn_name = "test:resourcePayloadToValue";
    let starfield_resource_payload_to_value_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(starfield_resource_payload_to_value_func_code)
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

    let starfield_kripke_func_code = "async function hesperus_is_phosphorus(input) {
            if input.hesperus === \"hesperus\" { return \"phosphorus\"; }
            else return \"not hesperus\";
        }";
    let starfield_kripke_func_name = "hesperus_is_phosphorus";
    let starfield_kripke_func = FuncSpec::builder()
        .name(starfield_kripke_func_name)
        .unique_id(starfield_kripke_func_name)
        .data(
            FuncSpecData::builder()
                .name(starfield_kripke_func_name)
                .code_plaintext(starfield_kripke_func_code)
                .handler(starfield_kripke_func_name)
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::String)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("hesperus")
                .kind(FuncArgumentKind::String)
                .build()?,
        )
        .build()?;

    let starfield_schema = SchemaSpec::builder()
        .name("starfield")
        .data(
            SchemaSpecData::builder()
                .name("starfield")
                .category("test exclusive")
                .category_name("starfield")
                .build()
                .expect("schema spec data build"),
        )
        .variant(
            SchemaVariantSpec::builder()
                .name("v0")
                .unique_id("starfield_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .name("v0")
                        .color("#ffffff")
                        .func_unique_id(&starfield_authoring_schema_func.unique_id)
                        .build()
                        .expect("build variant spec data"),
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
                        .name("hidden_prop")
                        .kind(PropKind::String)
                        .hidden(true)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("freestar")
                        .kind(PropKind::String)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("attributes")
                        .kind(PropKind::String)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .name("identity")
                                .socket_name("bethesda")
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("possible_world_a")
                        .kind(PropKind::Object)
                        .entry(
                            PropSpec::builder()
                                .name("wormhole_1")
                                .kind(PropKind::Object)
                                .entry(
                                    PropSpec::builder()
                                        .name("wormhole_2")
                                        .kind(PropKind::Object)
                                        .entry(
                                            PropSpec::builder()
                                                .name("wormhole_3")
                                                .kind(PropKind::Object)
                                                .entry(
                                                    PropSpec::builder()
                                                        .kind(PropKind::String)
                                                        .name("rigid_designator")
                                                        .build()?,
                                                )
                                                .build()?,
                                        )
                                        .build()?,
                                )
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("possible_world_b")
                        .kind(PropKind::Object)
                        .entry(
                            PropSpec::builder()
                                .name("wormhole_1")
                                .kind(PropKind::Object)
                                .entry(
                                    PropSpec::builder()
                                        .name("wormhole_2")
                                        .kind(PropKind::Object)
                                        .entry(
                                            PropSpec::builder()
                                                .name("wormhole_3")
                                                .kind(PropKind::Object)
                                                .entry(
                                                    PropSpec::builder()
                                                        .kind(PropKind::String)
                                                        .name("naming_and_necessity")
                                                        .func_unique_id(
                                                            &starfield_kripke_func.unique_id,
                                                        )
                                                        .input(
                                                            AttrFuncInputSpec::builder()
                                                                .kind(AttrFuncInputSpecKind::Prop)
                                                                .name("hesperus")
                                                                .prop_path(PropPath::new([
                                                                    "root",
                                                                    "domain",
                                                                    "possible_world_a",
                                                                    "wormhole_1",
                                                                    "wormhole_2",
                                                                    "wormhole_3",
                                                                    "rigid_designator",
                                                                ]))
                                                                .build()?,
                                                        )
                                                        .build()?,
                                                )
                                                .build()?,
                                        )
                                        .build()?,
                                )
                                .build()?,
                        )
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("universe")
                        .kind(PropKind::Object)
                        .entry(
                            PropSpec::builder()
                                .name("galaxies")
                                .kind(PropKind::Array)
                                .func_unique_id(
                                    &fallout_entries_to_galaxies_transform_func.unique_id,
                                )
                                .input(
                                    AttrFuncInputSpec::builder()
                                        .kind(AttrFuncInputSpecKind::InputSocket)
                                        .name("entries")
                                        .socket_name("fallout")
                                        .build()?,
                                )
                                .type_prop(
                                    PropSpec::builder()
                                        .name("galaxy")
                                        .kind(PropKind::Object)
                                        .entry(
                                            PropSpec::builder()
                                                .name("sun")
                                                .kind(PropKind::String)
                                                .build()?,
                                        )
                                        .entry(
                                            PropSpec::builder()
                                                .name("planets")
                                                .kind(PropKind::Integer)
                                                .build()?,
                                        )
                                        .build()?,
                                )
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("bethesda")
                        .data(
                            SocketSpecData::builder()
                                .name("bethesda")
                                .connection_annotations(serde_json::to_string(&vec!["bethesda"])?)
                                .kind(SocketSpecKind::Input)
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
                                .connection_annotations(serde_json::to_string(&vec!["fallout"])?)
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&ActionKind::Create)
                        .func_unique_id(&starfield_create_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(&ActionKind::Refresh)
                        .func_unique_id(&starfield_refresh_action_func.unique_id)
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let starfield_spec = starfield_builder
        .func(identity_func_spec)
        .func(starfield_refresh_action_func)
        .func(starfield_create_action_func)
        .func(fallout_entries_to_galaxies_transform_func)
        .func(starfield_authoring_schema_func)
        .func(starfield_resource_payload_to_value_func)
        .func(starfield_kripke_func)
        .schema(starfield_schema)
        .build()?;

    let starfield_pkg = SiPkg::load_from_spec(starfield_spec)?;
    import_pkg_from_pkg(
        ctx,
        &starfield_pkg,
        Some(crate::pkg::ImportOptions {
            schemas: Some(vec!["starfield".into()]),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}
