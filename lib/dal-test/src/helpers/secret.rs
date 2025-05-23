use dal::{
    BuiltinsResult,
    prop::{
        PropPath,
        SECRET_KIND_WIDGET_OPTION_LABEL,
    },
};
use si_pkg::{
    AttrFuncInputSpec,
    AttrFuncInputSpecKind,
    FuncSpec,
    PropSpec,
    PropSpecKind,
    PropSpecWidgetKind,
    SocketSpec,
    SocketSpecArity,
    SocketSpecData,
    SocketSpecKind,
};

/// Mimics functionality from "asset_builder.ts".
pub fn assemble_dummy_secret_socket_and_prop(
    identity_func_spec: &FuncSpec,
    secret_definition_name: &str,
) -> BuiltinsResult<(SocketSpec, PropSpec)> {
    // Create the input socket for the secret.
    let secret_input_socket = SocketSpec::builder()
        .name(secret_definition_name)
        .data(
            SocketSpecData::builder()
                .name(secret_definition_name)
                .connection_annotations(serde_json::to_string(&vec![
                    secret_definition_name.to_lowercase(),
                ])?)
                .kind(SocketSpecKind::Input)
                .arity(SocketSpecArity::One)
                .func_unique_id(&identity_func_spec.unique_id)
                .build()?,
        )
        .build()?;

    // Create the secret prop for the secret.
    let secret_prop = PropSpec::builder()
        .name(secret_definition_name)
        .kind(PropSpecKind::String)
        .widget_kind(PropSpecWidgetKind::Secret)
        .func_unique_id(&identity_func_spec.unique_id)
        .widget_options(serde_json::json![
            [
                {
                    "label": SECRET_KIND_WIDGET_OPTION_LABEL,
                    "value": secret_definition_name
                }
            ]
        ])
        .input(
            AttrFuncInputSpec::builder()
                .name("identity")
                .kind(AttrFuncInputSpecKind::InputSocket)
                .socket_name(secret_definition_name)
                .build()?,
        )
        .build()?;

    Ok((secret_input_socket, secret_prop))
}

/// Mimics the "defineSecret" function in "asset_builder.ts".
pub fn assemble_secret_definition_dummy(
    identity_func_spec: &FuncSpec,
    secret_definition_name: &str,
) -> BuiltinsResult<(PropSpec, PropSpec, SocketSpec)> {
    // First, create the child of "/root/secret_definition" that defines our secret.
    let new_secret_definition_prop = PropSpec::builder()
        .name("value")
        .kind(PropSpecKind::String)
        .widget_kind(PropSpecWidgetKind::Password)
        .build()?;

    // Second, add it as a new prop underneath "/root/secrets" object. Make sure the "secretKind" is available.
    let new_secret_prop = PropSpec::builder()
        .name(secret_definition_name)
        .kind(PropSpecKind::String)
        .widget_kind(PropSpecWidgetKind::Secret)
        .widget_options(serde_json::json![
            [
                {
                    "label": SECRET_KIND_WIDGET_OPTION_LABEL,
                    "value": secret_definition_name
                }
            ]
        ])
        .build()?;

    // Third, add an output socket for other components to use the secret.
    let new_secret_output_socket = SocketSpec::builder()
        .name(secret_definition_name)
        .data(
            SocketSpecData::builder()
                .name(secret_definition_name)
                .connection_annotations(serde_json::to_string(&vec![
                    secret_definition_name.to_lowercase(),
                ])?)
                .kind(SocketSpecKind::Output)
                .arity(SocketSpecArity::One)
                .func_unique_id(&identity_func_spec.unique_id)
                .build()?,
        )
        .input(
            AttrFuncInputSpec::builder()
                .name("identity")
                .kind(AttrFuncInputSpecKind::Prop)
                .prop_path(PropPath::new(["root", "secrets", secret_definition_name]))
                .build()?,
        )
        .build()?;

    Ok((
        new_secret_definition_prop,
        new_secret_prop,
        new_secret_output_socket,
    ))
}
