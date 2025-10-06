use std::{
    collections::HashMap,
    result,
};

use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::{
    AuditLogKind,
    PropValueSource,
};
use si_id::{
    AttributeValueId,
    ComponentId,
    FuncId,
};
use telemetry::prelude::*;
extern crate tuple_vec_map;

use super::{
    path::AttributePath,
    prototype::argument::{
        AttributePrototypeArgument,
        static_value::StaticArgumentValue,
    },
    value::subscription::ValueSubscription,
};
use crate::{
    AttributePrototype,
    AttributeValue,
    Component,
    DalContext,
    Func,
    Prop,
    PropKind,
    WsEvent,
    component::resource::ResourceData,
    func::intrinsics::IntrinsicFunc,
    prop::PropError,
    workspace_snapshot::node_weight::{
        NodeWeight,
        reason_node_weight::Reason,
    },
};

pub type Result<T> = result::Result<T, AttributesError>;

#[remain::sorted]
#[derive(thiserror::Error, Debug)]
pub enum AttributesError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] super::prototype::AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] super::prototype::argument::AttributePrototypeArgumentError),
    #[error("attribute $source: {0} has extra fields: {1}")]
    AttributeSourceHasExtraFields(serde_json::Value, serde_json::Value),
    #[error("invalid attribute $source: {0}")]
    AttributeSourceInvalid(serde_json::Value),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] crate::attribute::value::AttributeValueError),
    #[error("attribute value {0} not from component {1}")]
    AttributeValueNotFromComponent(AttributeValueId, ComponentId),
    #[error(
        "cannot update create-only property at path '{0}' when component has a resource attached"
    )]
    CannotUpdateCreateOnlyProperty(String),
    #[error("component error: {0}")]
    Component(#[from] crate::ComponentError),
    #[error("func error: {0}")]
    Func(#[from] crate::FuncError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("source component not found: {0}")]
    SourceComponentNotFound(String),
    #[error("transactions error: {0}")]
    Transactions(#[from] crate::TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] crate::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] crate::WsEventError),
}

#[derive(Debug)]
pub struct AttributeUpdateCounts {
    pub set_count: usize,
    pub unset_count: usize,
    pub subscription_count: usize,
}

/// A set of attributes you want to set, with the values you want to set them to.
///
/// Ensures that users cannot manually update restricted prop trees (secrets, resource_value, quals, etc.)
///
/// - SET constant attribute values by putting the path to the attribute you want to set as the key,
///   and the value you want to set it to on the right.
///   NOTE: This will also enqueue update actions for the component if the preconditions are met
///
///       {
///         "/si/name": "Baby's First Subnet",
///         "/domain/IpAddresses/0": "10.0.0.1",
///         "/domain/Tags/Environment": "production",
///         "/domain/DomainConfig/blah.com/TTL": 3600
///       }
///
/// - REPLACE objects/arrays/maps: of special note, if you set an entire array, map or object,
///   it *replaces* its value, and all existing keys are removed or unset. Another way of saying
///   it: after you do this, the attribute on the left will be exactly equal to the value
///   on the right, nothing more, nothing less.
///
///       {
///         "/domain/Tags": { "Environment": "production" },
///         "/domain/IpAddresses": [ "10.0.0.1", "10.0.0.2" ],
///         "/domain/DomainConfig/blah.com": { "TTL": 3600 },
///         "/domain": { "IpAddresses": [ "10.0.0.1" ] }
///       }
///
/// - APPEND to array using `-` (or by setting the n+1'th element). If you set an array element
///   that doesn't exist yet, it will be created. `-` is a special syntax for "add a new array
///   element with this value," that doesn't require you to know the (the drawback being you
///   can't append multiple elements to the same array in one API using `-`).
///
///   It is an error to create an array element too far off the end of the array, but you can
///   specify multiple separate elements in order if you want. (It is probably easier to replace
///   the whole array in that case.)
///
///       {
///         "/domain/IpAddresses/0": "10.0.0.0",
///         "/domain/IpAddresses/1": "10.0.0.1",
///         "/domain/IpAddresses/2": "10.0.0.2",
///         "/domain/IpAddresses/-": "10.0.0.3"
///       }
///
/// - INSERT to map by setting its value: if you set a map element that hasn't been created yet,
///   it will be created. This will also happen if you set a *field* in a map element that doesn't exist yet (i.e. a
///   map element with object values).
///
///       {
///         "/domain/Tags/Environment": "production",
///         "/domain/DomainConfig/blah.com/TTL": 3600
///       }
///
/// - UNSET a value using `{ "$source": null }`. The value will revert to using its default value.
///   (NOTE: `{ "$source": {} }` unsets the value as well, allowing JS callers to construct the
///   API call using `{ "$source": { value: myValueVariable } }``. If myValue is undefined, it
///   will unset the value, but if it is null, it will set the value to null.
///
///       {
///         "/domain/Timeout": { "$source": null },
///         "/domain/DomainConfig/blah.com/TTL": { "$source": "value" }
///       }
///
/// - REMOVE an array or map element: unsetting an array or map element will remove it from the
///   array or map. The remaining elements will shift over (it won't "leave a hole").
///
///   *Of note: if you want to remove multiple specific array elements, you should pass them in
///   reverse order.*
///
///       {
///         "/domain/Tags/Environment": { "$source": null },
///         "/domain/IpAddresses/2": { "$source": null },
///         "/domain/IpAddresses/1": { "$source": null }
///       }
///
/// - SUBSCRIBE to another attribute's value: this will cause the value to always equal another
///   attribute's value. Components may be specified by their name (which must be globally unique)
///   or ComponentId.
///
///       {
///         "/domain/SubnetId": {
///           "$source": { "component": "ComponentNameOrId", "path": "/resource/SubnetId" }
///         }
///       }
///
///   You may specify a function ID to be used in subscription, to transform the value before setting
///   it to the destination AV.
///
///   If no func argument is passed, the func will be si:Identity.
///
///       {
///         "/domain/SubnetId": {
///           "$source": { "component": "ComponentNameOrId", "path": "/resource/SubnetId", "func": "01JWBMRZAANBHKD2G2S5PZQTMA" }
///         }
///       }
///
/// - ESCAPE HATCH for setting a value: setting an attribute to `{ "$source": { "value": <value> } }`
///   has the same behavior as all the above cases. The reason this exists is, if you happen to
///   have an object with a "$source" key, the existing interface would treat that as an error.
///   This allows you to set that value anyway.
///
///   This is a safer way to "escape" values if you are writing code that sets values generically
///   without knowing their types and can avoid misinterpreted instructions or possibly even
///   avoid injection attacks.
///
///       {
///         "/domain/Tags": {
///           "$source": {
///             "value": { "$source": "ThisTagIsActuallyNamed_$source" }
///           }
///         }
///       }
///
pub async fn update_attributes(
    ctx: &DalContext,
    component_id: ComponentId,
    updates: AttributeSources,
) -> Result<AttributeUpdateCounts> {
    let updates_to_process = updates.validate_user_can_set_directly();
    update_attributes_inner(ctx, component_id, updates_to_process).await
}

/// Simimlar to [`update_attributes`], only skips the validation entirely so we can expose functionality to management operations
/// but not our APIs/users
pub async fn update_attributes_without_validation(
    ctx: &DalContext,
    component_id: ComponentId,
    updates: AttributeSources,
) -> Result<AttributeUpdateCounts> {
    update_attributes_inner(ctx, component_id, updates).await
}

/// Helper function to check if a prop has the "si_create_only_prop" widget option
fn is_create_only_prop(prop: &Prop) -> bool {
    if let Some(widget_options) = &prop.widget_options {
        widget_options
            .iter()
            .any(|option| option.label() == "si_create_only_prop")
    } else {
        false
    }
}

async fn get_before_prop_value_source(
    ctx: &DalContext,
    av_id: AttributeValueId,
) -> Result<Option<PropValueSource>> {
    let prototype_id = AttributeValue::prototype_id(ctx, av_id).await?;
    let args = AttributePrototype::list_arguments(ctx, prototype_id).await?;
    let mut prop_value_sources = Vec::with_capacity(args.len());
    for arg_id in args {
        let source = AttributePrototypeArgument::value_source(ctx, arg_id).await?;
        let prop_source = match source {
            super::prototype::argument::value_source::ValueSource::StaticArgumentValue(
                static_argument_value_id,
            ) => {
                let static_argument = StaticArgumentValue::get_by_id(ctx, static_argument_value_id)
                    .await?
                    .value;
                PropValueSource::Value(serde_json::to_value(static_argument)?)
            }
            super::prototype::argument::value_source::ValueSource::ValueSubscription(
                value_subscription,
            ) => match value_subscription.resolve(ctx).await? {
                Some(resolved_av_id) => {
                    let source_component_id =
                        AttributeValue::component_id(ctx, resolved_av_id).await?;
                    let value = AttributeValue::view(ctx, resolved_av_id).await?;
                    PropValueSource::Subscription {
                        value,
                        source_component_id,
                        source_path: value_subscription.path.to_string(),
                    }
                }
                _ => PropValueSource::None,
            },
            _ => PropValueSource::None,
        };
        prop_value_sources.push(prop_source);
    }

    if prop_value_sources.len() > 1 || prop_value_sources.is_empty() {
        // TODO: We need to handle what happens if there's no source or multiple sources
        return Ok(None);
    }

    let source = prop_value_sources.into_iter().next();
    Ok(source)
}

async fn update_attributes_inner(
    ctx: &DalContext,
    component_id: ComponentId,
    updates: AttributeSources,
) -> Result<AttributeUpdateCounts> {
    let mut counts = AttributeUpdateCounts {
        set_count: 0,
        unset_count: 0,
        subscription_count: 0,
    };
    for (av_to_set, value) in updates {
        match value.try_into()? {
            Some(value) => {
                counts.set_count += 1;
                // Create the attribute at the given pa th if it does not exist
                let path = av_to_set.path();
                let target_av_id = av_to_set.clone().vivify(ctx, component_id).await?;

                // Check if this is a create-only property and component has a resource
                let prop_id = AttributeValue::prop_id(ctx, target_av_id).await?;
                let prop = Prop::get_by_id(ctx, prop_id).await?;
                if is_create_only_prop(&prop)
                    && Component::resource_by_id(ctx, component_id)
                        .await?
                        .is_some()
                {
                    return Err(AttributesError::CannotUpdateCreateOnlyProperty(
                        path.to_string(),
                    ));
                }

                let before_value_source = get_before_prop_value_source(ctx, target_av_id).await?;
                let mut after_value_source: Option<PropValueSource> = None;

                match value {
                    Source::Value(value) => {
                        // need to special case resource
                        if path == "/resource" {
                            let resource_data = ResourceData::new(
                                veritech_client::ResourceStatus::Ok,
                                Some(value.to_owned()),
                            );
                            let component = Component::get_by_id(ctx, component_id).await?;
                            component.set_resource(ctx, resource_data).await?;
                            continue;
                        }

                        let kind = AttributeValue::prop_kind(ctx, target_av_id).await?;

                        // If we are passed an object or array, to a JSON field, we JSON stringify it
                        // for the user.
                        let value = match (kind, &value) {
                            (
                                PropKind::Json,
                                serde_json::Value::Object(_) | serde_json::Value::Array(_),
                            ) => serde_json::to_string_pretty(&value)?.into(),
                            _ => value,
                        };

                        let before_value = AttributeValue::get_by_id(ctx, target_av_id)
                            .await?
                            .unprocessed_value(ctx)
                            .await?;
                        AttributeValue::update(ctx, target_av_id, value.to_owned().into()).await?;
                        if before_value.as_ref() != Some(&value) {
                            // Build the Audit Log entry!
                            after_value_source =
                                Some(PropValueSource::Value(serde_json::to_value(value)?));

                            // If the values have changed then we should enqueue an update action
                            // if the values haven't changed then we can skip this update action as it is usually a no-op
                            Component::enqueue_update_action_if_applicable(ctx, target_av_id)
                                .await?;
                        }
                    }
                    Source::Subscription {
                        component: source_component,
                        path: source_path,
                        func: func_ident,
                        _keep_existing_subscriptions,
                    } => {
                        counts.subscription_count += 1;

                        // First resolve the component_id (might be a name), then subscribe to the
                        // given path
                        let source_component_id = source_component
                            .resolve(ctx)
                            .await?
                            .ok_or(AttributesError::SourceComponentNotFound(source_component.0))?;
                        let subscription = ValueSubscription::new(
                            ctx,
                            source_component_id,
                            AttributePath::from_json_pointer(source_path),
                        )
                        .await?;

                        let maybe_func_id = if let Some(func) = func_ident {
                            func.resolve(ctx).await?
                        } else {
                            None
                        };

                        // Subscribe!
                        AttributeValue::set_to_subscription(
                            ctx,
                            target_av_id,
                            subscription.clone(),
                            maybe_func_id,
                            Reason::new_user_added(ctx),
                        )
                        .await?;

                        // Build the after value for the audit log!
                        after_value_source = match subscription.resolve(ctx).await? {
                            Some(resolved_av_id) => {
                                let source_component_id =
                                    AttributeValue::component_id(ctx, resolved_av_id).await?;
                                let view = AttributeValue::view(ctx, resolved_av_id).await?;
                                Some(PropValueSource::Subscription {
                                    value: view,
                                    source_component_id,
                                    source_path: subscription.path.to_string(),
                                })
                            }
                            _ => Some(PropValueSource::None),
                        };
                    }
                }
                ctx.write_audit_log(
                    AuditLogKind::SetAttribute {
                        component_id,
                        attribute_value_id: target_av_id,
                        path: path.to_string(),
                        before_value: before_value_source,
                        after_value: after_value_source,
                    },
                    path.to_string(),
                )
                .await?;
            }
            None => {
                counts.unset_count += 1;

                // Unset or remove the value if it exists
                if let Some(target_av_id) = av_to_set.clone().resolve(ctx, component_id).await? {
                    let before_value_source =
                        get_before_prop_value_source(ctx, target_av_id).await?;

                    let path = av_to_set.path();

                    AttributeValue::ensure_updateable(ctx, target_av_id).await?;
                    if parent_prop_is_map_or_array(ctx, target_av_id).await? {
                        // If the parent is a map or array, remove the value
                        AttributeValue::remove(ctx, target_av_id).await?;
                    } else {
                        // Otherwise, just set it to its default value
                        if AttributeValue::component_prototype_id(ctx, target_av_id)
                            .await?
                            .is_some()
                        {
                            AttributeValue::use_default_prototype(ctx, target_av_id).await?;
                        }
                    }

                    ctx.write_audit_log(
                        AuditLogKind::UnsetAttribute {
                            component_id,
                            attribute_value_id: target_av_id,
                            path: path.to_string(),
                            before_value: before_value_source,
                        },
                        path.to_string(),
                    )
                    .await?;
                }
            }
        }
    }

    // Notify the frontend about the updated attributes
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

    Ok(counts)
}

async fn parent_prop_is_map_or_array(ctx: &DalContext, av_id: AttributeValueId) -> Result<bool> {
    let Some(parent_av_id) = AttributeValue::parent_id(ctx, av_id).await? else {
        return Ok(false);
    };
    let parent_prop_kind = AttributeValue::prop_kind(ctx, parent_av_id).await?;
    Ok(matches!(parent_prop_kind, PropKind::Map | PropKind::Array))
}

/// A list of <path>: <source> pairs, used in attribute update APIs.
/// Preserves order as well as duplicate paths (so you can use `-` multiple times).
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    derive_more::Deref,
    derive_more::Into,
    derive_more::IntoIterator,
)]
pub struct AttributeSources(
    // tuple_vec_map preserves order and allows duplicates
    #[serde(with = "tuple_vec_map")] pub Vec<(AttributeValueIdent, ValueOrSourceSpec)>,
);

impl AttributeSources {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // We allow management functions to operate on the resource tree, but we
    // don't want to expose this to users
    pub fn validate_user_can_set_directly(self) -> Self {
        const IGNORE_PATHS: [&str; 4] =
            ["/code", "/deleted_at", "/qualification", "/resource_value"];

        const ALLOW_WITH_SUBS_ONLY: [&str; 1] = ["/secrets"];

        let mut filtered = Vec::new();

        for (av_ident, value_or_source) in self.0 {
            let path = av_ident.path();
            let should_ignore = IGNORE_PATHS
                .iter()
                .any(|ignore_path| path.starts_with(ignore_path));

            if should_ignore {
                warn!(
                    "Ignoring attempt to set value for restricted path: {}",
                    path
                );
                continue;
            }

            let is_subs_only = ALLOW_WITH_SUBS_ONLY
                .iter()
                .any(|sub_only_path| path.starts_with(sub_only_path));
            if is_subs_only && !matches!(value_or_source, ValueOrSourceSpec::SourceSpec(_)) {
                warn!("Attempting to set a raw value for a secret: {path}");
                continue;
            }

            filtered.push((av_ident, value_or_source));
        }

        Self(filtered)
    }
}

/// The source for a value
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, derive_more::From)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Source {
    // { value: <value> } - set value (null is a valid value to set it to)
    Value(serde_json::Value),

    // { component: "ComponentNameOrId", path: "/domain/Foo/Bar/0/Baz" } - subscribe this value to a path from a component
    #[serde(untagged, rename_all = "camelCase")]
    Subscription {
        component: ComponentIdent,
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        func: Option<FuncIdent>,
        // DEPRECATED and ignored. But kept here until we're sure callers are not using it
        #[serde(
            rename = "keep_existing_subscriptions",
            skip_serializing_if = "Option::is_none"
        )]
        _keep_existing_subscriptions: Option<bool>,
    },
}

/// Either raw value or a { "$source": ... } spec (JSON for the source/value for an attribute)
/// Use TryInto<Option<Source> to get Source out of it. If $source is set wrong, you will BadSourceSpecError.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged, rename = "camelCase")]
pub enum ValueOrSourceSpec {
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

/// { $source: <source> }. Separated from ValueOrSourceSpec so we could use deny_unknown_fields
#[derive(Serialize, Deserialize, Clone, Debug, derive_more::From)]
#[serde(rename = "camelCase", deny_unknown_fields)]
pub struct SourceSpec {
    #[serde(rename = "$source")]
    source: MaybeSource,
}

/// Source or "unset" ({} or null). Used mainly for JSON deserialization.
/// Use Into<Option<Source>> to get the real source.
#[derive(Serialize, Deserialize, Clone, Debug, derive_more::From)]
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
    type Error = AttributesError;
    fn try_from(from: ValueOrSourceSpec) -> Result<Self> {
        match from {
            ValueOrSourceSpec::SourceSpec(SourceSpec { source }) => Ok(source.into()),
            ValueOrSourceSpec::BadSourceSpec { source, extra } => {
                if extra.as_object().is_none_or(|o| o.is_empty()) {
                    Err(AttributesError::AttributeSourceInvalid(source))
                } else {
                    Err(AttributesError::AttributeSourceHasExtraFields(
                        source, extra,
                    ))
                }
            }
            ValueOrSourceSpec::RawValue(value) => Ok(Some(Source::Value(value))),
        }
    }
}

/// Convert from (path, source) to JS-capable AttributeSources
/// (particularly, this will escape { <attr>: <value> } to { $source: { value: <value> } } if
/// the value is an object with a $source key).
impl<I: Into<AttributeValueIdent>, S: Into<ValueOrSourceSpec>> From<Vec<(I, S)>>
    for AttributeSources
{
    fn from(pairs: Vec<(I, S)>) -> Self {
        AttributeSources(
            pairs
                .into_iter()
                .map(|(path, source)| (path.into(), source.into()))
                .collect(),
        )
    }
}

impl From<Source> for ValueOrSourceSpec {
    fn from(source: Source) -> Self {
        match source {
            // If it's an object with $source as a key, "escape" it as { $source: <value> }
            Source::Value(value) => value.into(),
            Source::Subscription { .. } => ValueOrSourceSpec::SourceSpec(SourceSpec {
                source: MaybeSource::Source(source),
            }),
        }
    }
}

impl From<serde_json::Value> for ValueOrSourceSpec {
    fn from(value: serde_json::Value) -> Self {
        // If it's an object with $source as a key, "escape" it as { $source: <value> }
        if value.as_object().is_some_and(|o| o.contains_key("$source")) {
            return ValueOrSourceSpec::SourceSpec(SourceSpec {
                source: MaybeSource::Source(value.into()),
            });
        }
        ValueOrSourceSpec::RawValue(value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, derive_more::From)]
#[serde(rename_all = "camelCase")]
pub struct ComponentIdent(String);

impl From<ComponentId> for ComponentIdent {
    fn from(id: ComponentId) -> Self {
        Self(id.to_string())
    }
}

impl From<ComponentIdent> for String {
    fn from(component_ident: ComponentIdent) -> Self {
        component_ident.0
    }
}

impl ComponentIdent {
    pub async fn resolve(&self, ctx: &DalContext) -> Result<Option<ComponentId>> {
        if let Some(id) = self.resolve_as_id(ctx).await? {
            return Ok(Some(id));
        }
        // Otherwise, try to find it by name
        Ok(Component::find_by_name(ctx, &self.0).await?)
    }

    async fn resolve_as_id(&self, ctx: &DalContext) -> Result<Option<ComponentId>> {
        // If it is not a ulid, we'll try the alternative
        let Ok(id) = self.0.parse() else {
            return Ok(None);
        };

        let Some(NodeWeight::Component(_)) =
            ctx.workspace_snapshot()?.get_node_weight_opt(id).await
        else {
            return Ok(None);
        };

        Ok(Some(id))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, derive_more::From)]
#[serde(rename_all = "camelCase")]
pub struct FuncIdent(String);

impl From<FuncId> for FuncIdent {
    fn from(id: FuncId) -> Self {
        Self(id.to_string())
    }
}

impl From<FuncIdent> for String {
    fn from(ident: FuncIdent) -> Self {
        ident.0
    }
}

impl FuncIdent {
    #[allow(unused)]
    pub async fn resolve(&self, ctx: &DalContext) -> Result<Option<FuncId>> {
        if let Some(id) = self.resolve_as_id(ctx).await? {
            return Ok(Some(id));
        }

        if let Some(func) = IntrinsicFunc::maybe_from_str(&self.0) {
            return Ok(Some(Func::find_intrinsic(ctx, func).await?));
        }

        // Otherwise, try to find it by name
        Ok(None)
    }

    async fn resolve_as_id(&self, ctx: &DalContext) -> Result<Option<FuncId>> {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, derive_more::From)]
#[serde(rename = "camelCase")]
pub struct AttributeValueIdent(String);

impl From<AttributeValueId> for AttributeValueIdent {
    fn from(id: AttributeValueId) -> Self {
        Self(id.to_string())
    }
}

impl From<AttributeValueIdent> for String {
    fn from(ident: AttributeValueIdent) -> Self {
        ident.0
    }
}

impl AttributeValueIdent {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn path(&self) -> &str {
        &self.0
    }

    pub async fn resolve(
        self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> Result<Option<AttributeValueId>> {
        if let Some(id) = self.resolve_as_id(ctx, component_id).await? {
            return Ok(Some(id));
        }

        let root_id = Component::root_attribute_value_id(ctx, component_id).await?;
        let path = AttributePath::from_json_pointer(self.0);
        Ok(path.resolve(ctx, root_id).await?)
    }

    async fn vivify(self, ctx: &DalContext, component_id: ComponentId) -> Result<AttributeValueId> {
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
    ) -> Result<Option<AttributeValueId>> {
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
            return Err(AttributesError::AttributeValueNotFromComponent(
                id,
                component_id,
            ));
        }
        Ok(Some(id))
    }
}
