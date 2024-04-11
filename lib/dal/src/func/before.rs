use thiserror::Error;
use veritech_client::{encrypt_value_tree, BeforeFunction, CycloneValueEncryptError};

use crate::attribute::value::AttributeValueError;
use crate::prop::{PropError, PropPath};
use crate::schema::variant::root_prop::RootPropChild;
use crate::schema::variant::SchemaVariantError;
use crate::{
    AttributeValue, Component, ComponentError, ComponentId, DalContext, EncryptedSecret, Func,
    FuncId, Prop, PropId, SchemaVariant, Secret, SecretError, SecretId, StandardModelError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum BeforeFuncError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("cyclone value encrypt error: {0}")]
    CycloneValueEncrypt(#[from] CycloneValueEncryptError),
    #[error("func error: {0}")]
    Func(String),
    #[error("error deserializing json")]
    JsonDeserialize(#[from] serde_json::Error),
    #[error("Function missing expected code: {0}")]
    MissingCode(FuncId),
    #[error("Function missing expected handler: {0}")]
    MissingHandler(FuncId),
    #[error("no widget options on secret prop id: {0}")]
    NoWidgetOptionsOnSecretProp(PropId),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
}

type BeforeFuncResult<T> = Result<T, BeforeFuncError>;

pub async fn before_funcs_for_component(
    ctx: &DalContext,
    component_id: &ComponentId,
) -> BeforeFuncResult<Vec<BeforeFunction>> {
    let secret_props = {
        let schema_variant = Component::schema_variant_id(ctx, *component_id).await?;
        let secrets_prop =
            SchemaVariant::find_root_child_prop_id(ctx, schema_variant, RootPropChild::Secrets)
                .await?;
        Prop::direct_child_prop_ids(ctx, secrets_prop).await?
    };

    let secret_definition_path = PropPath::new(["root", "secret_definition"]);
    let secret_path = PropPath::new(["root", "secrets"]);

    let mut funcs_and_secrets = vec![];
    for secret_prop_id in secret_props {
        let auth_funcs = auth_funcs_for_secret_prop_id(
            ctx,
            secret_prop_id,
            &secret_definition_path,
            &secret_path,
        )
        .await?;

        let av_ids = Prop::attribute_values_for_prop_id(ctx, secret_prop_id).await?;
        let mut maybe_secret_id = None;
        for av_id in av_ids {
            if AttributeValue::component_id(ctx, av_id).await? != *component_id {
                continue;
            }

            let av = AttributeValue::get_by_id(ctx, av_id).await?;

            maybe_secret_id = av.value(ctx).await?;
            break;
        }

        if let Some(secret_id_str) = maybe_secret_id {
            let id: SecretId = serde_json::from_value(secret_id_str)?;

            funcs_and_secrets.push((id, auth_funcs))
        }
    }

    let mut results = vec![];

    for (secret_id, funcs) in funcs_and_secrets {
        let secret = Secret::get_by_id_or_error(ctx, secret_id).await?;
        let encrypted_secret = EncryptedSecret::get_by_key(ctx, secret.key())
            .await?
            .ok_or(SecretError::EncryptedSecretNotFound(secret_id))?;

        // Decrypt message from EncryptedSecret
        let mut arg = encrypted_secret.decrypt(ctx).await?.message().into_inner();

        // Re-encrypt raw Value for transmission to Cyclone via Veritech
        encrypt_value_tree(&mut arg, ctx.encryption_key())?;

        for func in funcs {
            results.push(BeforeFunction {
                handler: func
                    .handler
                    .ok_or_else(|| BeforeFuncError::MissingHandler(func.id))?,
                code_base64: func
                    .code_base64
                    .ok_or_else(|| BeforeFuncError::MissingCode(func.id))?,
                arg: arg.clone(),
            })
        }
    }

    Ok(results)
}

/// This _private_ method gathers the authentication functions for a given [`PropId`](Prop) underneath "/root/secrets".
async fn auth_funcs_for_secret_prop_id(
    ctx: &DalContext,
    secret_prop_id: PropId,
    secret_definition_path: &PropPath,
    secret_path: &PropPath,
) -> BeforeFuncResult<Vec<Func>> {
    let secret_prop = Prop::get_by_id(ctx, secret_prop_id).await?;

    let secret_definition_name = secret_prop
        .widget_options
        .ok_or(BeforeFuncError::NoWidgetOptionsOnSecretProp(secret_prop_id))?
        .pop()
        .ok_or(BeforeFuncError::NoWidgetOptionsOnSecretProp(secret_prop_id))?
        .value;

    let mut auth_funcs = vec![];
    for secret_defining_sv_id in SchemaVariant::list_ids(ctx).await? {
        if Prop::find_prop_id_by_path_opt(ctx, secret_defining_sv_id, secret_definition_path)
            .await?
            .is_none()
        {
            continue;
        }

        let secrets_prop = Prop::find_prop_by_path(ctx, secret_defining_sv_id, secret_path).await?;

        let secret_child_prop_id = Prop::direct_single_child_prop_id(ctx, secrets_prop.id).await?;
        let secret_child_prop = Prop::get_by_id(ctx, secret_child_prop_id).await?;

        if secret_child_prop.name != secret_definition_name {
            continue;
        }

        for auth_func_id in
            SchemaVariant::list_auth_func_ids_for_id(ctx, secret_defining_sv_id).await?
        {
            auth_funcs.push(
                Func::get_by_id_or_error(ctx, auth_func_id)
                    .await
                    .map_err(|e| BeforeFuncError::Func(e.to_string()))?,
            )
        }

        break;
    }

    Ok(auth_funcs)
}
