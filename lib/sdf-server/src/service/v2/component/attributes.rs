use std::collections::HashMap;

use axum::{
    Json,
    Router,
    extract::Path,
    routing::put,
};
use dal::{
    AttributeValue,
    AttributeValueId,
    ChangeSet,
    Component,
    DalContext,
    PropKind,
    WsEvent,
    attribute::{
        path::AttributePath,
        value::subscription::ValueSubscription,
    },
};
use sdf_core::force_change_set_response::ForceChangeSetResponse;
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::Deserialize;
use serde_json::json;
use si_id::ComponentId;

use super::{
    ComponentIdFromPath,
    Error,
    Result,
};
use crate::app_state::AppState;

pub fn v2_routes() -> Router<AppState> {
    Router::new().route("/", put(update_attributes))
}

// A set of attributes you want to set, with the values you want to set them to.
//
// - SET constant attribute values by putting the path to the attribute you want to set as the key,
//   and the value you want to set it to on the right.
//
//       {
//         "/si/name": "Baby's First Subnet",
//         "/domain/IpAddresses/0": "10.0.0.1",
//         "/domain/Tags/Environment": "production",
//         "/domain/DomainConfig/blah.com/TTL": 3600
//       }
//
// - REPLACE objects/arrays/maps: of special note, if you set an entire array, map or object,
//   it *replaces* its value, and all existing keys are removed or unset. Another way of saying
//   it: after you do this, the attribute on the left will be exactly equal to the value
//   on the right, nothing more, nothing less.
//
//     {
//       "/domain/Tags": { "Environment": "production" },
//       "/domain/IpAddresses": [ "10.0.0.1", "10.0.0.2" ],
//       "/domain/DomainConfig/blah.com": { "TTL": 3600 },
//       "/domain": { "IpAddresses": [ "10.0.0.1" ] }
//     }
//
// - APPEND to array using `-` (or by setting the n+1'th element). If you set an array element
//   that doesn't exist yet, it will be created. `-` is a special syntax for "add a new array
//   element with this value," that doesn't require you to know the (the drawback being you
//   can't append multiple elements to the same array in one API using `-`).
//
//   It is an error to create an array element too far off the end of the array, but you can
//   specify multiple separate elements in order if you want. (It is probably easier to replace
//   the whole array in that case.)
//
//       {
//         "/domain/IpAddresses/0": "10.0.0.0",
//         "/domain/IpAddresses/1": "10.0.0.1",
//         "/domain/IpAddresses/2": "10.0.0.2",
//         "/domain/IpAddresses/-": "10.0.0.3"
//       }
//
// - INSERT to map by setting its value: if you set a map element that hasn't been created yet,
//   it will be created. This will also happen if you set a *field* in a map element that doesn't exist yet (i.e. a
//   map element with object values).
//
//       {
//         "/domain/Tags/Environment": "production",
//         "/domain/DomainConfig/blah.com/TTL": 3600
//       }
//
// - UNSET a value using `{ "$source": "value" }`. The value will revert to using its default value.
//
//       {
//         "/domain/Timeout": { "$source": "value" },
//         "/domain/DomainConfig/blah.com/TTL": { "$source": "value" }
//       }
//
// - REMOVE an array or map element: unsetting an array or map element will remove it from the
//   array or map. The remaining elements will shift over (it won't "leave a hole").
//
//   *Of note: if you want to remove multiple specific array elements, you should pass them in
//   reverse order.*
//
//       {
//         "/domain/Tags/Environment": { "$source": "value" },
//         "/domain/IpAddresses/2": { "$source": "value" },
//         "/domain/IpAddresses/1": { "$source": "value" }
//       }
//
// - SUBSCRIBE to another attribute's value: this will cause the value to always equal another
//   attribute's value. Components may be specified by their name (which must be globally unique)
//   or ComponentId.
//
//       {
//         "/domain/SubnetId": {
//           "$source": "subscription",
//           "component": "ComponentNameOrId",
//           "path": "/resource/SubnetId"
//         }
//       }
//
// - ESCAPE HATCH for setting a value: setting an attribute to `{ "$source": "value", "value": <value> }`
//   has the same behavior as all the above cases. The reason this exists is, if you happen to
//   have an object whose keys are "$source" and "value", the existing interface would treat that
//
//   This is a safer way to "escape" values if you are writing code that sets values generically
//   without knowing their types and can avoid misinterpreted instructions or possibly even
//   avoid injection attacks.
//
//       {
//         "/domain/Tags": {
//           "$source": "value",
//           "value": { "Environment": "Prod", "$source": "ThisTagIsActuallyNamed_$source" }
//         }
//       }
//
async fn update_attributes(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentIdFromPath { component_id }): Path<ComponentIdFromPath>,
    Json(updates): Json<HashMap<String, SetTo>>,
) -> Result<ForceChangeSetResponse<()>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;
    let target_root_id = Component::root_attribute_value_id(ctx, component_id).await?;
    let mut set_count = 0;
    let mut unset_count = 0;
    let mut subscription_count = 0;
    for (target_path, value) in updates {
        let target_path = AttributePath::from_json_pointer(&target_path);
        match value {
            SetTo::Value { value } | SetTo::UntaggedValue(value) => match value {
                SetToValue::Set(value) => {
                    set_count += 1;

                    // Create or update the value
                    let target_av_id = target_path.vivify(ctx, target_root_id).await?;
                    AttributeValue::update(ctx, target_av_id, value.into()).await?
                }
                SetToValue::Unset => {
                    unset_count += 1;

                    // Unset or remove the value if it exists
                    if let Some(target_av_id) = target_path.resolve(ctx, target_root_id).await? {
                        if parent_prop_is_map_or_array(ctx, target_av_id).await? {
                            // If the parent is a map or array, remove the value
                            AttributeValue::remove_by_id(ctx, target_av_id).await?;
                        } else {
                            // Otherwise, just set it to its default value
                            if AttributeValue::component_prototype_id(ctx, target_av_id)
                                .await?
                                .is_some()
                            {
                                AttributeValue::use_default_prototype(ctx, target_av_id).await?;
                            }
                        }
                    }
                }
            },
            SetTo::Subscription {
                component: source_component,
                path: source_path,
            } => {
                set_count += 1;
                subscription_count += 1;

                // Look up (or create) the AV based on its path
                let target_av_id = target_path.vivify(ctx, target_root_id).await?;

                // First resolve the component_id (might be a name), then subscribe to the
                // given path
                let source_component_id = source_component
                    .resolve(ctx)
                    .await?
                    .ok_or(Error::SourceComponentNotFound(source_component.0))?;
                let source_root_id =
                    Component::root_attribute_value_id(ctx, source_component_id).await?;

                // Make sure the subscribed-to path is valid (i.e. it doesn't have to resolve
                // to a value *right now*, but it must be a valid path to the schema as it
                // exists--correct prop names, numeric indices for arrays, etc.)
                let source_path = AttributePath::from_json_pointer(source_path);
                let source_root_prop_id = AttributeValue::prop_id(ctx, source_root_id).await?;
                source_path.validate(ctx, source_root_prop_id).await?;

                // Subscribe!
                AttributeValue::subscribe(
                    ctx,
                    target_av_id,
                    ValueSubscription {
                        attribute_value_id: source_root_id,
                        path: source_path,
                    },
                )
                .await?;
            }
        }
    }

    let component = Component::get_by_id(ctx, component_id).await?;

    let mut socket_map = HashMap::new();
    let payload = component
        .into_frontend_type(
            ctx,
            None,
            component.change_status(ctx).await?,
            &mut socket_map,
        )
        .await?;
    WsEvent::component_updated(ctx, payload)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit().await?;

    tracker.track(
        ctx,
        "component_attributes_updated",
        json!({
            "how": "/component/attributes",
            "component_id": component_id,
            "change_set_id": ctx.change_set_id(),
            "set_count": set_count,
            "unset_count": unset_count,
            "subscription_count": subscription_count,
        }),
    );
    println!(
        "{}",
        json!({
            "how": "/component/attributes",
            "component_id": component_id,
            "change_set_id": ctx.change_set_id(),
            "set_count": set_count,
            "unset_count": unset_count,
            "subscription_count": subscription_count,
        })
    );

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}

async fn parent_prop_is_map_or_array(ctx: &DalContext, av_id: AttributeValueId) -> Result<bool> {
    let Some(parent_av_id) = AttributeValue::parent_attribute_value_id(ctx, av_id).await? else {
        return Ok(false);
    };
    let parent_prop = AttributeValue::prop(ctx, parent_av_id).await?;
    Ok(matches!(parent_prop.kind, PropKind::Map | PropKind::Array))
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "$source")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
enum SetTo {
    // Set or unset a value for this attribute
    // { "$source": "value", value: <value> } - set value
    // { "$source": "value" } - unset value (this is what happens if you send JS { "$source": "value", "value": undefined })
    Value {
        #[serde(default)]
        value: SetToValue,
    },
    // Subscribe this value to a path from a component
    // { "$source": "subscription", component: "ComponentNameOrId", path: "/domain/Foo/Bar/0/Baz" }
    Subscription {
        component: ComponentIdent,
        path: String,
    },
    // Anything else is treated as a value (treated same as { "$source": "value", value: <value> })
    #[serde(untagged)]
    UntaggedValue(SetToValue),
}

// Like Option<serde_json::Value>, except missing values are treated as None (serde special cases
// Option<serde_json::Value> to treat null as None).
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(untagged)]
enum SetToValue {
    // All actual values (including null)
    Set(serde_json::Value),
    // Missing field is treated as Unset
    #[default]
    Unset,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct ComponentIdent(String);

impl ComponentIdent {
    async fn resolve(&self, ctx: &DalContext) -> Result<Option<ComponentId>> {
        // If it is a guid, try to find it by id
        if let Ok(component_id) = self.0.parse::<ComponentId>() {
            if ctx.workspace_snapshot()?.node_exists(component_id).await {
                return Ok(Some(component_id));
            }
        }
        // Otherwise, try to find it by name
        Ok(Component::find_by_name(ctx, &self.0).await?)
    }
}
