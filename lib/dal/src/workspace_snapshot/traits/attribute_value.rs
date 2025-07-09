use std::collections::{
    HashMap,
    VecDeque,
};

use async_trait::async_trait;
use si_id::{
    AttributePrototypeId,
    AttributeValueId,
};

use crate::{
    DalContext,
    PropKind,
    WorkspaceSnapshot,
    WorkspaceSnapshotError,
    attribute::value::{
        AttributeValueError,
        AttributeValueResult,
    },
    workspace_snapshot::graph::traits::attribute_value::AttributeValueExt as _,
};

#[async_trait]
pub trait AttributeValueExt {
    async fn attribute_value_view(
        &self,
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<serde_json::Value>>;

    async fn component_prototype_id(
        &self,
        id: AttributeValueId,
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
        let mut work_stack = Vec::with_capacity(av_tree.len());

        // Build the stack of work, ensuring that all children appear after their parent in the list.
        while let Some(av_tree_entry) = forward_queue.pop_front() {
            work_stack.push(av_tree_entry);

            let children = av_tree.children_of(av_tree_entry.attribute_value_id);
            forward_queue.reserve(children.len());
            forward_queue.extend(children);
        }

        // Handle building the views post-order traversal to ensure that all children have been
        // processed before their parent.
        while let Some(av_tree_entry) = work_stack.pop() {
            let av_node_weight = self
                .working_copy()
                .await
                .get_node_weight_by_id(av_tree_entry.attribute_value_id)?
                .get_attribute_value_node_weight()?;
            match av_node_weight.value() {
                None => {
                    sub_views.insert(av_tree_entry.attribute_value_id, serde_json::Value::Null);
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
                        PropKind::Array => {
                            let children = av_tree.children_of(av_tree_entry.attribute_value_id);
                            let mut entries = Vec::with_capacity(children.len());
                            for child_tree_entry in children {
                                entries.push(
                                    sub_views
                                        .remove(&child_tree_entry.attribute_value_id)
                                        .ok_or_else(|| {
                                            AttributeValueError::MissingForId(
                                                child_tree_entry.attribute_value_id,
                                            )
                                        })?,
                                );
                            }

                            serde_json::to_value(entries)?
                        }
                        PropKind::Map => {
                            let children = av_tree.children_of(av_tree_entry.attribute_value_id);
                            let mut entries = HashMap::with_capacity(children.len());
                            for child_tree_entry in children {
                                let key = self
                                    .working_copy()
                                    .await
                                    .key_for_attribute_value_id(
                                        child_tree_entry.attribute_value_id,
                                    )?
                                    .ok_or_else(|| {
                                        AttributeValueError::MissingKeyForMapEntry(
                                            child_tree_entry.attribute_value_id,
                                        )
                                    })?;
                                entries.insert(
                                    key,
                                    sub_views
                                        .remove(&child_tree_entry.attribute_value_id)
                                        .ok_or_else(|| {
                                            AttributeValueError::MissingForId(
                                                child_tree_entry.attribute_value_id,
                                            )
                                        })?,
                                );
                            }

                            serde_json::to_value(entries)?
                        }
                        PropKind::Object => {
                            let children = av_tree.children_of(av_tree_entry.attribute_value_id);
                            let mut entries = HashMap::with_capacity(children.len());
                            for child_tree_entry in children {
                                let prop_name = self
                                    .working_copy()
                                    .await
                                    .get_node_weight_by_id(child_tree_entry.prop_id)?
                                    .get_prop_node_weight()?
                                    .name()
                                    .to_owned();
                                entries.insert(
                                    prop_name,
                                    sub_views
                                        .remove(&child_tree_entry.attribute_value_id)
                                        .ok_or_else(|| {
                                            AttributeValueError::MissingForId(
                                                child_tree_entry.attribute_value_id,
                                            )
                                        })?,
                                );
                            }

                            serde_json::to_value(entries)?
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
        id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributePrototypeId>> {
        Ok(self.working_copy().await.component_prototype_id(id)?)
    }
}
