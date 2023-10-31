use content_store::{ContentHash, Store};
use serde_json::Value;
use ulid::Ulid;

use crate::change_set_pointer::ChangeSetPointer;
use crate::prop::{PropContent, PropContentV1, PropGraphNode};
use crate::property_editor::schema::WidgetKind;

use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::{WorkspaceSnapshotError, WorkspaceSnapshotResult};
use crate::{DalContext, Prop, PropId, PropKind, SchemaVariantId, Timestamp, WorkspaceSnapshot};

pub enum PropParent {
    OrderedProp(PropId),
    Prop(PropId),
    SchemaVariant(SchemaVariantId),
}

impl WorkspaceSnapshot {
    /// Create a new [`Prop`]. A corresponding [`AttributePrototype`] and [`AttributeValue`] will be
    /// created when the provided [`SchemaVariant`](crate::SchemaVariant) is
    /// [`finalized`](crate::SchemaVariant::finalize).
    pub async fn prop_create(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        name: impl AsRef<str>,
        kind: PropKind,
        widget_kind_and_options: Option<(WidgetKind, Option<Value>)>,
        prop_parent: PropParent,
        ordered: bool,
    ) -> WorkspaceSnapshotResult<PropGraphNode> {
        let timestamp = Timestamp::now();
        let name = name.as_ref();
        let (widget_kind, widget_options) = match widget_kind_and_options {
            Some((kind, options)) => (kind, options),
            None => (WidgetKind::from(kind), None),
        };

        let content = PropContentV1 {
            timestamp,
            name: name.to_string(),
            kind,
            widget_kind,
            widget_options,
            doc_link: None,
            hidden: false,
            refers_to_prop_id: None,
            diff_func_id: None,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&PropContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_prop(change_set, id, kind, name, hash)?;
        let node_index = if ordered {
            self.working_copy()?
                .add_ordered_node(change_set, node_weight)?
        } else {
            self.working_copy()?.add_node(node_weight)?
        };

        match prop_parent {
            PropParent::OrderedProp(ordered_prop_id) => {
                let parent_node_index = self
                    .working_copy()?
                    .get_node_index_by_id(ordered_prop_id.into())?;
                self.working_copy()?.add_ordered_edge(
                    change_set,
                    parent_node_index,
                    EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                    node_index,
                )?;
            }
            PropParent::Prop(prop_id) => {
                let parent_node_index =
                    self.working_copy()?.get_node_index_by_id(prop_id.into())?;
                self.working_copy()?.add_edge(
                    parent_node_index,
                    EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                    node_index,
                )?;
            }
            PropParent::SchemaVariant(schema_variant_id) => {
                let parent_node_index = self
                    .working_copy()?
                    .get_node_index_by_id(schema_variant_id.into())?;
                self.working_copy()?.add_edge(
                    parent_node_index,
                    EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                    node_index,
                )?;
            }
        };

        Ok(PropGraphNode::assemble(id, hash, content))
    }

    async fn prop_get_content(
        &mut self,
        ctx: &DalContext,
        prop_id: PropId,
    ) -> WorkspaceSnapshotResult<(ContentHash, PropContentV1)> {
        let id: Ulid = prop_id.into();
        let node_index = self.working_copy()?.get_node_index_by_id(id)?;
        let node_weight = self.working_copy()?.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: PropContent = ctx
            .content_store()
            .lock()
            .await
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let PropContent::V1(inner) = content;

        Ok((hash, inner))
    }

    pub async fn prop_modify_by_id<L>(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        id: PropId,
        lambda: L,
    ) -> WorkspaceSnapshotResult<PropGraphNode>
    where
        L: FnOnce(&mut Prop) -> WorkspaceSnapshotResult<()>,
    {
        let (_, inner) = self.prop_get_content(ctx, id).await?;

        let mut prop = Prop::assemble(id, &inner);
        lambda(&mut prop)?;
        let updated = PropContentV1::from(prop);

        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&PropContent::V1(updated.clone()))?;

        self.working_copy()?
            .update_content(change_set, id.into(), hash)?;

        Ok(PropGraphNode::assemble(id, hash, updated))
    }
}
