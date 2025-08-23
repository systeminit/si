use dal::{
    AttributePrototype,
    AttributeValue,
    Component,
    DalContext,
    Func,
    Prop,
    Secret,
    attribute::{
        prototype::argument::{
            AttributePrototypeArgument,
            static_value::StaticArgumentValue,
            value_source::ValueSource,
        },
        value::subscription::ValueSubscription,
    },
    func::intrinsics::IntrinsicFunc,
};
use si_frontend_mv_types::component::{
    ComponentDiffStatus,
    component_diff::{
        AttributeDiff,
        AttributeSource,
        AttributeSourceAndValue,
        ComponentDiff,
        SimplifiedAttributeSource,
    },
};
use si_id::{
    AttributePrototypeArgumentId,
    AttributePrototypeId,
    AttributeValueId,
    ComponentId,
};
use telemetry::prelude::*;

/// Generates a [`ComponentDiff`] MV.
pub async fn assemble(new_ctx: DalContext, id: ComponentId) -> crate::Result<ComponentDiff> {
    // If we are already on HEAD, it's unmodified; short circuit!
    let new_ctx = &new_ctx;
    let old_ctx = new_ctx.clone_with_head().await?;
    let old_ctx = &old_ctx;
    if new_ctx.change_set_id() == old_ctx.change_set_id() {
        return Ok(ComponentDiff {
            id,
            diff_status: ComponentDiffStatus::None,
            attribute_diffs: vec![],
        });
    }

    // Diff attributes
    let new_root_av_id = if Component::exists_by_id(new_ctx, id).await? {
        Some(Component::root_attribute_value_id(new_ctx, id).await?)
    } else {
        None
    };
    let old_root_av_id = if Component::exists_by_id(old_ctx, id).await? {
        Some(Component::root_attribute_value_id(old_ctx, id).await?)
    } else {
        None
    };
    let attribute_diffs = diff_attributes(old_ctx, old_root_av_id, new_ctx, new_root_av_id).await?;

    // Figure out diff status
    let diff_status = if new_root_av_id.is_some() {
        if old_root_av_id.is_none() {
            ComponentDiffStatus::Added
        } else if !attribute_diffs.is_empty() {
            ComponentDiffStatus::Modified
        } else {
            ComponentDiffStatus::None
        }
    } else {
        ComponentDiffStatus::Removed
    };

    Ok(ComponentDiff {
        id,
        diff_status,
        attribute_diffs,
    })
}

// Walk two attributes, diffing them and their children and adding the results to
async fn diff_attributes(
    old_ctx: &DalContext,
    old_av_id: Option<AttributeValueId>,
    new_ctx: &DalContext,
    new_av_id: Option<AttributeValueId>,
) -> crate::Result<Vec<(String, AttributeDiff)>> {
    let mut attribute_diffs = vec![];
    let mut work_queue = Vec::from([(old_av_id, new_av_id)]);
    while let Some((old_av_id, new_av_id)) = work_queue.pop() {
        if let Some(maybe_hidden) = new_av_id {
            let new_prop_id = AttributeValue::prop_id(new_ctx, maybe_hidden).await?;
            let new_prop = Prop::get_by_id(new_ctx, new_prop_id).await?;
            if new_prop.hidden {
                // This is a hidden prop, exclude it!
                continue;
            }
        }

        match (old_av_id, new_av_id) {
            // Modified or Unchanged
            (Some(old_av_id), Some(new_av_id)) => {
                // If they are different, push up the Modified entry.
                if !attributes_are_same(old_ctx, old_av_id, new_ctx, new_av_id).await? {
                    // If they are different, diff them.
                    let (_, new_path) = AttributeValue::path_from_root(new_ctx, new_av_id).await?;
                    let old = assemble_source_and_value(old_ctx, old_av_id).await?;
                    let new = assemble_source_and_value(new_ctx, new_av_id).await?;
                    attribute_diffs.push((new_path, AttributeDiff::Modified { old, new }));
                }

                // Check if any children are different (whether this particular AV was different
                // or not!)
                for (old_av_id, new_av_id) in
                    child_av_pairs(old_ctx, old_av_id, new_ctx, new_av_id).await?
                {
                    work_queue.push((old_av_id, new_av_id));
                }
            }

            // Added
            (None, Some(new_av_id)) => {
                let (_, new_path) = AttributeValue::path_from_root(new_ctx, new_av_id).await?;
                let new = assemble_source_and_value(new_ctx, new_av_id).await?;
                attribute_diffs.push((new_path, AttributeDiff::Added { new }));

                for new_av_id in AttributeValue::get_child_av_ids_in_order(new_ctx, new_av_id)
                    .await?
                    .into_iter()
                    .rev()
                {
                    work_queue.push((None, Some(new_av_id)));
                }
            }

            // Removed
            (Some(old_av_id), None) => {
                let (_, old_path) = AttributeValue::path_from_root(old_ctx, old_av_id).await?;
                let old = assemble_source_and_value(old_ctx, old_av_id).await?;
                attribute_diffs.push((old_path, AttributeDiff::Removed { old }));

                for old_av_id in AttributeValue::get_child_av_ids_in_order(old_ctx, old_av_id)
                    .await?
                    .into_iter()
                    .rev()
                {
                    work_queue.push((Some(old_av_id), None));
                }
            }

            (None, None) => {}
        }
    }
    Ok(attribute_diffs)
}

async fn attributes_are_same(
    old_ctx: &DalContext,
    old_av_id: AttributeValueId,
    new_ctx: &DalContext,
    new_av_id: AttributeValueId,
) -> crate::Result<bool> {
    // If the JS values are different, they are different
    //
    // (We only check the content address; it's technically possible for two JS values to be
    // the same even if content is different, but only when there is extra whitespace in the
    // JSON)
    let old_av = AttributeValue::node_weight(old_ctx, old_av_id).await?;
    let new_av = AttributeValue::node_weight(new_ctx, new_av_id).await?;
    if old_av.value() != new_av.value() {
        return Ok(false);
    }

    // If the prototypes are different, they are different
    let old_prototype_id = AttributeValue::component_prototype_id(old_ctx, old_av_id).await?;
    let new_prototype_id = AttributeValue::component_prototype_id(new_ctx, new_av_id).await?;
    match (old_prototype_id, new_prototype_id) {
        (Some(old_prototype_id), Some(new_prototype_id)) => {
            if !prototypes_are_same(old_ctx, old_prototype_id, new_ctx, new_prototype_id).await? {
                return Ok(false);
            }
        }
        (None, None) => {
            let old_prototype_id =
                AttributeValue::schema_variant_prototype_id(old_ctx, old_av_id).await?;
            let new_prototype_id =
                AttributeValue::schema_variant_prototype_id(new_ctx, new_av_id).await?;
            if !prototypes_are_same(old_ctx, old_prototype_id, new_ctx, new_prototype_id).await? {
                return Ok(false);
            }
        }
        // If one is the default prototype and the other is the schema variant prototype,
        // they are different *even if the values / prototypes are the same*. This means the
        // user has explicitly overridden, or unset the value.
        (Some(_), None) | (None, Some(_)) => {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn prototypes_are_same(
    old_ctx: &DalContext,
    old_prototype_id: AttributePrototypeId,
    new_ctx: &DalContext,
    new_prototype_id: AttributePrototypeId,
) -> crate::Result<bool> {
    // If the functions are different, they are different.
    if AttributePrototype::func_id(old_ctx, old_prototype_id).await?
        != AttributePrototype::func_id(new_ctx, new_prototype_id).await?
    {
        return Ok(false);
    }

    // If the arguments are different, they are different.
    let old_apa_ids = AttributePrototype::list_arguments(old_ctx, old_prototype_id).await?;
    let new_apa_ids = AttributePrototype::list_arguments(new_ctx, new_prototype_id).await?;
    if old_apa_ids.len() != new_apa_ids.len() {
        return Ok(false);
    }
    for (old_apa_id, new_apa_id) in old_apa_ids.into_iter().zip(new_apa_ids) {
        if !arguments_are_same(old_ctx, old_apa_id, new_ctx, new_apa_id).await? {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn arguments_are_same(
    old_ctx: &DalContext,
    old_apa_id: AttributePrototypeArgumentId,
    new_ctx: &DalContext,
    new_apa_id: AttributePrototypeArgumentId,
) -> crate::Result<bool> {
    // If they are for different arguments, they are different.
    if AttributePrototypeArgument::func_argument_id(old_ctx, old_apa_id).await?
        != AttributePrototypeArgument::func_argument_id(new_ctx, new_apa_id).await?
    {
        return Ok(false);
    }

    // If they are for different targets, they are different.
    let old_arg = AttributePrototypeArgument::get_by_id(old_ctx, old_apa_id).await?;
    let new_arg = AttributePrototypeArgument::get_by_id(new_ctx, new_apa_id).await?;
    if old_arg.targets() != new_arg.targets() {
        return Ok(false);
    }

    // If they have different value sources, they are different.
    let old_value = AttributePrototypeArgument::value_source(old_ctx, old_apa_id).await?;
    let new_value = AttributePrototypeArgument::value_source(new_ctx, new_apa_id).await?;
    if !value_sources_are_same(old_ctx, old_value, new_ctx, new_value).await? {
        return Ok(false);
    }

    Ok(true)
}

async fn value_sources_are_same(
    old_ctx: &DalContext,
    old_value: ValueSource,
    new_ctx: &DalContext,
    new_value: ValueSource,
) -> crate::Result<bool> {
    Ok(match (old_value, new_value) {
        (ValueSource::InputSocket(old_id), ValueSource::InputSocket(new_id)) => old_id == new_id,
        (ValueSource::OutputSocket(old_id), ValueSource::OutputSocket(new_id)) => old_id == new_id,
        (ValueSource::Prop(old_id), ValueSource::Prop(new_id)) => old_id == new_id,
        (ValueSource::Secret(old_id), ValueSource::Secret(new_id)) => old_id == new_id,
        // Static values must have the same value
        (ValueSource::StaticArgumentValue(old_id), ValueSource::StaticArgumentValue(new_id)) => {
            StaticArgumentValue::value_content_hash(old_ctx, old_id).await?
                == StaticArgumentValue::value_content_hash(new_ctx, new_id).await?
        }
        // Subscriptions must go to the same component and path
        (ValueSource::ValueSubscription(old_sub), ValueSource::ValueSubscription(new_sub)) => {
            subscriptions_are_same(old_ctx, old_sub, new_ctx, new_sub).await?
        }

        // Different types are different!
        // NOTE: Writing out all the possibilities so if a new source is added, it will have to be
        // added here as well due to exhaustive matching
        (
            ValueSource::InputSocket(_)
            | ValueSource::OutputSocket(_)
            | ValueSource::Prop(_)
            | ValueSource::Secret(_)
            | ValueSource::StaticArgumentValue(_)
            | ValueSource::ValueSubscription(_),
            _,
        ) => false,
    })
}

async fn subscriptions_are_same(
    old_ctx: &DalContext,
    old_sub: ValueSubscription,
    new_ctx: &DalContext,
    new_sub: ValueSubscription,
) -> crate::Result<bool> {
    // If they go to different paths, they are different.
    if old_sub.path != new_sub.path {
        return Ok(false);
    }

    // Short circuit if attribute value IDs are the same--they definitely go to the same root then.
    // This saves us a bunch of traversal work for a very common case.
    if old_sub.attribute_value_id == new_sub.attribute_value_id {
        return Ok(true);
    }

    // Also have to check the attribute value paths (in case a sub goes to a non-root av)
    let (old_root_id, old_av_path) =
        AttributeValue::path_from_root(old_ctx, old_sub.attribute_value_id).await?;
    let (new_root_id, new_av_path) =
        AttributeValue::path_from_root(new_ctx, new_sub.attribute_value_id).await?;
    if old_av_path != new_av_path {
        return Ok(false);
    }

    // If they go to different components, they are different.
    let old_component_id = AttributeValue::component_id(old_ctx, old_root_id).await?;
    let new_component_id = AttributeValue::component_id(new_ctx, new_root_id).await?;
    if old_component_id != new_component_id {
        return Ok(false);
    }

    Ok(true)
}

async fn assemble_source_and_value(
    ctx: &DalContext,
    av_id: AttributeValueId,
) -> crate::Result<AttributeSourceAndValue> {
    let value = AttributeValue::view(ctx, av_id).await?;
    let source = assemble_source(ctx, av_id).await?;
    Ok(AttributeSourceAndValue { value, source })
}

async fn assemble_source(
    ctx: &DalContext,
    av_id: AttributeValueId,
) -> crate::Result<AttributeSource> {
    // Get the controlling prototype and where it's from (from_schema / from_ancestor)
    let (from_ancestor, av_id) = match AttributeValue::controlling_av_id(ctx, av_id).await? {
        Some(controlling_av_id) if controlling_av_id != av_id => {
            let (_, path) = AttributeValue::path_from_root(ctx, controlling_av_id).await?;
            (Some(path), controlling_av_id)
        }
        _ => (None, av_id),
    };
    let (from_schema, prototype_id) =
        match AttributeValue::component_prototype_id(ctx, av_id).await? {
            Some(prototype_id) => (None, prototype_id),
            None => (
                Some(true),
                AttributeValue::schema_variant_prototype_id(ctx, av_id).await?,
            ),
        };

    // Assemble the result
    let simplified_source = assemble_simplified_source(ctx, av_id, prototype_id).await?;
    Ok(AttributeSource {
        simplified_source,
        from_schema,
        from_ancestor,
    })
}

async fn assemble_simplified_source(
    ctx: &DalContext,
    av_id: AttributeValueId,
    prototype_id: AttributePrototypeId,
) -> crate::Result<SimplifiedAttributeSource> {
    // Handle special-case subscription or value
    let args = AttributePrototype::list_arguments(ctx, prototype_id).await?;
    if let Some(&arg_id) = args.first()
        && args.len() == 1
    {
        let func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        if let Some(intrinsic) = Func::intrinsic_kind(ctx, func_id).await? {
            // Secret value (value will be the EncryptedSecretKey but we need to get the actual Secret)
            if let Some(ValueSource::Secret(secret_id)) =
                AttributePrototypeArgument::value_source_opt(ctx, arg_id).await?
            {
                let value = match AttributeValue::get_by_id(ctx, av_id)
                    .await?
                    .value(ctx)
                    .await?
                {
                    Some(value) => value,
                    None => serde_json::Value::Null,
                };
                // if the value source is a secret - construct the correct response
                let secret = crate::secret::assemble(ctx, secret_id).await?;
                return Ok(SimplifiedAttributeSource::SecretValue { value, secret });
            }
            // Static value (si:setString(), si:setObject(), etc.)
            if let Some(static_value) =
                AttributePrototypeArgument::static_value_by_id(ctx, arg_id).await?
                && intrinsic.set_func().is_some()
            {
                return Ok(SimplifiedAttributeSource::Value {
                    value: static_value.value,
                });
            }

            // Subscription (si:identity(subscription to /component /path))
            if let ValueSource::ValueSubscription(subscription) =
                AttributePrototypeArgument::value_source(ctx, arg_id).await?
                && IntrinsicFunc::Identity == intrinsic
            {
                // Special case secrets
                // if it's a subscription to /secrets/* - get the secret
                if subscription
                    .path
                    .is_under(&dal::attribute::path::AttributePath::JsonPointer(
                        "/secrets".to_string(),
                    ))
                {
                    if let Some(secret_av_id) = subscription.resolve(ctx).await? {
                        let component =
                            AttributeValue::component_id(ctx, subscription.attribute_value_id)
                                .await?;
                        if let Some(value) = AttributeValue::get_by_id(ctx, secret_av_id)
                            .await?
                            .value(ctx)
                            .await?
                        {
                            let secret_key = Secret::key_from_value_in_attribute_value(value)?;
                            if let Ok(secret_id) =
                                Secret::get_id_by_key_or_error(ctx, secret_key).await
                            {
                                let secret = crate::secret::assemble(ctx, secret_id).await?;
                                return Ok(SimplifiedAttributeSource::SecretSubscription {
                                    component,
                                    path: subscription.path.to_string(),
                                    secret,
                                });
                            } else {
                                // Fall back to complex prototype if we can't find the secret?
                                warn!(si.error.message="Could not find secret for secret key", si.secret_key=%secret_key);
                            }
                        }
                    }
                }

                // Don't bother if the path isn't /; if it's a subscription whose root is
                // deeply nested, we'll call it a complex prototype and let fmt_title handle it.
                let (root_id, root_path) =
                    AttributeValue::path_from_root(ctx, subscription.attribute_value_id).await?;
                if root_path == "/" {
                    let component = AttributeValue::component_id(ctx, root_id).await?;
                    return Ok(SimplifiedAttributeSource::Subscription {
                        component,
                        path: subscription.path.to_string(),
                    });
                }
            }
        }
    }

    // If it isn't a special simplified case, show the prototype instead.
    let component_id = AttributeValue::component_id(ctx, av_id).await?;
    Ok(SimplifiedAttributeSource::Prototype {
        prototype: AttributePrototype::fmt_title(ctx, prototype_id, Some(component_id)).await,
    })
}

async fn child_av_pairs(
    old_ctx: &DalContext,
    old_parent_av_id: AttributeValueId,
    new_ctx: &DalContext,
    new_parent_av_id: AttributeValueId,
) -> crate::Result<Vec<(Option<AttributeValueId>, Option<AttributeValueId>)>> {
    let new_children = AttributeValue::get_child_av_ids_in_order(new_ctx, new_parent_av_id).await?;
    let old_children = AttributeValue::get_child_av_ids_in_order(old_ctx, old_parent_av_id).await?;

    let mut result = Vec::with_capacity(new_children.len().max(old_children.len()));
    let new_children = new_children.into_iter().rev();
    let mut old_children = old_children.into_iter().rev().peekable();
    for new_av_id in new_children {
        // Check if the old attribute value had this field.
        //
        // TODO match field name for objects and maps. If old and new maps are in a different
        // order, or if the type or field order changes during a schema upgrade, then this may
        // not detect whether two fields are the same.
        let old_av_id = match old_children.peek() {
            Some(&old_av_id) => {
                let old_key = AttributeValue::key_for_id(old_ctx, old_av_id).await?;
                let new_key = AttributeValue::key_for_id(new_ctx, new_av_id).await?;
                match (old_key, new_key) {
                    (Some(old_key), Some(new_key)) if old_key == new_key => old_children.next(),
                    (None, None) => old_children.next(),
                    _ => None,
                }
            }
            None => None,
        };
        result.push((old_av_id, Some(new_av_id)));
    }

    // Go through any remaining old children we haven't consumed, and add them at the end
    for old_av_id in old_children {
        result.push((Some(old_av_id), None));
    }

    Ok(result)
}
