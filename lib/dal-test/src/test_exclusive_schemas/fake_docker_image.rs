use dal::pkg::import_pkg_from_pkg;
use dal::prop::PropPath;
use dal::ComponentType;
use dal::{BuiltinsResult, DalContext};
use si_pkg::{
    AttrFuncInputSpec, AttrFuncInputSpecKind, PkgSpec, PropSpec, PropSpecKind, PropSpecWidgetKind,
    SchemaSpec, SchemaVariantSpec, SchemaVariantSpecData, SiPkg, SocketSpecArity, SocketSpecData,
    SocketSpecKind,
};
use si_pkg::{SchemaSpecData, SocketSpec};

use crate::test_exclusive_schemas::{
    build_asset_func, create_identity_func, PKG_CREATED_BY, PKG_VERSION,
};

pub(crate) async fn migrate_test_exclusive_schema_fake_docker_image(
    ctx: &DalContext,
) -> BuiltinsResult<()> {
    let mut builder = PkgSpec::builder();

    let schema_name = "Docker Image";

    builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    let fn_name = "test:scaffoldFakeDockerImage";
    let authoring_schema_func = build_asset_func(fn_name)?;

    let schema = SchemaSpec::builder()
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
                .unique_id("docker_image_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&authoring_schema_func.unique_id)
                        .component_type(ComponentType::Component)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("image")
                        .kind(PropSpecKind::String)
                        .func_unique_id(&identity_func_spec.unique_id)
                        .widget_kind(PropSpecWidgetKind::Text)
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
                        .name("ExposedPorts")
                        .kind(PropSpecKind::Array)
                        .widget_kind(PropSpecWidgetKind::Array)
                        .type_prop(
                            PropSpec::builder()
                                .name("ExposedPort")
                                .kind(PropSpecKind::String)
                                .widget_kind(PropSpecWidgetKind::Text)
                                .build()?,
                        )
                        .build()?,
                )
                .socket(
                    SocketSpec::builder()
                        .name("Container Image")
                        .data(
                            SocketSpecData::builder()
                                .name("Container Image")
                                .kind(SocketSpecKind::Output)
                                .arity(SocketSpecArity::Many)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .connection_annotations(serde_json::to_string(&vec![
                                    "Container Image",
                                ])?)
                                .build()?,
                        )
                        .input(
                            AttrFuncInputSpec::builder()
                                .kind(AttrFuncInputSpecKind::Prop)
                                .name("identity")
                                .prop_path(PropPath::new(["root"]))
                                .build()?,
                        )
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let spec = builder
        .func(identity_func_spec)
        .func(authoring_schema_func)
        .schema(schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(spec)?;
    import_pkg_from_pkg(ctx, &pkg, None).await?;

    Ok(())
}
