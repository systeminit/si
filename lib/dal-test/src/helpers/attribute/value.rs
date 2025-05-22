use color_eyre::eyre::eyre;
use dal::{
    AttributeValue,
    Component,
    DalContext,
    attribute::{
        path::AttributePath,
        value::subscription::ValueSubscription,
    },
};
use si_id::{
    AttributeValueId,
    ComponentId,
    FuncId,
};

use crate::{
    Result,
    helpers::component::ComponentKey,
};

///
/// Things that you can pass as attribute values (id, or (component, path))
///
#[allow(async_fn_in_trait)]
pub trait AttributeValueKey {
    ///
    /// Get the AttributeValueId for this key
    ///
    async fn lookup_attribute_value(self, ctx: &DalContext) -> Result<AttributeValueId>;
    ///
    /// Get the AttributeValueId for this key, or None if it doesn't exist
    ///
    async fn resolve_attribute_value(self, ctx: &DalContext) -> Result<Option<AttributeValueId>>;
    ///
    /// Get the AttributeValueId for this key, *or create it* if it doesn't exist
    ///
    async fn vivify_attribute_value(self, ctx: &DalContext) -> Result<AttributeValueId>;
    ///
    /// Turn this into a subscription (resolves component but not av)
    ///
    async fn to_subscription(self, ctx: &DalContext) -> Result<ValueSubscription>;
}
impl AttributeValueKey for AttributeValueId {
    async fn lookup_attribute_value(self, _: &DalContext) -> Result<AttributeValueId> {
        Ok(self)
    }
    async fn resolve_attribute_value(self, _: &DalContext) -> Result<Option<AttributeValueId>> {
        Ok(Some(self))
    }
    async fn vivify_attribute_value(self, _: &DalContext) -> Result<AttributeValueId> {
        Ok(self)
    }
    async fn to_subscription(self, ctx: &DalContext) -> Result<ValueSubscription> {
        let (root, path) = AttributeValue::path_from_root(ctx, self).await?;
        let path: &str = &path;
        (root, path).to_subscription(ctx).await
    }
}
impl AttributeValueKey for ValueSubscription {
    async fn lookup_attribute_value(self, ctx: &DalContext) -> Result<AttributeValueId> {
        self.resolve(ctx)
            .await?
            .ok_or(eyre!("Attribute value not found"))
    }
    async fn resolve_attribute_value(self, ctx: &DalContext) -> Result<Option<AttributeValueId>> {
        Ok(self.resolve(ctx).await?)
    }
    async fn vivify_attribute_value(self, ctx: &DalContext) -> Result<AttributeValueId> {
        Ok(self.path.vivify(ctx, self.attribute_value_id).await?)
    }
    async fn to_subscription(self, _: &DalContext) -> Result<ValueSubscription> {
        Ok(self)
    }
}
impl AttributeValueKey for (AttributeValueId, &str) {
    async fn lookup_attribute_value(self, ctx: &DalContext) -> Result<AttributeValueId> {
        self.to_subscription(ctx)
            .await?
            .lookup_attribute_value(ctx)
            .await
    }
    async fn resolve_attribute_value(self, ctx: &DalContext) -> Result<Option<AttributeValueId>> {
        self.to_subscription(ctx)
            .await?
            .resolve_attribute_value(ctx)
            .await
    }
    async fn vivify_attribute_value(self, ctx: &DalContext) -> Result<AttributeValueId> {
        self.to_subscription(ctx)
            .await?
            .vivify_attribute_value(ctx)
            .await
    }
    async fn to_subscription(self, _: &DalContext) -> Result<ValueSubscription> {
        Ok(ValueSubscription {
            attribute_value_id: self.0,
            path: AttributePath::from_json_pointer(self.1),
        })
    }
}
impl AttributeValueKey for (ComponentId, &str) {
    async fn lookup_attribute_value(self, ctx: &DalContext) -> Result<AttributeValueId> {
        self.to_subscription(ctx)
            .await?
            .lookup_attribute_value(ctx)
            .await
    }
    async fn resolve_attribute_value(self, ctx: &DalContext) -> Result<Option<AttributeValueId>> {
        self.to_subscription(ctx)
            .await?
            .resolve_attribute_value(ctx)
            .await
    }
    async fn vivify_attribute_value(self, ctx: &DalContext) -> Result<AttributeValueId> {
        self.to_subscription(ctx)
            .await?
            .vivify_attribute_value(ctx)
            .await
    }
    async fn to_subscription(self, ctx: &DalContext) -> Result<ValueSubscription> {
        let root_id = Component::root_attribute_value_id(ctx, self.0).await?;
        (root_id, self.1).to_subscription(ctx).await
    }
}
impl AttributeValueKey for (&str, &str) {
    async fn lookup_attribute_value(self, ctx: &DalContext) -> Result<AttributeValueId> {
        self.to_subscription(ctx)
            .await?
            .lookup_attribute_value(ctx)
            .await
    }
    async fn resolve_attribute_value(self, ctx: &DalContext) -> Result<Option<AttributeValueId>> {
        self.to_subscription(ctx)
            .await?
            .resolve_attribute_value(ctx)
            .await
    }
    async fn vivify_attribute_value(self, ctx: &DalContext) -> Result<AttributeValueId> {
        self.to_subscription(ctx)
            .await?
            .vivify_attribute_value(ctx)
            .await
    }
    async fn to_subscription(self, ctx: &DalContext) -> Result<ValueSubscription> {
        let component_id = self.0.lookup_component(ctx).await?;
        (component_id, self.1).to_subscription(ctx).await
    }
}

/// Set the subscriptions on a value
pub async fn subscribe<S: AttributeValueKey>(
    ctx: &DalContext,
    subscriber: impl AttributeValueKey,
    subscriptions: impl IntoIterator<Item = S>,
) -> Result<()> {
    subscribe_with_custom_function(ctx, subscriber, subscriptions, None).await
}

/// Set the subscriptions on a value
pub async fn subscribe_with_custom_function<S: AttributeValueKey>(
    ctx: &DalContext,
    subscriber: impl AttributeValueKey,
    subscriptions: impl IntoIterator<Item = S>,
    func_id: Option<FuncId>,
) -> Result<()> {
    let subscriber = subscriber.vivify_attribute_value(ctx).await?;
    let mut converted_subscriptions = vec![];
    for subscription in subscriptions {
        converted_subscriptions.push(subscription.to_subscription(ctx).await?);
    }
    AttributeValue::set_to_subscriptions(ctx, subscriber, converted_subscriptions, func_id).await?;
    Ok(())
}

// TODO add a helper to change subscription funcs easily

/// Get the value
pub async fn get(ctx: &DalContext, av: impl AttributeValueKey) -> Result<serde_json::Value> {
    let av_id = av.lookup_attribute_value(ctx).await?;
    AttributeValue::view_by_id(ctx, av_id)
        .await?
        .ok_or(eyre!("Attribute value not found"))
}

/// Check whether the value exists and is set
pub async fn has_value(ctx: &DalContext, av: impl AttributeValueKey) -> Result<bool> {
    match av.resolve_attribute_value(ctx).await? {
        Some(av_id) => Ok(AttributeValue::view_by_id(ctx, av_id).await?.is_some()),
        None => Ok(false),
    }
}

/// Set a value (creates it if it doesn't exist)
pub async fn set(
    ctx: &DalContext,
    av: impl AttributeValueKey,
    value: impl Into<serde_json::Value>,
) -> Result<()> {
    let av_id = av.vivify_attribute_value(ctx).await?;
    AttributeValue::update(ctx, av_id, Some(value.into())).await?;
    Ok(())
}

/// Unset a value (creates it if it doesn't exist)
pub async fn unset(ctx: &DalContext, av: impl AttributeValueKey) -> Result<()> {
    let av_id = av.vivify_attribute_value(ctx).await?;
    AttributeValue::update(ctx, av_id, None).await?;
    Ok(())
}
