use color_eyre::eyre::eyre;
use dal::{
    AttributeValue,
    Component,
    DalContext,
    attribute::{
        path::AttributePath,
        value::subscription::ValueSubscription,
    },
    workspace_snapshot::node_weight::reason_node_weight::Reason,
};
use si_id::{
    AttributeValueId,
    FuncId,
};

use crate::{
    Result,
    helpers::component::{
        self,
        ComponentKey,
    },
};

/// Lookup an attribute value by its key (component, path) pair
pub async fn id(ctx: &DalContext, key: impl AttributeValueKey) -> Result<AttributeValueId> {
    AttributeValueKey::id(ctx, key).await
}

/// Get or create an attribute value by its key (component, path) pair
pub async fn vivify(ctx: &DalContext, key: impl AttributeValueKey) -> Result<AttributeValueId> {
    AttributeValueKey::vivify(ctx, key).await
}

/// Set the subscriptions on a value
pub async fn subscribe(
    ctx: &DalContext,
    subscriber: impl AttributeValueKey,
    subscription: impl AttributeValueKey,
) -> Result<()> {
    subscribe_with_custom_function(ctx, subscriber, subscription, None).await
}

/// Set the subscriptions on a value
pub async fn subscribe_with_custom_function(
    ctx: &DalContext,
    subscriber: impl AttributeValueKey,
    subscription: impl AttributeValueKey,
    func_id: Option<FuncId>,
) -> Result<()> {
    let subscriber = vivify(ctx, subscriber).await?;
    AttributeValue::set_to_subscription(
        ctx,
        subscriber,
        AttributeValueKey::to_subscription(ctx, subscription).await?,
        func_id,
        Reason::new_user_added(ctx),
    )
    .await?;
    Ok(())
}

// TODO add a helper to change subscription funcs easily

/// Get the value
pub async fn get(ctx: &DalContext, av: impl AttributeValueKey) -> Result<serde_json::Value> {
    let av_id = id(ctx, av).await?;
    AttributeValue::view(ctx, av_id)
        .await?
        .ok_or(eyre!("Attribute missing value"))
}

/// Check whether the value exists and is set
pub async fn has_value(ctx: &DalContext, av: impl AttributeValueKey) -> Result<bool> {
    match AttributeValueKey::resolve(ctx, av).await? {
        Some(av_id) => Ok(AttributeValue::view(ctx, av_id).await?.is_some()),
        None => Ok(false),
    }
}

/// Set a value (creates it if it doesn't exist)
pub async fn set(
    ctx: &DalContext,
    av: impl AttributeValueKey,
    value: impl Into<serde_json::Value>,
) -> Result<()> {
    let av_id = vivify(ctx, av).await?;
    AttributeValue::update(ctx, av_id, Some(value.into())).await?;
    Ok(())
}

/// Check whether the value exists and is set
pub async fn is_set(ctx: &DalContext, av: impl AttributeValueKey) -> Result<bool> {
    match AttributeValueKey::resolve(ctx, av).await? {
        Some(av_id) => Ok(AttributeValue::component_prototype_id(ctx, av_id)
            .await?
            .is_some()),
        None => Ok(false),
    }
}

/// Unset a value (creates it if it doesn't exist)
pub async fn unset(ctx: &DalContext, av: impl AttributeValueKey) -> Result<()> {
    let av_id = vivify(ctx, av).await?;
    AttributeValue::update(ctx, av_id, None).await?;
    Ok(())
}

///
/// Things that you can pass as attribute values (id, or (component, path))
///
#[allow(async_fn_in_trait)]
pub trait AttributeValueKey {
    ///
    /// Get the AttributeValueId for this key
    ///
    async fn id(ctx: &DalContext, key: Self) -> Result<AttributeValueId>;
    ///
    /// Get the AttributeValueId for this key, or None if it doesn't exist
    ///
    async fn resolve(ctx: &DalContext, key: Self) -> Result<Option<AttributeValueId>>;
    ///
    /// Get the AttributeValueId for this key, *or create it* if it doesn't exist
    ///
    async fn vivify(ctx: &DalContext, key: Self) -> Result<AttributeValueId>;
    ///
    /// Turn this into a subscription (resolves component but not av)
    ///
    async fn to_subscription(ctx: &DalContext, key: Self) -> Result<ValueSubscription>;
}
impl AttributeValueKey for AttributeValueId {
    async fn id(_: &DalContext, key: Self) -> Result<AttributeValueId> {
        Ok(key)
    }
    async fn resolve(_: &DalContext, key: Self) -> Result<Option<AttributeValueId>> {
        Ok(Some(key))
    }
    async fn vivify(_: &DalContext, key: Self) -> Result<AttributeValueId> {
        Ok(key)
    }
    async fn to_subscription(ctx: &DalContext, key: Self) -> Result<ValueSubscription> {
        let (root, path) = AttributeValue::path_from_root(ctx, key).await?;
        let path: &str = &path;
        AttributeValueKey::to_subscription(ctx, (root, path)).await
    }
}
impl AttributeValueKey for ValueSubscription {
    async fn id(ctx: &DalContext, key: Self) -> Result<AttributeValueId> {
        key.resolve(ctx)
            .await?
            .ok_or(eyre!("Attribute value not found"))
    }
    async fn resolve(ctx: &DalContext, key: Self) -> Result<Option<AttributeValueId>> {
        Ok(key.resolve(ctx).await?)
    }
    async fn vivify(ctx: &DalContext, key: Self) -> Result<AttributeValueId> {
        Ok(key.path.vivify(ctx, key.attribute_value_id).await?)
    }
    async fn to_subscription(_: &DalContext, key: Self) -> Result<ValueSubscription> {
        Ok(key)
    }
}
impl AttributeValueKey for (AttributeValueId, &str) {
    async fn id(ctx: &DalContext, key: Self) -> Result<AttributeValueId> {
        let sub = AttributeValueKey::to_subscription(ctx, key).await?;
        AttributeValueKey::id(ctx, sub).await
    }
    async fn resolve(ctx: &DalContext, key: Self) -> Result<Option<AttributeValueId>> {
        let sub = AttributeValueKey::to_subscription(ctx, key).await?;
        AttributeValueKey::resolve(ctx, sub).await
    }
    async fn vivify(ctx: &DalContext, key: Self) -> Result<AttributeValueId> {
        let sub = AttributeValueKey::to_subscription(ctx, key).await?;
        AttributeValueKey::vivify(ctx, sub).await
    }
    async fn to_subscription(_: &DalContext, key: Self) -> Result<ValueSubscription> {
        Ok(ValueSubscription {
            attribute_value_id: key.0,
            path: AttributePath::from_json_pointer(key.1),
        })
    }
}
impl<T: ComponentKey> AttributeValueKey for (T, &str) {
    async fn id(ctx: &DalContext, key: Self) -> Result<AttributeValueId> {
        let sub = AttributeValueKey::to_subscription(ctx, key).await?;
        AttributeValueKey::id(ctx, sub).await
    }
    async fn resolve(ctx: &DalContext, key: Self) -> Result<Option<AttributeValueId>> {
        let sub = AttributeValueKey::to_subscription(ctx, key).await?;
        AttributeValueKey::resolve(ctx, sub).await
    }
    async fn vivify(ctx: &DalContext, key: Self) -> Result<AttributeValueId> {
        let sub = AttributeValueKey::to_subscription(ctx, key).await?;
        AttributeValueKey::vivify(ctx, sub).await
    }
    async fn to_subscription(ctx: &DalContext, key: Self) -> Result<ValueSubscription> {
        let component_id = component::id(ctx, key.0).await?;
        let root_id = Component::root_attribute_value_id(ctx, component_id).await?;
        AttributeValueKey::to_subscription(ctx, (root_id, key.1)).await
    }
}
