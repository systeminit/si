use async_trait::async_trait;
use si_id::{
    AttributePrototypeId,
    PropId,
};

use crate::{
    DalContext,
    EdgeWeightKindDiscriminants,
    prop::{
        PropError,
        PropResult,
    },
    workspace_snapshot::{
        graph::traits::prop::PropExt as _,
        split_snapshot::SplitSnapshot,
        traits::{
            attribute_prototype::AttributePrototypeExt as _,
            attribute_prototype_argument::AttributePrototypeArgumentExt as _,
            func::FuncExt as _,
            prop::PropExt,
        },
    },
};

#[async_trait]
impl PropExt for SplitSnapshot {
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

    async fn ts_type(&self, _prop_id: PropId) -> PropResult<String> {
        Ok("any".to_string())
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

        // Process work stack in reverse (post-order) so children are built before parents
        while let Some(prop_id) = work_stack.pop() {
            let prop_data = tree_data
                .props
                .get(&prop_id)
                .ok_or_else(|| PropError::PropNotFound(prop_id))?;

            // Fetch prop content for fields
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
