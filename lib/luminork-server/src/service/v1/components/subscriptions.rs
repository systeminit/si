use dal::{
    AttributeValue,
    AttributeValueId,
    Component,
    ComponentId,
    DalContext,
    Func,
    FuncId,
    attribute::{
        path::AttributePath,
        value::subscription::ValueSubscription,
    },
    func::intrinsics::IntrinsicFunc,
    workspace_snapshot::node_weight::{
        NodeWeight,
        reason_node_weight::Reason,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use utoipa::ToSchema;

use super::{
    ComponentReference,
    ComponentsError,
    ComponentsResult,
    resolve_component_reference,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    #[serde(flatten)]
    pub component_ref: ComponentReference,

    pub prop_path: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = String)]
    pub function: Option<FuncReference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_existing_subscriptions: Option<bool>,
}

pub async fn handle_subscription(
    ctx: &dal::DalContext,
    av_to_set: AttributeValueIdent,
    sub: &Subscription,
    component_id: ComponentId,
    component_list: &[ComponentId],
) -> ComponentsResult<()> {
    let target_av_id = av_to_set.vivify(ctx, component_id).await?;

    let source_component_id =
        resolve_component_reference(ctx, &sub.component_ref, component_list).await?;

    let subscription = ValueSubscription {
        attribute_value_id: Component::root_attribute_value_id(ctx, source_component_id).await?,
        path: AttributePath::from_json_pointer(sub.prop_path.clone()),
    };

    subscription.validate(ctx).await?;

    let existing_subscriptions = match sub.keep_existing_subscriptions {
        Some(true) => AttributeValue::subscriptions(ctx, target_av_id).await?,
        Some(false) | None => None,
    };
    let mut subscriptions = existing_subscriptions.unwrap_or(vec![]);
    if !subscriptions.contains(&subscription) {
        subscriptions.push(subscription);
    }

    let maybe_func_id = if let Some(func) = sub.function.clone() {
        func.resolve(ctx).await?
    } else {
        None
    };

    AttributeValue::set_to_subscriptions(
        ctx,
        target_av_id,
        subscriptions,
        maybe_func_id,
        Reason::new_user_added(ctx),
    )
    .await?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Hash)]
#[serde(rename_all = "camelCase")]
pub struct FuncReference(String);

impl FuncReference {
    #[allow(unused)]
    async fn resolve(&self, ctx: &DalContext) -> ComponentsResult<Option<FuncId>> {
        if let Some(id) = self.resolve_as_id(ctx).await? {
            return Ok(Some(id));
        }

        if let Some(func) = IntrinsicFunc::maybe_from_str(&self.0) {
            return Ok(Some(Func::find_intrinsic(ctx, func).await?));
        }

        // Otherwise, try to find it by name
        Ok(None)
    }

    async fn resolve_as_id(&self, ctx: &DalContext) -> ComponentsResult<Option<FuncId>> {
        let Ok(id) = self.0.parse() else {
            return Ok(None);
        };

        let Some(NodeWeight::Func(_)) = ctx.workspace_snapshot()?.get_node_weight_opt(id).await
        else {
            return Ok(None);
        };

        Ok(Some(id))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeValueIdent(String);

impl AttributeValueIdent {
    #[allow(unused)]
    async fn resolve(
        self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentsResult<Option<AttributeValueId>> {
        if let Some(id) = self.resolve_as_id(ctx, component_id).await? {
            return Ok(Some(id));
        }

        let root_id = Component::root_attribute_value_id(ctx, component_id).await?;
        let path = AttributePath::from_json_pointer(self.0);
        Ok(path.resolve(ctx, root_id).await?)
    }

    async fn vivify(
        self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentsResult<AttributeValueId> {
        if let Some(id) = self.resolve_as_id(ctx, component_id).await? {
            return Ok(id);
        }

        let root_id = Component::root_attribute_value_id(ctx, component_id).await?;
        let path = AttributePath::from_json_pointer(&self.0);
        Ok(path.vivify(ctx, root_id).await?)
    }

    async fn resolve_as_id(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentsResult<Option<AttributeValueId>> {
        // If it is not a ulid, we'll try the alternative
        let Ok(id) = self.0.parse() else {
            return Ok(None);
        };
        // If it doesn't exist, we'll try the alternative
        if !ctx.workspace_snapshot()?.node_exists(id).await {
            return Ok(None);
        }
        // If it *does* exist but is from a different component or not from a component,
        // that is a hard error.
        if AttributeValue::component_id(ctx, id).await? != component_id {
            return Err(ComponentsError::AttributeValueNotFromComponent(
                id,
                component_id,
            ));
        }
        Ok(Some(id))
    }
}
