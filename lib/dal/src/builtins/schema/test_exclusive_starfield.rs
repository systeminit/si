use si_pkg::{
    ActionFuncSpec, AttrFuncInputSpec, AttrFuncInputSpecKind, FuncArgumentSpec, FuncSpec,
    FuncSpecBackendKind, FuncSpecBackendResponseType, LeafFunctionSpec, PkgSpec, PropSpec,
    SchemaSpec, SchemaVariantSpec, SiPkg, SocketSpec, SocketSpecKind,
};

use crate::func::argument::FuncArgumentKind;
use crate::pkg::import_pkg_from_pkg;
use crate::schema::variant::leaves::LeafInputLocation;
use crate::schema::variant::leaves::LeafKind;
use crate::{builtins::schema::MigrationDriver, prop::PropPath, ActionKind};
use crate::{BuiltinsResult, DalContext, PropKind};

impl MigrationDriver {
    pub async fn migrate_test_exclusive_starfield(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let mut starfield_builder = PkgSpec::builder();

        starfield_builder
            .name("starfield")
            .version("2023-05-23")
            .created_by("System Initiative");

        let identity_func_spec = FuncSpec::identity_func()?;
        let starfield_confirmation_code = "async function exists(input) {
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

        let starfield_confirmation_func = FuncSpec::builder()
            .name("test:confirmationStarfield")
            .code_plaintext(starfield_confirmation_code)
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

        let starfield_create_action_code = "async function create() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";
        let starfield_create_action_func = FuncSpec::builder()
            .name("test:createActionStarfield")
            .code_plaintext(starfield_create_action_code)
            .handler("create")
            .backend_kind(FuncSpecBackendKind::JsAction)
            .response_type(FuncSpecBackendResponseType::Action)
            .build()?;

        let starfield_refresh_action_code =
            "async function refresh(component: Input): Promise<Output> {
              return { payload: { \"poop\": true }, status: \"ok\" };
            }";

        let starfield_refresh_action_func = FuncSpec::builder()
            .name("test:refreshActionStarfield")
            .handler("refresh")
            .code_plaintext(starfield_refresh_action_code)
            .backend_kind(FuncSpecBackendKind::JsAction)
            .response_type(FuncSpecBackendResponseType::Action)
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
        let fallout_entries_to_galaxies_transform_func = FuncSpec::builder()
            .name("test:falloutEntriesToGalaxies")
            .code_plaintext(fallout_entries_to_galaxies_transform_code)
            .handler("falloutEntriesToGalaxies")
            .backend_kind(FuncSpecBackendKind::JsAttribute)
            .response_type(FuncSpecBackendResponseType::Array)
            .argument(
                FuncArgumentSpec::builder()
                    .name("entries")
                    .kind(FuncArgumentKind::Array)
                    .element_kind(Some(FuncArgumentKind::Object.into()))
                    .build()?,
            )
            .build()?;

        let starfield_schema = SchemaSpec::builder()
            .name("starfield")
            .category("test exclusive")
            .category_name("starfield")
            .variant(
                SchemaVariantSpec::builder()
                    .color("#ffffff")
                    .name("v0")
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
                            .name("freestar")
                            .kind(PropKind::String)
                            .build()?,
                    )
                    .domain_prop(
                        PropSpec::builder()
                            .name("attributes")
                            .kind(PropKind::String)
                            .func_unique_id(identity_func_spec.unique_id)
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
                            .name("universe")
                            .kind(PropKind::Object)
                            .entry(
                                PropSpec::builder()
                                    .name("galaxies")
                                    .kind(PropKind::Array)
                                    .func_unique_id(
                                        fallout_entries_to_galaxies_transform_func.unique_id,
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
                            .kind(SocketSpecKind::Input)
                            .build()?,
                    )
                    .socket(
                        SocketSpec::builder()
                            .name("fallout")
                            .kind(SocketSpecKind::Input)
                            .build()?,
                    )
                    .leaf_function(
                        LeafFunctionSpec::builder()
                            .leaf_kind(LeafKind::Confirmation)
                            .func_unique_id(starfield_confirmation_func.unique_id)
                            .inputs(vec![LeafInputLocation::Resource.into()])
                            .build()?,
                    )
                    .action_func(
                        ActionFuncSpec::builder()
                            .kind(&ActionKind::Create)
                            .func_unique_id(starfield_create_action_func.unique_id)
                            .build()?,
                    )
                    .action_func(
                        ActionFuncSpec::builder()
                            .kind(&ActionKind::Refresh)
                            .func_unique_id(starfield_refresh_action_func.unique_id)
                            .build()?,
                    )
                    .build()?,
            )
            .build()?;

        let starfield_spec = starfield_builder
            .func(identity_func_spec)
            .func(starfield_refresh_action_func)
            .func(starfield_create_action_func)
            .func(starfield_confirmation_func)
            .func(fallout_entries_to_galaxies_transform_func)
            .schema(starfield_schema)
            .build()?;

        let starfield_pkg = SiPkg::load_from_spec(starfield_spec)?;
        import_pkg_from_pkg(ctx, &starfield_pkg, "test:starfield").await?;

        Ok(())
    }
}
