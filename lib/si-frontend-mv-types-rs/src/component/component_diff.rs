use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ComponentId,
    workspace_snapshot::EntityKind,
};
use tuple_vec_map;

use crate::{
    component::{
        ComponentDiffStatus,
        ComponentTextDiff,
    },
    reference::ReferenceKind,
    secret::Secret,
};

/// Differences between a component in a changeset vs. head
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[mv(
  trigger_entity = EntityKind::Component,
  reference_kind = ReferenceKind::ComponentDiff,
)]
pub struct ComponentDiff {
    /// The component ID
    pub id: ComponentId,
    /// The status of the component (added or modified)
    pub diff_status: ComponentDiffStatus,
    /// Attributes that have changed.
    ///
    /// Only one entry per spot in the tree: if an attribute is here, neither its parents nor
    /// its children will have their own entries.
    // tuple_vec_map preserves order while allowing the JSON representation to be an object
    // (for more ergonomic APIs)
    #[serde(with = "tuple_vec_map")]
    pub attribute_diffs: Vec<(String, AttributeDiff)>,
    // Also include the TextDiff as we always need it when we need this MV
    pub resource_diff: ComponentTextDiff,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(untagged, rename_all = "camelCase", deny_unknown_fields)]
#[allow(clippy::large_enum_variant)]
pub enum AttributeDiff {
    /// This value was added in the current changeset.
    Added { new: AttributeSourceAndValue },
    /// This attribute was removed entirely from the current changeset.
    Removed { old: AttributeSourceAndValue },
    /// Either the source or value has changed (or both).
    Modified {
        new: AttributeSourceAndValue,
        old: AttributeSourceAndValue,
    },
}

/// An attribute's source, as well as its current value.
///
/// - Value:
///
///       {
///           $value: "us-east-1",
///           $source: {
///              value: "us-east-1"
///           }
///       }
///
/// - Subscription:
///
///       {
///            $value: "us-east-1",
///            $source: {
///                component: "My Region",
///                path: "/domain/region"
///            }
///       }
///
/// - Set by schema (e.g. attribute function):
///
///       {
///           $value: "ami-1234567890EXAMPLE",
///           $source: {
///               fromSchema: true,
///               prototype: "AWS_EC2_AMI:getImageIdFromAws()"
///           }
///       }
///
///       {
///           $value: "Region is us-east-2",
///           $source: {
///               fromSchema: true,
///               fromAncestor: "/domain/Rendered",
///               prototype: "String_Template:renderValue()"
///           }
///       }
///
/// - Set by a parent subscription:
///
///       {
///          $value: "127.0.0.1"
///          $source: {
///              fromAncestor: "/domain/SecurityGroupIngress/3",
///              component: "My Security Group Ingress Rule",
///              path: "/domain",
///          }
///       }
///
/// - Secret subscription:
///
///       {
///           $source: {
///               component: "My Region",
///               path: "/secrets/AWS Credential",
///           },
///           $value: "...",
///           $secret: {
///             name: "My Credential",
///             ...
///           }
///       }
///
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttributeSourceAndValue {
    /// The source of this attribute.
    #[serde(rename = "$source")]
    pub source: AttributeSource,

    /// The computed value of this attribute, *including child values*.
    ///
    /// - If an object field is None (i.e. currentValue is not in the JSON), the field will not
    ///   show up (i.e. None == undefined).
    /// - If an object field is null, the field will show up and have a null value.
    /// - Object, maps and array values will include the child value.
    #[serde(rename = "$value", skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,

    /// Information about the secret, if this points at one.
    #[serde(rename = "$secret", skip_serializing_if = "Option::is_none")]
    pub secret: Option<Secret>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
// TODO add deny_unknown_fields
#[serde(rename_all = "camelCase")]
pub struct AttributeSource {
    // The actual source (value, subscription or prototype).
    #[serde(flatten)]
    pub simplified_source: SimplifiedAttributeSource,

    /// Whether this value came from the schema prototype. Missing or false means it came from
    /// a component prototype.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_schema: Option<bool>,

    /// If this value came from a dynamic prototype or subscription on a parent/ancestor
    /// attribute, this will be set to the path of that parent/ancestor. If this is unspecified,
    /// the value came from a prototype on this attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_ancestor: Option<String>,
}

/// The source of a value (but not whether it came from the schema, or from a parent)
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(untagged, rename_all = "camelCase", deny_unknown_fields)]
pub enum SimplifiedAttributeSource {
    /// This attribute was set to a value by the user.
    ///
    ///     { value: "us-east-1"}
    ///
    /// If this is an object, map or array, the value will always be {} or [].
    ///
    /// This represents values set by component prototypes:
    /// - si:setString("string"), etc.
    /// - si:setMap({})
    /// - si:setObject({})
    /// - si:setObject([])
    ///
    Value { value: serde_json::Value },

    /// This attribute is set to a subscription.
    ///
    ///     { component: "My Region", path: "/domain/region" }
    ///
    /// This represents values set by si:identity subscription:
    /// - si:identity(arg = /domain/SubnetId on Subnet)
    ///
    Subscription {
        component: ComponentId,
        path: String,
    },

    /// This attribute is set to a complex function, possibly with multiple arguments.
    ///
    ///     { complexPrototype: "si:normalizeToArray(arg = /domain/SubnetId on Subnet)" }
    ///
    /// NOTE: we serialize it so you can read it, but you should not rely on the value or parse
    /// it.
    ///
    Prototype { prototype: String },
}

#[allow(clippy::panic_in_result_fn)]
#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn deserialize_component_diff() -> serde_json::Result<()> {
        let id = ComponentId::new();
        assert_eq!(
            ComponentDiff {
                id,
                diff_status: ComponentDiffStatus::Added,
                attribute_diffs: vec![
                    (
                        "/domain/Foo".to_string(),
                        AttributeDiff::Removed {
                            old: AttributeSourceAndValue {
                                source: AttributeSource {
                                    simplified_source: SimplifiedAttributeSource::Value {
                                        value: json!("foo")
                                    },
                                    from_schema: None,
                                    from_ancestor: None,
                                },
                                value: Some(json!("foo")),
                                secret: None,
                            }
                        }
                    ),
                    (
                        "/domain/Bar".to_string(),
                        AttributeDiff::Added {
                            new: AttributeSourceAndValue {
                                source: AttributeSource {
                                    simplified_source: SimplifiedAttributeSource::Value {
                                        value: json!("bar")
                                    },
                                    from_schema: None,
                                    from_ancestor: None,
                                },
                                value: Some(json!("bar")),
                                secret: None,
                            }
                        }
                    ),
                    (
                        "/domain/Baz".to_string(),
                        AttributeDiff::Modified {
                            new: AttributeSourceAndValue {
                                source: AttributeSource {
                                    simplified_source: SimplifiedAttributeSource::Value {
                                        value: json!("new_baz")
                                    },
                                    from_schema: None,
                                    from_ancestor: None,
                                },
                                value: Some(json!("new_baz")),
                                secret: None,
                            },
                            old: AttributeSourceAndValue {
                                source: AttributeSource {
                                    simplified_source: SimplifiedAttributeSource::Value {
                                        value: json!("baz")
                                    },
                                    from_schema: None,
                                    from_ancestor: None,
                                },
                                value: Some(json!("baz")),
                                secret: None,
                            }
                        }
                    ),
                ],
                resource_diff: ComponentTextDiff {
                    current: None,
                    diff: None
                }
            },
            serde_json::from_str(&format!(
                r#"{{ "id": {}, "diffStatus": "Added", "resourceDiff": {{}}, "attributeDiffs": {} }}"#,
                serde_json::to_string(&id)?,
                r#"{
                        "/domain/Foo": {
                            "old": {
                                "$source": { "value": "foo" },
                                "$value": "foo"
                            }
                        },
                        "/domain/Bar": {
                            "new": {
                                "$source": { "value": "bar" },
                                "$value": "bar"
                            }
                        },
                        "/domain/Baz": {
                            "new": {
                                "$source": { "value": "new_baz" },
                                "$value": "new_baz"
                            },
                            "old": {
                                "$source": { "value": "baz" },
                                "$value": "baz"
                            }
                        }
                    }"#
            ))?
        );

        Ok(())
    }

    #[test]
    fn deserialize_attribute_source_and_value() -> Result<(), serde_json::Error> {
        assert_eq!(
            AttributeSourceAndValue {
                source: AttributeSource {
                    simplified_source: SimplifiedAttributeSource::Value {
                        value: json!("foo")
                    },
                    from_schema: None,
                    from_ancestor: None,
                },
                value: Some(json!("foo")),
                secret: None,
            },
            serde_json::from_str(
                r#"{
                    "$source": { "value": "foo" },
                    "$value": "foo"
                }"#,
            )?
        );

        Ok(())
    }

    #[test]
    fn deserialize_attribute_source() -> Result<(), serde_json::Error> {
        assert_eq!(
            AttributeSource {
                simplified_source: SimplifiedAttributeSource::Value {
                    value: json!("foo")
                },
                from_schema: None,
                from_ancestor: None,
            },
            serde_json::from_str(r#"{ "value": "foo" }"#)?
        );

        Ok(())
    }

    #[test]
    fn deserialize_simplified_attribute_source() -> Result<(), serde_json::Error> {
        assert_eq!(
            SimplifiedAttributeSource::Value {
                value: json!("foo")
            },
            serde_json::from_str(r#"{ "value": "foo" }"#)?
        );

        Ok(())
    }
}
