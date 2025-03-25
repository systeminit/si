//! This module contains the ability to construct a [`schema`](PropertyEditorSchema) for a
//! [`SchemaVariant`](crate::SchemaVariant)'s properties.

use serde::{Deserialize, Serialize};
use si_pkg::PropSpecWidgetKind;
use std::collections::{HashMap, VecDeque};
use strum::{AsRefStr, Display, EnumString};

use crate::prop::{PropPath, WidgetOptions};
use crate::property_editor::{PropertyEditorPropId, PropertyEditorResult};
use crate::{DalContext, Prop, PropId, PropKind, SchemaVariant, SchemaVariantId};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorSchema {
    pub root_prop_id: PropertyEditorPropId,
    pub props: HashMap<PropertyEditorPropId, PropertyEditorProp>,
    pub child_props: HashMap<PropertyEditorPropId, Vec<PropertyEditorPropId>>,
}

impl PropertyEditorSchema {
    pub async fn assemble(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        has_resource: bool,
    ) -> PropertyEditorResult<Self> {
        let mut props = HashMap::new();
        let mut child_props = HashMap::new();

        let builder = PropertyEditorPropBuilder::new(ctx, schema_variant_id).await?;

        // Get the root prop and load it into the work queue.
        let root_prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(["root"])).await?;
        let root_prop = Prop::get_by_id(ctx, root_prop_id).await?;
        let root_property_editor_prop = builder.build(ctx, root_prop, has_resource).await?;
        let root_property_editor_prop_id = root_property_editor_prop.id;
        props.insert(root_property_editor_prop_id, root_property_editor_prop);

        let mut work_queue = VecDeque::from([(root_prop_id, root_property_editor_prop_id)]);
        while let Some((prop_id, property_editor_prop_id)) = work_queue.pop_front() {
            // Collect all child props.
            let mut cache = Vec::new();
            {
                let child_props = Prop::direct_child_props_ordered(ctx, prop_id).await?;

                for child_prop in child_props {
                    // Skip anything at and under "/root/secret_definition",
                    // also skip hidden props EXCEPT /root/resource_value
                    if prop_id == root_prop_id && child_prop.name == "secret_definition"
                        || (child_prop.hidden && child_prop.name != "resource_value")
                    {
                        continue;
                    }
                    cache.push(child_prop);
                }
            }

            // Now that we have the child props, prepare the property editor props and load the work queue.
            let mut child_property_editor_prop_ids = Vec::new();
            for child_prop in cache {
                let child_prop_id = child_prop.id;
                let child_property_editor_prop =
                    builder.build(ctx, child_prop, has_resource).await?;

                // Load the work queue with the child prop.
                work_queue.push_back((child_prop_id, child_property_editor_prop.id));

                // Cache the child property editor props to eventually insert into the child property editor props map.
                child_property_editor_prop_ids.push(child_property_editor_prop.id);

                // Insert the child property editor prop into the props map.
                props.insert(child_property_editor_prop.id, child_property_editor_prop);
            }
            child_props.insert(property_editor_prop_id, child_property_editor_prop_ids);
        }

        Ok(PropertyEditorSchema {
            root_prop_id: root_prop_id.into(),
            props,
            child_props,
        })
    }
}

/// A builder for creating [`PropertyEditorProps`](PropertyEditorProp).
#[derive(Debug)]
pub struct PropertyEditorPropBuilder {
    /// If the [`SchemaVariant`] is a secret-defining [`SchemaVariant`], this will be populated
    /// with the [`PropId`](Prop) of the [`Prop`] underneath "/root/secrets" corresponding to
    /// the definition.
    origin_secret_prop_id: Option<PropId>,
}

impl PropertyEditorPropBuilder {
    pub async fn new(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropertyEditorResult<Self> {
        // We need to know if we are secret-defining schema variant and what the corresponding
        // "/root/secrets" prop is. We need to know this because a secret prop can only be edited
        // if it is the origin secret prop on its corresponding secret-defining component.
        let origin_secret_prop_id = if SchemaVariant::is_secret_defining(ctx, schema_variant_id)
            .await?
        {
            let output_socket =
                SchemaVariant::find_output_socket_for_secret_defining_id(ctx, schema_variant_id)
                    .await?;
            Some(
                Prop::find_prop_id_by_path(
                    ctx,
                    schema_variant_id,
                    &PropPath::new(["root", "secrets", output_socket.name()]),
                )
                .await?,
            )
        } else {
            None
        };

        Ok(Self {
            origin_secret_prop_id,
        })
    }

    /// This non-consuming method creates a [`PropertyEditorProp`] with a given [`Prop`].
    pub async fn build(
        &self,
        ctx: &DalContext,
        prop: Prop,
        has_resource: bool,
    ) -> PropertyEditorResult<PropertyEditorProp> {
        let default_can_be_set_by_socket = !prop.input_socket_sources(ctx).await?.is_empty();

        let mut is_create_only = false;
        let filtered_widget_options = prop.widget_options.map(|options| {
            options
                .into_iter()
                .filter(|option| {
                    if option.label() == "si_create_only_prop" {
                        is_create_only = true;
                        false
                    } else {
                        true
                    }
                })
                .collect::<WidgetOptions>()
        });

        Ok(PropertyEditorProp {
            id: prop.id.into(),
            name: prop.name,
            kind: prop.kind.into(),
            widget_kind: PropertyEditorPropWidgetKind::new(
                prop.widget_kind,
                filtered_widget_options,
            ),
            doc_link: prop.doc_link.map(Into::into),
            documentation: prop.documentation.map(Into::into),
            validation_format: prop.validation_format.map(Into::into),
            default_can_be_set_by_socket,
            is_origin_secret: match self.origin_secret_prop_id {
                Some(prop_id) => prop_id == prop.id,
                None => false,
            },
            create_only: is_create_only && has_resource,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorProp {
    pub id: PropertyEditorPropId,
    pub name: String,
    pub kind: PropertyEditorPropKind,
    pub widget_kind: PropertyEditorPropWidgetKind,
    pub doc_link: Option<String>,
    pub documentation: Option<String>,
    pub validation_format: Option<String>,
    pub default_can_be_set_by_socket: bool,
    pub is_origin_secret: bool,
    pub create_only: bool,
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PropertyEditorPropKind {
    Array,
    Boolean,
    Float,
    Integer,
    Json,
    Map,
    Object,
    String,
}

impl From<PropKind> for PropertyEditorPropKind {
    fn from(prop_kind: PropKind) -> Self {
        match prop_kind {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Boolean,
            PropKind::Float => Self::Float,
            PropKind::Integer => Self::Integer,
            PropKind::Json => Self::Json,
            PropKind::Object => Self::Object,
            PropKind::String => Self::String,
            PropKind::Map => Self::Map,
        }
    }
}

#[remain::sorted]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PropertyEditorPropWidgetKind {
    Array,
    Checkbox,
    CodeEditor,
    Color,
    ComboBox { options: Option<WidgetOptions> },
    Header,
    Map,
    Password,
    Secret { options: Option<WidgetOptions> },
    Select { options: Option<WidgetOptions> },
    Text,
    TextArea,
}

/// This type goes into the content store. It cannot be reordered, and existing variants cannot be changed. new variants can be added to the end.
#[derive(
    AsRefStr, Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Display, EnumString, Copy,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum WidgetKind {
    Array,
    Checkbox,
    CodeEditor,
    Color,
    /// Provides a text input with auto-completion for corresponding "primitive" (e.g. string, number, boolean)
    /// [`PropKinds`](crate::PropKind).
    ComboBox,
    Header,
    Map,
    Password,
    Secret,
    /// Provides a select box for corresponding "primitive" (e.g. string, number, boolean)
    /// [`PropKinds`](crate::PropKind).
    Select,
    Text,
    TextArea,
}

impl From<WidgetKind> for PropSpecWidgetKind {
    fn from(value: WidgetKind) -> Self {
        match value {
            WidgetKind::Array => Self::Array,
            WidgetKind::Checkbox => Self::Checkbox,
            WidgetKind::CodeEditor => Self::CodeEditor,
            WidgetKind::Header => Self::Header,
            WidgetKind::Map => Self::Map,
            WidgetKind::Password => Self::Password,
            WidgetKind::Select => Self::Select,
            WidgetKind::Color => Self::Color,
            WidgetKind::Secret => Self::Secret,
            WidgetKind::Text => Self::Text,
            WidgetKind::TextArea => Self::TextArea,
            WidgetKind::ComboBox => Self::ComboBox,
        }
    }
}

impl From<&PropSpecWidgetKind> for WidgetKind {
    fn from(value: &PropSpecWidgetKind) -> Self {
        match value {
            PropSpecWidgetKind::Array => Self::Array,
            PropSpecWidgetKind::Checkbox => Self::Checkbox,
            PropSpecWidgetKind::CodeEditor => Self::CodeEditor,
            PropSpecWidgetKind::Header => Self::Header,
            PropSpecWidgetKind::Map => Self::Map,
            PropSpecWidgetKind::Password => Self::Password,
            PropSpecWidgetKind::Select => Self::Select,
            PropSpecWidgetKind::Color => Self::Color,
            PropSpecWidgetKind::Secret => Self::Secret,
            PropSpecWidgetKind::Text => Self::Text,
            PropSpecWidgetKind::TextArea => Self::TextArea,
            PropSpecWidgetKind::ComboBox => Self::ComboBox,
        }
    }
}

impl PropertyEditorPropWidgetKind {
    pub fn new(widget_kind: WidgetKind, widget_options: Option<WidgetOptions>) -> Self {
        match widget_kind {
            WidgetKind::Array => Self::Array,
            WidgetKind::Checkbox => Self::Checkbox,
            WidgetKind::CodeEditor => Self::CodeEditor,
            WidgetKind::Header => Self::Header,
            WidgetKind::Map => Self::Map,
            WidgetKind::Password => Self::Password,
            WidgetKind::Select => Self::Select {
                options: widget_options,
            },
            WidgetKind::Color => Self::Color,
            WidgetKind::Secret => Self::Secret {
                options: widget_options,
            },
            WidgetKind::Text => Self::Text,
            WidgetKind::TextArea => Self::TextArea,
            WidgetKind::ComboBox => Self::ComboBox {
                options: widget_options,
            },
        }
    }
}
