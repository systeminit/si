use async_trait::async_trait;
use petgraph::prelude::*;
use si_id::{
    AttributePrototypeId,
    PropId,
};

use crate::{
    DalContext,
    EdgeWeightKindDiscriminants,
    PropKind,
    WorkspaceSnapshot,
    WorkspaceSnapshotGraphVCurrent,
    prop::{
        PropError,
        PropResult,
    },
    slow_rt,
    workspace_snapshot::{
        graph::traits::prop::PropExt as _,
        node_weight::PropNodeWeight,
        traits::{
            attribute_prototype::AttributePrototypeExt as _,
            attribute_prototype_argument::AttributePrototypeArgumentExt,
            func::FuncExt as _,
        },
    },
};

#[async_trait]
pub trait PropExt {
    /// The default value for a [`Prop`][crate::prop::Prop], if there is a default value.
    async fn prop_default_value(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Option<serde_json::Value>>;

    /// The first [`AttributePrototypeId`] for the given [`PropId`]. It is possible for a
    /// prop to have multiple prototypes (for example with maps having ones for different
    /// keys in the map), but it is an error for the prop to not have any prototypes.
    async fn prop_prototype_id(&self, prop_id: PropId) -> PropResult<AttributePrototypeId>;

    /// Generate a TypeScript type for a prop tree.
    async fn ts_type(&self, prop_id: PropId) -> PropResult<String>;

    /// Build complete prop schema tree.
    async fn build_prop_schema_tree(
        &self,
        ctx: &DalContext,
        root_prop_id: PropId,
    ) -> PropResult<si_frontend_mv_types::prop_schema::PropSchemaV1>;
}

#[async_trait]
impl PropExt for WorkspaceSnapshot {
    async fn prop_default_value(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
    ) -> PropResult<Option<serde_json::Value>> {
        let prototype_id = self.prop_prototype_id(prop_id).await?;
        let func_id = self.attribute_prototype_func_id(prototype_id).await?;
        if self.func_is_dynamic(func_id).await? {
            return Ok(None);
        }

        match self
            .attribute_prototype_arguments(prototype_id)
            .await?
            .first()
        {
            Some(&apa_id) => self
                .attribute_prototype_argument_static_value(ctx, apa_id)
                .await
                .map_err(Into::into),
            None => Ok(None),
        }
    }

    async fn prop_prototype_id(&self, prop_id: PropId) -> PropResult<AttributePrototypeId> {
        self.outgoing_targets_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Prototype)
            .await?
            .first()
            .copied()
            .map(Into::into)
            .ok_or_else(|| PropError::MissingPrototypeForProp(prop_id))
    }

    async fn ts_type(&self, prop_id: PropId) -> PropResult<String> {
        let self_clone = self.clone();
        slow_rt::spawn(async move { ts_type(self_clone, prop_id).await })?.await?
    }

    async fn build_prop_schema_tree(
        &self,
        ctx: &DalContext,
        root_prop_id: PropId,
    ) -> PropResult<si_frontend_mv_types::prop_schema::PropSchemaV1> {
        use std::collections::{
            HashMap,
            VecDeque,
        };

        let tree_data = self
            .working_copy()
            .await
            .build_prop_schema_tree_data(root_prop_id)?
            .ok_or_else(|| PropError::PropNotFound(root_prop_id))?;

        let mut sub_schemas = HashMap::new();
        let mut forward_queue = VecDeque::from([root_prop_id]);
        let mut work_stack = Vec::with_capacity(tree_data.props.len());

        // Build work stack ensuring children appear after parents (BFS)
        while let Some(prop_id) = forward_queue.pop_front() {
            work_stack.push(prop_id);

            if let Some(child_ids) = tree_data.children.get(&prop_id) {
                forward_queue.extend(child_ids);
            }
        }

        // PProcess work stack in reverse (post-order) so children are built before parents
        while let Some(prop_id) = work_stack.pop() {
            let prop_data = tree_data
                .props
                .get(&prop_id)
                .ok_or_else(|| PropError::PropNotFound(prop_id))?;

            let prop_content = ctx
                .layer_db()
                .cas()
                .try_read_as::<crate::layer_db_types::PropContent>(&prop_data.content_hash)
                .await?
                .ok_or_else(|| PropError::PropNotFound(prop_id))?;

            let content_v2: crate::layer_db_types::PropContentV2 = prop_content.into();
            let validation_format = content_v2.validation_format.clone();
            let hidden = Some(content_v2.hidden);
            let doc_link = content_v2.doc_link.clone();
            let description = content_v2.documentation.clone();

            let default_value = self.prop_default_value(ctx, prop_id).await?;

            let children = if let Some(child_ids) = tree_data.children.get(&prop_id) {
                let mut child_schemas = Vec::with_capacity(child_ids.len());
                for &child_id in child_ids {
                    if let Some(child_schema) = sub_schemas.remove(&child_id) {
                        child_schemas.push(child_schema);
                    } else {
                        return Err(PropError::PropNotFound(child_id)); // Child not ready, shouldn't happen with post-order
                    }
                }
                if child_schemas.is_empty() {
                    None
                } else {
                    Some(child_schemas)
                }
            } else {
                None
            };

            let prop_type = prop_data.kind.as_ref();

            let schema = si_frontend_mv_types::prop_schema::PropSchemaV1 {
                prop_id,
                name: prop_data.name.clone(),
                prop_type: prop_type.to_string(),
                description,
                children,
                validation_format,
                default_value,
                hidden,
                doc_link,
            };

            sub_schemas.insert(prop_id, schema);
        }

        sub_schemas
            .remove(&root_prop_id)
            .ok_or_else(|| PropError::PropNotFound(root_prop_id))
    }
}

async fn ts_type(snap: WorkspaceSnapshot, prop_id: PropId) -> PropResult<String> {
    let graph = snap.working_copy().await;
    let index = graph.get_node_index_by_id(prop_id)?;
    let node = graph.get_node_weight(index)?.as_prop_node_weight()?;
    let mut result = String::new();
    append_ts_type(&graph, node, index, &mut result)?;
    Ok(result)
}

fn append_ts_type(
    graph: &WorkspaceSnapshotGraphVCurrent,
    node: &PropNodeWeight,
    index: NodeIndex,
    buf: &mut String,
) -> PropResult<()> {
    /// Check if the parent of the given node has the specified path.
    fn parent_has_path(
        graph: &WorkspaceSnapshotGraphVCurrent,
        index: NodeIndex,
        path: &[&str],
    ) -> PropResult<bool> {
        // Get the parent
        let parent_index = graph.get_edge_weight_kind_target_idx_opt(
            index,
            Incoming,
            EdgeWeightKindDiscriminants::Use,
        )?;

        // If the path is empty, we match iff there is no parent
        let Some((&name, parent_path)) = path.split_last() else {
            return Ok(parent_index.is_none());
        };

        // If the path is non-empty, but we have a parent, we don't match
        let Some(parent_index) = parent_index else {
            return Ok(false);
        };

        let node = graph.get_node_weight(parent_index)?.as_prop_node_weight()?;
        Ok(name == node.name() && parent_has_path(graph, parent_index, parent_path)?)
    }

    // Special cases
    if node.name() == "status" && parent_has_path(graph, index, &["root", "resource"])? {
        buf.push_str("'ok' | 'warning' | 'error' | undefined | null");
        return Ok(());
    }
    if node.name() == "payload" && parent_has_path(graph, index, &["root", "resource"])? {
        buf.push_str("any");
        return Ok(());
    }

    match node.kind() {
        PropKind::Array => {
            append_ts_element_type(graph, index, buf)?;
            buf.push_str("[]");
        }
        PropKind::Boolean => buf.push_str("boolean"),
        PropKind::Float | PropKind::Integer => buf.push_str("number"),
        PropKind::Json => buf.push_str("any"),
        PropKind::Map => {
            buf.push_str("Record<string, ");
            append_ts_element_type(graph, index, buf)?;
            buf.push('>');
        }
        PropKind::Object => {
            buf.push_str("{\n");
            for child_index in graph.ordered_children_for_node(index)?.unwrap_or(vec![]) {
                let child_node = graph.get_node_weight(child_index)?.as_prop_node_weight()?;
                buf.push_str(&serde_json::to_string(child_node.name())?);
                buf.push_str("?: ");
                append_ts_type(graph, child_node, child_index, buf)?;
                buf.push_str(" | null;\n");
            }
            buf.push('}');
        }
        PropKind::String => buf.push_str("string"),
    };
    Ok(())
}

/// Generate a TypeScript type for the element type of an array or map.
fn append_ts_element_type(
    graph: &WorkspaceSnapshotGraphVCurrent,
    parent_index: NodeIndex,
    buf: &mut String,
) -> PropResult<()> {
    let element_prop_index = graph.get_edge_weight_kind_target_idx(
        parent_index,
        Outgoing,
        EdgeWeightKindDiscriminants::Use,
    )?;
    let element_prop_node = graph
        .get_node_weight(element_prop_index)?
        .as_prop_node_weight()?;
    append_ts_type(graph, element_prop_node, element_prop_index, buf)
}
