use std::collections::{
    HashMap,
    VecDeque,
};

use dal::{
    AttributePrototype,
    AttributeValue,
    AttributeValueId,
    Component,
    DalContext,
    Prop,
    PropId,
    Secret,
    SecretError,
    SecretId,
    component::ControllingFuncData,
    secret::EncryptedSecretKey,
    validation::ValidationOutputNode,
    workspace_snapshot::traits::func::FuncExt as _,
};
use si_frontend_mv_types::{
    component::attribute_tree::{
        self,
        AttributeTree,
        AttributeValue as AttributeValueMv,
        AvTreeInfo,
        ExternalSource,
        ValidationOutput,
    },
    secret::Secret as SecretMv,
};
use si_id::ComponentId;
use telemetry::prelude::*;

use crate::{
    schema_variant::prop_tree,
    secret,
};

/// Generates an [`AttributeTree`] MV.
pub async fn assemble(ctx: DalContext, component_id: ComponentId) -> crate::Result<AttributeTree> {
    let ctx = &ctx;

    let component = Component::get_by_id(ctx, component_id).await?;
    let root_av_id = Component::root_attribute_value_id(ctx, component_id).await?;
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let secrets_category_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "secrets"]).await?;
    let secret_ids_by_key = Secret::list_ids_by_key(ctx).await?;

    let mut attribute_values = HashMap::new();
    let mut props = HashMap::new();
    let mut tree_info = HashMap::new();

    let mut work_queue = VecDeque::from([root_av_id]);

    while let Some(av_id) = work_queue.pop_front() {
        let maybe_parent_av_id = AttributeValue::parent_id(ctx, av_id).await?;
        let child_av_ids = AttributeValue::get_child_av_ids_in_order(ctx, av_id).await?;
        work_queue.extend(&child_av_ids);
        tree_info.insert(
            av_id,
            AvTreeInfo {
                parent: maybe_parent_av_id,
                children: child_av_ids,
            },
        );

        let prop_id = AttributeValue::prop_id_opt(ctx, av_id).await?;

        // Build si_frontend_mv_types::AttributeValue & add to attribute_values HashMap.
        let key = AttributeValue::key_for_id(ctx, av_id).await?;
        let (value, maybe_secret) = render_value_and_secret(
            ctx,
            av_id,
            prop_id,
            maybe_parent_av_id,
            secrets_category_av_id,
            &secret_ids_by_key,
        )
        .await?;

        let subscriptions = AttributeValue::subscriptions(ctx, av_id).await?;

        let external_sources: Option<Vec<ExternalSource>> = if let Some(subs) = subscriptions {
            let mut sources = Vec::with_capacity(subs.len());
            for sub in subs {
                let comp_id = AttributeValue::component_id(ctx, sub.attribute_value_id).await?;
                let comp_name = Component::name_by_id(ctx, comp_id).await?;
                let source = ExternalSource {
                    component_id: comp_id,
                    path: sub.path.to_string(),
                    component_name: comp_name,
                    is_secret: sub.path.to_string().starts_with("/secrets/"),
                };
                sources.push(source);
            }
            Some(sources)
        } else {
            None
        };

        let prototype_id = AttributeValue::prototype_id(ctx, av_id).await?;
        let func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        let is_dynamic_func = ctx.workspace_snapshot()?.func_is_dynamic(func_id).await?;

        // FIXME(nick): this is likely incorrect.
        let controlling_func = ControllingFuncData {
            func_id,
            av_id,
            is_dynamic_func,
        };

        // NOTE(nick): I ported Victor's comment.
        //
        // Note (victor): An attribute value is overridden if there is an attribute
        // prototype for this specific AV, which means it's set for the component,
        // not the schema variant. If the av is controlled, this check should be
        // made for its controlling AV.
        // This could be standalone func for AV, but we'd have to implement a
        // controlling_ancestors_for_av_id for av, instead of for the whole component.
        // Not a complicated task, but the PR that adds this has enough code as it is.
        let overridden = AttributeValue::component_prototype_id(ctx, controlling_func.av_id)
            .await?
            .is_some();
        let validation = ValidationOutputNode::find_for_attribute_value_id(ctx, av_id)
            .await?
            .map(|node| ValidationOutput {
                status: match node.validation.status {
                    dal::validation::ValidationStatus::Pending => {
                        attribute_tree::ValidationStatus::Pending
                    }
                    dal::validation::ValidationStatus::Error => {
                        attribute_tree::ValidationStatus::Error
                    }
                    dal::validation::ValidationStatus::Failure => {
                        attribute_tree::ValidationStatus::Failure
                    }
                    dal::validation::ValidationStatus::Success => {
                        attribute_tree::ValidationStatus::Success
                    }
                },
                message: node.validation.message,
            });

        let (_, av_path) = AttributeValue::path_from_root(ctx, av_id).await?;

        let av_mv = AttributeValueMv {
            id: av_id,
            prop_id,
            key,
            path: av_path,
            value,
            external_sources,
            is_controlled_by_ancestor: controlling_func.av_id != av_id,
            is_controlled_by_dynamic_func: controlling_func.is_dynamic_func,
            overridden,
            validation,
            secret: maybe_secret,
            has_socket_connection: false,
            is_default_source: AttributeValue::is_default_subscription_source(ctx, av_id).await?,
        };
        attribute_values.insert(av_id, av_mv);

        if let Some(prop_id) = prop_id {
            // If si_frontend_mv_types::Prop is not already in props HashMap, build & add.
            if let std::collections::hash_map::Entry::Vacant(e) = props.entry(prop_id) {
                let prop_mv =
                    prop_tree::assemble_prop(ctx.clone(), prop_id, schema_variant_id).await?;
                e.insert(prop_mv);
            }
        }
    }

    Ok(AttributeTree {
        id: component_id,
        attribute_values,
        props,
        tree_info,
        component_name: component.name(ctx).await?,
        schema_name: component.schema(ctx).await?.name,
    })
}

async fn render_value_and_secret(
    ctx: &DalContext,
    av_id: AttributeValueId,
    maybe_prop_id: Option<PropId>,
    maybe_parent_av_id: Option<AttributeValueId>,
    secrets_category_av_id: AttributeValueId,
    secret_ids_by_key: &HashMap<EncryptedSecretKey, SecretId>,
) -> crate::Result<(serde_json::Value, Option<SecretMv>)> {
    let maybe_value = AttributeValue::get_by_id(ctx, av_id)
        .await?
        .value(ctx)
        .await?;

    // We only want to proceed if we are dealing with a non-null value and the parent is the secret
    // category node. In all other conditions, either return the value if there is one or use the
    // prop's default value if it exists.
    let value = match maybe_value {
        Some(value)
            if value != serde_json::Value::Null
                && maybe_parent_av_id == Some(secrets_category_av_id) =>
        {
            value
        }
        Some(value) => return Ok((value, None)),
        None => {
            let value = match maybe_prop_id {
                Some(prop_id) => Prop::default_value(ctx, prop_id)
                    .await?
                    .unwrap_or(serde_json::Value::Null),
                None => serde_json::Value::Null,
            };
            return Ok((value, None));
        }
    };

    // Try to extract the secret key from the value in the attribute value.
    match Secret::key_from_value_in_attribute_value(value) {
        Ok(secret_key) => {
            // Check the cache to assemble the secret accordingly.
            match secret_ids_by_key.get(&secret_key) {
                Some(secret_id) => {
                    let rendered_value = serde_json::to_value(secret_id)?;
                    let secret = secret::assemble(ctx, *secret_id).await?;
                    Ok((rendered_value, Some(secret)))
                }
                None => {
                    // NOTE(nick): I ported this comment.
                    //
                    // If none of the secrets in the workspace have this key, we assume
                    // that dependent values haven't updated yet and will be fixed
                    // shortly. Thus we treat the property as missing for now and
                    // return null.
                    //
                    // This is an expected issue, so we don't warn--but it could trigger
                    // if something more serious is going on that is making the lookup
                    // fail more persistently, so we may want to measure how often it
                    // happens and figure out how to alert in that case.
                    warn!(
                        name: "Secret key does not match",
                        %av_id,
                        "Secret key in dependent value does not match any secret key; assuming that dependent values are not up to date and treating the property temporarily as missing",
                    );
                    Ok((serde_json::Value::Null, None))
                }
            }
        }
        Err(SecretError::EncryptedSecretKeyParse(_)) => {
            // NOTE(nick): this appears to happen during authoring, but the reason is
            // unclear. We need to not only avoid hard failing the MV build here, but also
            // use tracing to figure out why this happens in authoring over the long term.
            warn!(
                name: "Value for secret attribute value is not a secret key",
                %av_id,
                "Value in dependent value is not a secret key; assuming that dependent values are not up to date and treating the property temporarily as missing",
            );
            Ok((serde_json::Value::Null, None))
        }
        Err(err) => Err(err.into()),
    }
}
