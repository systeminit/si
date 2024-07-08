use dal::func::argument::FuncArgumentKind;
use dal::pkg::import_pkg_from_pkg;
use dal::{BuiltinsResult, DalContext};
use dal::{ComponentType, PropKind};
use si_pkg::{
    AttrFuncInputSpec, AttrFuncInputSpecKind, FuncArgumentSpec, FuncSpec, FuncSpecBackendKind,
    FuncSpecBackendResponseType, FuncSpecData, PkgSpec, PropSpec, PropSpecWidgetKind, SchemaSpec,
    SchemaVariantSpec, SchemaVariantSpecData, SiPkg, SocketSpecArity,
};
use si_pkg::{SchemaSpecData, SocketSpec, SocketSpecData, SocketSpecKind};

use crate::test_exclusive_schemas::{
    build_asset_func, create_identity_func, PKG_CREATED_BY, PKG_VERSION,
};

pub(crate) async fn migrate_test_exclusive_schema_fake_butane(
    ctx: &DalContext,
) -> BuiltinsResult<()> {
    let mut builder = PkgSpec::builder();

    let schema_name = "Butane";

    builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    let fn_name = "test:scaffoldFakeButane";
    let authoring_schema_func = build_asset_func(fn_name)?;

    let code = "async function main(input: Input): Promise < Output > {
        if (input.images === undefined || input.images === null) return [];
        let images = Array.isArray(input.images) ? input.images : [input.images];

        let units: any[] = [];

        images
        .filter((i: any) => i ?? false)
        .forEach(function(dockerImage: any) {
            let name = dockerImage.si.name
                .replace(/[^A-Za-z0-9]/g, '-')
                .replace(/-+$/, '')
                .toLowerCase();
            let unit: Record < string, any > = {
                name: name + '.service',
                enabled: true,
            };

            let ports = '';
            let dockerImageExposedPorts = dockerImage.domain.ExposedPorts;
            if (
                !(
                    dockerImageExposedPorts === undefined ||
                    dockerImageExposedPorts === null
                )
            ) {
                dockerImageExposedPorts.forEach(function(dockerImageExposedPort: any) {
                    if (
                        !(
                            dockerImageExposedPort === undefined ||
                            dockerImageExposedPort === null
                        )
                    ) {
                        let parts = dockerImageExposedPort.split('/');
                        try {
                            // Prefix with a blank space.
                            ports = ports + ` --publish ${parts[0]}:${parts[0]}`;
                        } catch (err) {}
                    }
                });
            }

            let image = dockerImage.domain.image;
            let defaultDockerHost = 'docker.io';
            let imageParts = image.split('/');
            if (imageParts.length === 1) {
                image = [defaultDockerHost, 'library', imageParts[0]].join('/');
            } else if (imageParts.length === 2) {
                image = [defaultDockerHost, imageParts[0], imageParts[1]].join('/');
            }

            let description = name.charAt(0).toUpperCase() + name.slice(1);

            unit.contents = `[Unit]\nDescription=${description}\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill ${name}\nExecStartPre=-/bin/podman rm ${name}\nExecStartPre=/bin/podman pull ${image}\nExecStart=/bin/podman run --name ${name}${ports} ${image}\n\n[Install]\nWantedBy=multi-user.target`;

            units.push(unit);
        });

        return units;
    }";
    let fn_name = "test:dockerImagesToButaneUnits";
    let docker_images_to_butane_units_func_spec = FuncSpec::builder()
        .name(fn_name)
        .unique_id(fn_name)
        .data(
            FuncSpecData::builder()
                .name(fn_name)
                .code_plaintext(code)
                .handler("main")
                .backend_kind(FuncSpecBackendKind::JsAttribute)
                .response_type(FuncSpecBackendResponseType::Array)
                .build()?,
        )
        .argument(
            FuncArgumentSpec::builder()
                .name("images")
                .kind(FuncArgumentKind::Array)
                .element_kind(Some(FuncArgumentKind::Object.into()))
                .build()?,
        )
        .build()?;

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
                .unique_id("butane_sv")
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
                        .name("version")
                        .kind(PropKind::String)
                        .default_value(serde_json::json!("1.4.0"))
                        .widget_kind(PropSpecWidgetKind::Text)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("variant")
                        .kind(PropKind::String)
                        .default_value(serde_json::json!("fcos"))
                        .widget_kind(PropSpecWidgetKind::Text)
                        .build()?,
                )
                .domain_prop(
                    PropSpec::builder()
                        .name("systemd")
                        .kind(PropKind::Object)
                        .entry(
                            PropSpec::builder()
                                .name("units")
                                .kind(PropKind::Array)
                                .func_unique_id(&docker_images_to_butane_units_func_spec.unique_id)
                                .input(
                                    AttrFuncInputSpec::builder()
                                        .kind(AttrFuncInputSpecKind::InputSocket)
                                        .name("images")
                                        .socket_name("Container Image")
                                        .build()?,
                                )
                                .type_prop(
                                    PropSpec::builder()
                                        .name("unit")
                                        .kind(PropKind::Object)
                                        .entry(
                                            PropSpec::builder()
                                                .name("name")
                                                .kind(PropKind::String)
                                                .widget_kind(PropSpecWidgetKind::Text)
                                                .build()?,
                                        )
                                        .entry(
                                            PropSpec::builder()
                                                .name("enabled")
                                                .kind(PropKind::Boolean)
                                                .widget_kind(PropSpecWidgetKind::Checkbox)
                                                .build()?,
                                        )
                                        .entry(
                                            PropSpec::builder()
                                                .name("contents")
                                                .kind(PropKind::String)
                                                .widget_kind(PropSpecWidgetKind::TextArea)
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
                        .name("Container Image")
                        .data(
                            SocketSpecData::builder()
                                .name("Container Image")
                                .kind(SocketSpecKind::Input)
                                .arity(SocketSpecArity::Many)
                                .func_unique_id(&identity_func_spec.unique_id)
                                .connection_annotations(serde_json::to_string(&vec![
                                    "Container Image",
                                ])?)
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
        .func(docker_images_to_butane_units_func_spec)
        .schema(schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(spec)?;
    import_pkg_from_pkg(ctx, &pkg, None).await?;

    Ok(())
}
