//! This module contains the ability to construct a [`schema`](PropertyEditorSchema) for a
//! [`SchemaVariant`](crate::SchemaVariant)'s properties.

use serde::{Deserialize, Serialize};
use si_pkg::PropSpecWidgetKind;
use std::collections::{HashMap, VecDeque};
use strum::{AsRefStr, Display, EnumString};

use crate::prop::{PropPath, WidgetOptions};
use crate::property_editor::{PropertyEditorPropId, PropertyEditorResult};
use crate::workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::{DalContext, Prop, PropId, PropKind, SchemaVariantId};

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
    ) -> PropertyEditorResult<Self> {
        let mut props = HashMap::new();
        let mut child_props = HashMap::new();

        // Get the root prop and load it into the work queue.
        let root_prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(["root"]))?;
        let root_prop = Prop::get_by_id(ctx, root_prop_id).await?;
        let root_property_editor_prop = PropertyEditorProp::new(root_prop);
        let root_property_editor_prop_id = root_property_editor_prop.id;
        props.insert(root_property_editor_prop_id, root_property_editor_prop);

        let mut work_queue = VecDeque::from([(root_prop_id, root_property_editor_prop_id)]);
        while let Some((prop_id, property_editor_prop_id)) = work_queue.pop_front() {
            // Collect all child props.
            let mut cache = Vec::new();
            {
                let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
                for child_prop_node_index in workspace_snapshot
                    .outgoing_targets_for_edge_weight_kind(
                        prop_id,
                        EdgeWeightKindDiscriminants::Use,
                    )?
                {
                    if let NodeWeight::Prop(child_prop_weight) =
                        workspace_snapshot.get_node_weight(child_prop_node_index)?
                    {
                        let child_prop_id: PropId = child_prop_weight.id().into();

                        // Skip anything at and under "/root/secret_definition"
                        if prop_id == root_prop_id
                            && child_prop_weight.name() == "secret_definition"
                        {
                            continue;
                        }
                        cache.push(child_prop_id);
                    }
                }
            }

            // Now that we have the child props, prepare the property editor props and load the work queue.
            let mut child_property_editor_prop_ids = Vec::new();
            for child_prop_id in cache {
                // NOTE(nick): we already have the node weight, but I believe we still want to use "get_by_id" to
                // get the content from the store. Perhaps, there's a more efficient way that we can do this.
                let child_prop = Prop::get_by_id(ctx, child_prop_id).await?;
                let child_property_editor_prop = PropertyEditorProp::new(child_prop);

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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorProp {
    pub id: PropertyEditorPropId,
    pub name: String,
    pub kind: PropertyEditorPropKind,
    pub widget_kind: PropertyEditorPropWidgetKind,
    pub doc_link: Option<String>,
    pub documentation: Option<String>,
}

impl PropertyEditorProp {
    pub fn new(prop: Prop) -> PropertyEditorProp {
        PropertyEditorProp {
            id: prop.id.into(),
            name: prop.name,
            kind: prop.kind.into(),
            widget_kind: PropertyEditorPropWidgetKind::new(
                prop.widget_kind,
                prop.widget_options.map(|v| v.to_owned()),
            ),
            doc_link: prop.doc_link.map(Into::into),
            documentation: prop.documentation.map(Into::into),
        }
    }
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PropertyEditorPropKind {
    Array,
    Boolean,
    Integer,
    Map,
    Object,
    String,
}

impl From<PropKind> for PropertyEditorPropKind {
    fn from(prop_kind: PropKind) -> Self {
        match prop_kind {
            PropKind::Array => Self::Array,
            PropKind::Boolean => Self::Boolean,
            PropKind::Integer => Self::Integer,
            PropKind::Object => Self::Object,
            PropKind::String => Self::String,
            PropKind::Map => Self::Map,
        }
    }
}

// TODO(nick,theo,wendy): consider passing "widget options" to _all_ widgets.
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

// NOTE(nick): this was from the old edit fields code, but it's interesting that we have multiple
// widget kind enums? Not important to look at right now, though.
#[remain::sorted]
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
