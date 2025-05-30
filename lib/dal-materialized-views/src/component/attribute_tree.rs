use std::collections::{
    HashMap,
    HashSet,
    VecDeque,
};

use dal::{
    AttributePrototype,
    AttributeValue,
    Component,
    DalContext,
    Func,
    Prop,
    Secret,
    component::ControllingFuncData,
    validation::ValidationOutputNode,
};
use si_frontend_mv_types::component::attribute_tree::{
    self,
    AttributeTree,
    AttributeValue as AttributeValueMv,
    AvTreeInfo,
    ValidationOutput,
};
use si_id::{
    AttributeValueId,
    ComponentId,
    InputSocketId,
};
use telemetry::prelude::*;

use crate::{
    schema_variant::prop_tree,
    secret,
};

/// Generates an [`AttributeTree`] MV.
pub async fn assemble(ctx: DalContext, component_id: ComponentId) -> crate::Result<AttributeTree> {
    let ctx = &ctx;

    let root_av_id = Component::root_attribute_value_id(ctx, component_id).await?;
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let sockets_on_component: HashSet<InputSocketId> =
        Component::incoming_connections_for_id(ctx, component_id)
            .await?
            .iter()
            .map(|c| c.to_input_socket_id)
            .collect();
    let secrets_category_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "secrets"]).await?;
    let secret_ids_by_key = Secret::list_ids_by_key(ctx).await?;

    let mut attribute_values = HashMap::new();
    let mut props = HashMap::new();
    let mut tree_info = HashMap::new();

    let mut work_queue = VecDeque::from([root_av_id]);

    while let Some(av_id) = work_queue.pop_front() {
        let maybe_parent_av_id = AttributeValue::parent_id(ctx, av_id).await?;
        let child_av_ids: Vec<AttributeValueId> =
            AttributeValue::get_child_avs_in_order(ctx, av_id)
                .await?
                .iter()
                .map(|av| av.id())
                .collect();
        work_queue.extend(&child_av_ids);
        tree_info.insert(
            av_id,
            AvTreeInfo {
                parent: maybe_parent_av_id,
                children: child_av_ids,
            },
        );

        let maybe_prop = AttributeValue::prop_opt(ctx, av_id).await?;

        // Build si_frontend_mv_types::AttributeValue & add to attribute_values HashMap.
        let key = AttributeValue::key_for_id(ctx, av_id).await?;
        let (value, maybe_secret) = {
            let mut default_none_secret_id = None;
            let mut value = match AttributeValue::get_by_id(ctx, av_id)
                .await?
                .value(ctx)
                .await?
            {
                Some(value) => value,
                None => match &maybe_prop {
                    Some(prop) => Prop::default_value(ctx, prop.id)
                        .await?
                        .unwrap_or(serde_json::Value::Null),
                    None => serde_json::Value::Null,
                },
            };

            if value != serde_json::Value::Null
                && maybe_parent_av_id == Some(secrets_category_av_id)
            {
                let secret_key = Secret::key_from_value_in_attribute_value(value)?;
                value = match secret_ids_by_key.get(&secret_key) {
                    Some(secret_id) => {
                        let secret = secret::assemble(ctx, *secret_id).await?;
                        default_none_secret_id = Some(secret);
                        serde_json::to_value(secret_id)?
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
                            av_id = %av_id,
                            "Secret key in dependent value does not match any secret key; assuming that dependent values are not up to date and treating the property temporarily as missing",
                        );
                        serde_json::Value::Null
                    }
                }
            }

            (value, default_none_secret_id)
        };
        let sockets_for_av = AttributeValue::list_input_socket_sources_for_id(ctx, av_id).await?;
        let can_be_set_by_socket = !sockets_for_av.is_empty();

        let subscriptions = AttributeValue::subscriptions(ctx, av_id).await?;
        let is_from_sub_external_source = subscriptions.is_some_and(|subs| !subs.is_empty());

        let is_from_external_source = is_from_sub_external_source
            || sockets_for_av
                .iter()
                .any(|s| sockets_on_component.contains(s));
        let prototype_id = AttributeValue::prototype_id(ctx, av_id).await?;
        let func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;

        // FIXME(nick): this is likely incorrect.
        let controlling_func = ControllingFuncData {
            func_id,
            av_id,
            is_dynamic_func: func.is_dynamic(),
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
            prop_id: maybe_prop.as_ref().map(|p| p.id),
            key,
            path: Some(av_path),
            value,
            can_be_set_by_socket,
            is_from_external_source,
            is_controlled_by_ancestor: controlling_func.av_id != av_id,
            is_controlled_by_dynamic_func: controlling_func.is_dynamic_func,
            overridden,
            validation,
            secret: maybe_secret,
        };
        attribute_values.insert(av_id, av_mv);

        if let Some(prop) = maybe_prop {
            // If si_frontend_mv_types::Prop is not already in props HashMap, build & add.
            if let std::collections::hash_map::Entry::Vacant(e) = props.entry(prop.id) {
                let prop_mv =
                    prop_tree::assemble_prop(ctx.clone(), prop.id(), schema_variant_id).await?;
                e.insert(prop_mv);
            }
        }
    }

    Ok(AttributeTree {
        attribute_values,
        props,
        tree_info,
    })
}
