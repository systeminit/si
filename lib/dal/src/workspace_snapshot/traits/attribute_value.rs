use std::collections::{
    HashMap,
    VecDeque,
};

use async_trait::async_trait;
use indexmap::IndexMap;
use serde_json::json;
use si_id::{
    AttributePrototypeId,
    AttributeValueId,
};

use crate::{
    DalContext,
    PropKind,
    WorkspaceSnapshot,
    WorkspaceSnapshotError,
    attribute::value::AttributeValueResult,
    workspace_snapshot::{
        graph::traits::attribute_value::{
            AttributeValueExt as _,
            AttributeValueTree,
            AttributeValueTreeEntry,
        },
        traits::prop::PropExt as _,
    },
};

/// Builds the value for complex types (Array, Map, Object) by calling `build_complex_value`
/// if the AttributeValue's value is None, but it has children. This is used when the AttributeValue
/// has a value, which means even if the value has no children, we should return the default empty value
async fn build_complex_value_or_default(
    workspace_snapshot: &WorkspaceSnapshot,
    av_tree_entry: &AttributeValueTreeEntry,
    av_tree: &AttributeValueTree,
    sub_views: &mut HashMap<AttributeValueId, serde_json::Value>,
) -> AttributeValueResult<serde_json::Value> {
    // If the AttributeValue has children with relevant data to thread through, use it.
    // Otherwise, return the default empty value for the type.
    if let Some(value) =
        build_complex_value(workspace_snapshot, av_tree_entry, av_tree, sub_views).await?
    {
        Ok(value)
    } else {
        let default = match av_tree_entry.prop_kind {
            PropKind::Array => json!([]),
            PropKind::Object => json!({}),
            PropKind::Map => json!({}),
            _ => serde_json::Value::Null,
        };
        Ok(default)
    }
}

/// Builds the value for complex types (Array, Map, Object) by looking up their children in the
/// provided `sub_views` map. Returns the entries if we'd already determined we needed to build the children,
/// otherwise returns None.
async fn build_complex_value(
    workspace_snapshot: &WorkspaceSnapshot,
    av_tree_entry: &AttributeValueTreeEntry,
    av_tree: &AttributeValueTree,
    sub_views: &mut HashMap<AttributeValueId, serde_json::Value>,
) -> AttributeValueResult<Option<serde_json::Value>> {
    let value = match av_tree_entry.prop_kind {
        PropKind::Array => {
            let children = av_tree.children_of(av_tree_entry.attribute_value_id);
            let mut entries = Vec::with_capacity(children.len());
            for child_tree_entry in children {
                if let Some(child_value) = sub_views.remove(&child_tree_entry.attribute_value_id) {
                    entries.push(child_value);
                }
            }
            match entries.is_empty() {
                true => None,
                false => Some(serde_json::to_value(entries)?),
            }
        }
        PropKind::Map => {
            let children = av_tree.children_of(av_tree_entry.attribute_value_id);
            let mut entries = IndexMap::with_capacity(children.len());
            for child_tree_entry in children {
                if let Some(key) = workspace_snapshot
                    .working_copy()
                    .await
                    .key_for_attribute_value_id(child_tree_entry.attribute_value_id)?
                {
                    if let Some(child_value) =
                        sub_views.remove(&child_tree_entry.attribute_value_id)
                    {
                        entries.insert(key, child_value);
                    }
                }
            }
            match entries.is_empty() {
                true => None,
                false => Some(serde_json::to_value(entries)?),
            }
        }
        PropKind::Object => {
            let children = av_tree.children_of(av_tree_entry.attribute_value_id);
            let mut entries = IndexMap::with_capacity(children.len());
            for child_tree_entry in children {
                let prop_name = workspace_snapshot
                    .working_copy()
                    .await
                    .get_node_weight_by_id(child_tree_entry.prop_id)?
                    .get_prop_node_weight()?
                    .name()
                    .to_owned();
                if let Some(child_value) = sub_views.remove(&child_tree_entry.attribute_value_id) {
                    entries.insert(prop_name, child_value);
                }
            }
            match entries.is_empty() {
                true => None,
                false => Some(serde_json::to_value(entries)?),
            }
        }
        _ => None,
    };
    Ok(value)
}

#[async_trait]
pub trait AttributeValueExt {
    async fn attribute_value_view(
        &self,
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<serde_json::Value>>;

    async fn component_prototype_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributePrototypeId>>;
}

#[async_trait]
impl AttributeValueExt for WorkspaceSnapshot {
    async fn attribute_value_view(
        &self,
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        let av_tree = self
            .working_copy()
            .await
            .attribute_value_tree(attribute_value_id)?;
        let mut sub_views = HashMap::new();
        let mut forward_queue = VecDeque::from_iter([av_tree.root()]);
        let mut work_stack = Vec::with_capacity(av_tree.count());

        // Build the stack of work, ensuring that all children appear after their parent in the list.
        while let Some(av_tree_entry) = forward_queue.pop_front() {
            work_stack.push(av_tree_entry);

            let children = av_tree.children_of(av_tree_entry.attribute_value_id);
            forward_queue.extend(children);
        }

        // Handle building the views in post-order traversal to ensure that all children have been
        // processed before their parent.
        while let Some(av_tree_entry) = work_stack.pop() {
            let av_node_weight = self
                .working_copy()
                .await
                .get_node_weight_by_id(av_tree_entry.attribute_value_id)?
                .get_attribute_value_node_weight()?;
            match av_node_weight.value() {
                None => {
                    // If the AttributeValue's value is None, and it does not have a component-specific
                    // AttributePrototype, look for a default value for the Prop.
                    // If there's no default value, and it's a complex type, build it from its children.
                    if self
                        .component_prototype_id(av_tree_entry.attribute_value_id)
                        .await?
                        .is_none()
                    {
                        if let Some(default_value) =
                            self.prop_default_value(ctx, av_tree_entry.prop_id).await?
                        {
                            sub_views.insert(av_tree_entry.attribute_value_id, default_value);
                        } else {
                            let value =
                                build_complex_value(self, &av_tree_entry, &av_tree, &mut sub_views)
                                    .await?;
                            if let Some(value) = value {
                                sub_views.insert(av_tree_entry.attribute_value_id, value);
                            }
                        }
                    }
                }
                Some(value_content_address) => {
                    let value = match av_tree_entry.prop_kind {
                        PropKind::Boolean
                        | PropKind::Float
                        | PropKind::Integer
                        | PropKind::Json
                        | PropKind::String => ctx
                            .layer_db()
                            .cas()
                            .try_read_as::<si_events::CasValue>(
                                &value_content_address.content_hash(),
                            )
                            .await?
                            .map(Into::into)
                            .ok_or_else(|| {
                                WorkspaceSnapshotError::MissingContentFromStore(
                                    av_tree_entry.attribute_value_id.into(),
                                )
                            })?,
                        PropKind::Array | PropKind::Map | PropKind::Object => {
                            build_complex_value_or_default(
                                self,
                                &av_tree_entry,
                                &av_tree,
                                &mut sub_views,
                            )
                            .await?
                        }
                    };
                    sub_views.insert(av_tree_entry.attribute_value_id, value);
                }
            }
        }

        Ok(sub_views.remove(&attribute_value_id))
    }

    async fn component_prototype_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributePrototypeId>> {
        self.working_copy()
            .await
            .component_prototype_id(attribute_value_id)
    }
}
