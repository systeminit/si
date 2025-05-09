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
// - UNSET a value using `{ "$source": null }`. The value will revert to using its default value.
//   (NOTE: `{ "$source": {} }` unsets the value as well, allowing JS callers to construct the
//   API call using `{ "$source": { value: myValueVariable } }``. If myValue is undefined, it
//   will unset the value, but if it is null, it will set the value to null.
//
//       {
//         "/domain/Timeout": { "$source": null },
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
//         "/domain/Tags/Environment": { "$source": null },
//         "/domain/IpAddresses/2": { "$source": null },
//         "/domain/IpAddresses/1": { "$source": null }
//       }
//
// - SUBSCRIBE to another attribute's value: this will cause the value to always equal another
//   attribute's value. Components may be specified by their name (which must be globally unique)
//   or ComponentId.
//
//       {
//         "/domain/SubnetId": {
//           "$source": { "component": "ComponentNameOrId", "path": "/resource/SubnetId" }
//         }
//       }
//
// - ESCAPE HATCH for setting a value: setting an attribute to `{ "$source": { "value": <value> } }`
//   has the same behavior as all the above cases. The reason this exists is, if you happen to
//   have an object with a "$source" key, the existing interface would treat that as an error.
//   This allows you to set that value anyway.
//
//   This is a safer way to "escape" values if you are writing code that sets values generically
//   without knowing their types and can avoid misinterpreted instructions or possibly even
//   avoid injection attacks.
//
//       {
//         "/domain/Tags": {
//           "$source": {
//             "value": { "$source": "ThisTagIsActuallyNamed_$source" }
//           }
//         }
//       }
//
async fn update_attributes(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentIdFromPath { component_id }): Path<ComponentIdFromPath>,
    Json(updates): Json<HashMap<String, ValueOrSourceSpec>>,
) -> Result<ForceChangeSetResponse<()>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;
    let target_root_id = Component::root_attribute_value_id(ctx, component_id).await?;
    let mut set_count = 0;
    let mut unset_count = 0;
    let mut subscription_count = 0;
    for (target_path, value) in updates {
        let target_path = AttributePath::from_json_pointer(&target_path);
        match value.try_into()? {
            Some(value) => {
                set_count += 1;

                // Create or update the attribute at the given path
                let target_av_id = target_path.vivify(ctx, target_root_id).await?;
                match value {
                    Source::Value(value) => {
                        AttributeValue::update(ctx, target_av_id, value.into()).await?
                    }
                    Source::Subscription {
                        component: source_component,
                        path: source_path,
                    } => {
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
                        let source_root_prop_id =
                            AttributeValue::prop_id(ctx, source_root_id).await?;
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
            None => {
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
    let Some(parent_av_id) = AttributeValue::parent_id(ctx, av_id).await? else {
        return Ok(false);
    };
    let parent_prop = AttributeValue::prop(ctx, parent_av_id).await?;
    Ok(matches!(parent_prop.kind, PropKind::Map | PropKind::Array))
}

// The source for a value
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
enum Source {
    // { value: <value> } - set value (null is a valid value to set it to)
    Value(serde_json::Value),

    // { component: "ComponentNameOrId", path: "/domain/Foo/Bar/0/Baz" } - subscribe this value to a path from a component
    #[serde(untagged)]
    Subscription {
        component: ComponentIdent,
        path: String,
    },
}

/// Either raw value or a { "$source": ... } spec (JSON for the source/value for an attribute)
/// Use TryInto<Option<Source> to get Source out of it. If $source is set wrong, you will BadSourceSpecError.
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged, rename = "camelCase")]
enum ValueOrSourceSpec {
    /// Explicit sources:
    /// - static value: { "$source": { value: ... } }
    /// - subscription: { "$source": { component: "ComponentNameOrId", path: "/domain/Foo/Bar/0/Baz" } }
    /// - unset value: { "$source": null } or { "$source": {} } - unset value
    SourceSpec(SourceSpec),
    /// Catch errors: if it isn't a valid source, but has a "$source" field, treat it as an error
    /// so you get a 400 if you misuse the API
    BadSourceSpec {
        #[serde(rename = "$source")]
        source: serde_json::Value,
        #[serde(flatten)]
        extra: serde_json::Value,
    },
    /// Any other JSON value is accepted and treated the same as { "$source": { "value": <value> } }
    RawValue(serde_json::Value),
}

/// Used in ValueOrSourceSpecJson { "$source": <source> }
#[derive(Deserialize, Clone, Debug)]
#[serde(rename = "camelCase", deny_unknown_fields)]
struct SourceSpec {
    #[serde(rename = "$source")]
    source: MaybeSource,
}

/// Source or "unset" ({} or null). Used mainly for JSON deserialization.
/// Use Into<Option<Source>> to get the real source.
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged, rename = "camelCase", deny_unknown_fields)]
enum MaybeSource {
    Source(Source),
    EmptyObject {},
    Null,
}

impl From<MaybeSource> for Option<Source> {
    fn from(from: MaybeSource) -> Self {
        match from {
            MaybeSource::Source(source) => Some(source),
            MaybeSource::EmptyObject {} | MaybeSource::Null => None,
        }
    }
}

impl TryFrom<ValueOrSourceSpec> for Option<Source> {
    type Error = Error;
    fn try_from(from: ValueOrSourceSpec) -> Result<Self> {
        match from {
            ValueOrSourceSpec::SourceSpec(SourceSpec { source }) => Ok(source.into()),
            ValueOrSourceSpec::BadSourceSpec { source, extra } => {
                if extra.as_object().is_none_or(|o| o.is_empty()) {
                    Err(Error::AttributeSourceInvalid(source))
                } else {
                    Err(Error::AttributeSourceHasExtraFields(source, extra))
                }
            }
            ValueOrSourceSpec::RawValue(value) => Ok(Some(Source::Value(value))),
        }
    }
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
