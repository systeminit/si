use dal::action::prototype::ActionKind;
use dal::func::argument::FuncArgumentKind;
use dal::func::intrinsics::IntrinsicFunc;
use dal::pkg::{ImportOptions, import_pkg_from_pkg};
use dal::prop::PropPath;
use dal::{BuiltinsResult, DalContext, PropKind, SchemaId};
use si_pkg::{
    ActionFuncSpec, AttrFuncInputSpec, AttrFuncInputSpecKind, FuncArgumentSpec, FuncSpec,
    FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData, PkgSpec, PropSpec, SchemaSpec,
    SchemaVariantSpec, SchemaVariantSpecData, SiPkg, SocketSpec, SocketSpecData, SocketSpecKind,
};
use si_pkg::{SchemaSpecData, SocketSpecArity};

use crate::test_exclusive_schemas::{PKG_CREATED_BY, PKG_VERSION};

pub(crate) async fn migrate_test_exclusive_schema_starfield(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut starfield_builder = PkgSpec::builder();

    let schema_name = "starfield";

    starfield_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

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
          if (entries.length == 0) return galaxies;
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

    let starfield_kripke_func_code = "async function hesperus_is_phosphorus(input) {
            if (input.hesperus === \"hesperus\") { return \"phosphorus\"; }
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
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(schema_name)
                .build()?,
        )
        .variant(
            SchemaVariantSpec::builder()
                .version("v0")
                .unique_id("starfield_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&starfield_authoring_schema_func.unique_id)
                        .build()?,
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
                        .kind(ActionKind::Create)
                        .func_unique_id(&starfield_create_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Refresh)
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
        .func(starfield_kripke_func)
        .schema(starfield_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(starfield_spec)?;
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id.into()),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}

pub(crate) async fn migrate_test_exclusive_schema_morningstar(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut morningstar_builder = PkgSpec::builder();
    let schema_name = "morningstar";

    morningstar_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let star_func_code = "async function star(input: Input): Promise<Output> {
        if (!input?.stars) {
            return 'a starless sky';
        }
        const stars = Array.isArray(input.stars) ? input.stars : [input.stars];
        return stars[0] ?? 'not a star in the sky';
    }";

    let star_func_name = "star";
    let star_func = FuncSpec::builder()
        .name(star_func_name)
        .unique_id(star_func_name)
        .data(
            FuncSpecData::builder()
                .name(star_func_name)
                .code_plaintext(star_func_code)
                .handler(star_func_name)
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::String)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("stars")
                .kind(FuncArgumentKind::Array)
                .build()?,
        )
        .build()?;

    let morningstar_scaffold_func = "function createAsset() {\
                return new AssetBuilder().build();
            }";
    let fn_name = "test:scaffoldMorningstarAsset";
    let morningstar_authoring_schema_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(morningstar_scaffold_func)
                .handler("createAsset")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()?,
        )
        .build()?;

    let morningstar_schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(schema_name)
                .build()?,
        )
        .variant(
            SchemaVariantSpec::builder()
                .version("v0")
                .unique_id("morningstar_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&morningstar_authoring_schema_func.unique_id)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("stars")
                        .kind(PropKind::String)
                        .func_unique_id(&star_func.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .name("stars")
                                .socket_name("naming_and_necessity")
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("naming_and_necessity")
                        .data(
                            SocketSpecData::builder()
                                .arity(SocketSpecArity::Many)
                                .name("naming_and_necessity")
                                .connection_annotations(serde_json::to_string(&vec![
                                    "naming_and_necessity",
                                ])?)
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let morningstar_spec = morningstar_builder
        .func(identity_func_spec)
        .func(star_func)
        .func(morningstar_authoring_schema_func)
        .schema(morningstar_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(morningstar_spec)?;
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id.into()),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}

pub(crate) async fn migrate_test_exclusive_schema_etoiles(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut etoiles_builder = PkgSpec::builder();

    let schema_name = "etoiles";

    etoiles_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let etoiles_scaffold_func = "function createAsset() {\
                return new AssetBuilder().build();
            }";
    let fn_name = "test:scaffoldetoilesAsset";
    let etoiles_authoring_schema_func = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(etoiles_scaffold_func)
                .handler("createAsset")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()?,
        )
        .build()?;

    let etoiles_kripke_func_code = "async function la_belle_etoile(input) {
            let rigid_designator = 'not hesperus';

            if (input?.possible_world_a?.wormhole_1?.wormhole_2?.wormhole_3?.rigid_designator === \"hesperus\") { rigid_designator = \"phosphorus\"; }

            return {
                wormhole_1: {
                    wormhole_2: {
                        wormhole_3: {
                            rigid_designator,
                        }
                    }
                }
            }
        }";
    let etoiles_kripke_func_name = "la_belle_etoile";
    let etoiles_kripke_func = FuncSpec::builder()
        .name(etoiles_kripke_func_name)
        .unique_id(etoiles_kripke_func_name)
        .data(
            FuncSpecData::builder()
                .name(etoiles_kripke_func_name)
                .code_plaintext(etoiles_kripke_func_code)
                .handler(etoiles_kripke_func_name)
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Object)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("possible_world_a")
                .kind(FuncArgumentKind::Object)
                .build()?,
        )
        .build()?;

    let etoiles_schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(schema_name)
                .build()?,
        )
        .variant(
            SchemaVariantSpec::builder()
                .version("v0")
                .unique_id("etoiles_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&etoiles_authoring_schema_func.unique_id)
                        .build()?,
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
                        .name("private_language")
                        .kind(PropKind::String)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .name("identity")
                                .socket_name("private_language")
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
                        .func_unique_id(&etoiles_kripke_func.unique_id)
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::Prop)
                                .name("possible_world_a")
                                .prop_path(PropPath::new(["root", "domain", "possible_world_a"]))
                                .build()?,
                        )
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
                .socket(
                    SocketSpec::builder()
                        .name("naming_and_necessity")
                        .data(
                            SocketSpecData::builder()
                                .name("naming_and_necessity")
                                .connection_annotations(serde_json::to_string(&vec![
                                    "naming_and_necessity",
                                ])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new([
                                    "root",
                                    "domain",
                                    "possible_world_b",
                                    "wormhole_1",
                                    "wormhole_2",
                                    "wormhole_3",
                                    "rigid_designator",
                                ]))
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("private_language")
                        .data(
                            SocketSpecData::builder()
                                .name("private_language")
                                .connection_annotations(serde_json::to_string(&vec![
                                    "private_language",
                                ])?)
                                .kind(SocketSpecKind::Input)
                                .arity(SocketSpecArity::One)
                                .build()?,
                        )
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let etoiles_spec = etoiles_builder
        .func(identity_func_spec)
        .func(etoiles_authoring_schema_func)
        .func(etoiles_kripke_func)
        .schema(etoiles_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(etoiles_spec)?;
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id.into()),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}

pub(crate) async fn migrate_test_exclusive_schema_private_language(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut private_lang_builder = PkgSpec::builder();

    let schema_name = "private_language";

    private_lang_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let private_lang_scaff = "function createAsset() {\
                return new AssetBuilder().build();
            }";
    let fn_name = "test:scaffoldPrivateLang";
    let private_lang_auth_schema = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(private_lang_scaff)
                .handler("createAsset")
                .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
                .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
                .build()?,
        )
        .build()?;

    let private_lang_payload_to_val = "async function translate(arg: Input): Promise<Output> {\
            return arg.payload ?? {};
        }";
    let fn_name = "test:resourcePayloadToValue";
    let private_lang_payload_to_val_fn = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(private_lang_payload_to_val)
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

    let private_lang_schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(schema_name)
                .build()?,
        )
        .variant(
            SchemaVariantSpec::builder()
                .version("v0")
                .unique_id("priv_lang_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&private_lang_auth_schema.unique_id)
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("private_language")
                        .data(
                            SocketSpecData::builder()
                                .name("private_language")
                                .connection_annotations(serde_json::to_string(&vec![
                                    "private_language",
                                ])?)
                                .kind(SocketSpecKind::Output)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .name("identity")
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(PropPath::new(["root", "si", "name"]))
                                .build()?,
                        )
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let private_lang_spec = private_lang_builder
        .func(identity_func_spec)
        .func(private_lang_auth_schema)
        .func(private_lang_payload_to_val_fn)
        .schema(private_lang_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(private_lang_spec)?;
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id.into()),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}
