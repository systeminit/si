use std::collections::VecDeque;

use content_store::{ContentHash, Store};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use ulid::Ulid;

use crate::change_set_pointer::ChangeSetPointer;
use crate::func::intrinsics::IntrinsicFunc;
use crate::schema::variant::root_prop::RootProp;
use crate::schema::variant::{
    SchemaVariantContent, SchemaVariantContentV1, SchemaVariantGraphNode,
};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::graph::NodeIndex;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, PropNodeWeight};
use crate::workspace_snapshot::{WorkspaceSnapshotError, WorkspaceSnapshotResult};
use crate::{
    ActionKind, AttributePrototypeId, DalContext, FuncId, PropKind, Schema, SchemaId,
    SchemaVariant, SchemaVariantId, SocketArity, Timestamp, WorkspaceSnapshot,
};

pub mod root_prop;

impl WorkspaceSnapshot {
    pub async fn schema_variant_create(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        name: impl AsRef<str>,
        schema_id: SchemaId,
        ui_hidden: bool,
    ) -> WorkspaceSnapshotResult<(SchemaVariantGraphNode, RootProp)> {
        let name = name.as_ref();
        let timestamp = Timestamp::now();

        let content = SchemaVariantContentV1 {
            timestamp,
            name: name.to_string(),
            root_prop_id: None,
            // schema_variant_definition_id: None,
            link: None,
            ui_hidden,
            finalized_once: false,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&SchemaVariantContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::SchemaVariant(hash))?;
        let node_index = self.working_copy()?.add_node(node_weight)?;

        let schema_node_index = self
            .working_copy()?
            .get_node_index_by_id(schema_id.into())?;
        self.working_copy()?.add_edge(
            schema_node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            node_index,
        )?;

        let schema_variant_id: SchemaVariantId = id.into();

        let root_prop = self
            .schema_variant_create_root_prop_tree(ctx, change_set, schema_variant_id, schema_id)
            .await?;

        let func_id = self.func_find_intrinsic(IntrinsicFunc::Identity)?;

        self.internal_provider_create_explicit_with_socket(
            ctx,
            change_set,
            schema_variant_id,
            "Frame",
            func_id,
            SocketArity::Many,
            true,
        )
        .await?;
        self.external_provider_create_with_socket(
            ctx,
            change_set,
            schema_variant_id,
            "Frame",
            None,
            func_id,
            SocketArity::One,
            true,
        )
        .await?;

        Ok((
            SchemaVariantGraphNode::assemble(id, hash, content),
            root_prop,
        ))
    }

    async fn schema_variant_get_root_prop(
        &mut self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<PropNodeWeight> {
        let edge_targets: Vec<NodeIndex> = self
            .edges_directed(schema_variant_id.into(), Direction::Outgoing)?
            .map(|edge_ref| edge_ref.target())
            .collect();

        for index in edge_targets {
            let node_weight = self.get_node_weight(index)?;
            // TODO(nick): ensure that only one prop can be under a schema variant.
            if let NodeWeight::Prop(inner_weight) = node_weight {
                if inner_weight.name() == "root" {
                    return Ok(inner_weight.clone());
                }
            }
        }
        todo!("could not get root prop")
    }

    pub async fn schema_variant_create_default_prototypes(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<()> {
        let func_id = self.func_find_intrinsic(IntrinsicFunc::Unset)?;
        let root_prop = self.schema_variant_get_root_prop(schema_variant_id).await?;
        let mut work_queue: VecDeque<PropNodeWeight> = VecDeque::from(vec![root_prop]);

        while let Some(prop) = work_queue.pop_front() {
            // See an attribute prototype exists.
            let mut found_attribute_prototype_id: Option<AttributePrototypeId> = None;
            let targets = self.outgoing_targets_for_edge_weight_kind(
                prop.id(),
                EdgeWeightKindDiscriminants::Prototype,
            )?;
            for target in targets {
                let node_weight = self.get_node_weight(target)?;
                if let Some(discriminants) = node_weight.content_address_discriminants() {
                    if let ContentAddressDiscriminants::AttributePrototype = discriminants {
                        found_attribute_prototype_id = Some(node_weight.id().into());
                        break;
                    }
                }
            }

            // Create the attribute prototype and appropriate edges if they do not exist.
            if found_attribute_prototype_id.is_none() {
                // We did not find a prototype, so we must create one.
                let (_attribute_prototype, attribute_prototype_node_index) = self
                    .attribute_prototype_create(ctx, change_set, func_id)
                    .await?;

                // New edge Prop --Prototype--> AttributePrototype.
                let prop_node_index = self.get_node_index_by_id(prop.id())?;
                self.add_edge(
                    prop_node_index,
                    EdgeWeight::new(change_set, EdgeWeightKind::Prototype)?,
                    attribute_prototype_node_index,
                )?;
            }

            // Push all children onto the work queue.
            let targets = self.outgoing_targets_for_edge_weight_kind(
                prop.id(),
                EdgeWeightKindDiscriminants::Use,
            )?;
            for target in targets {
                let node_weight = self.get_node_weight(target)?;
                if let NodeWeight::Prop(child_prop) = node_weight {
                    work_queue.push_back(child_prop.to_owned())
                }
            }
        }

        Ok(())
    }

    pub async fn schema_variant_create_implicit_internal_providers(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<()> {
        let root_prop = self.schema_variant_get_root_prop(schema_variant_id).await?;
        let mut work_queue = VecDeque::new();
        work_queue.push_back(root_prop);

        while let Some(prop) = work_queue.pop_front() {
            self.internal_provider_create_implicit(ctx, change_set, &prop)
                .await?;

            // Only descend if we are an object.
            if prop.kind() == PropKind::Object {
                let targets = self.outgoing_targets_for_edge_weight_kind(
                    prop.id(),
                    EdgeWeightKindDiscriminants::Use,
                )?;
                for target in targets {
                    let node_weight = self.get_node_weight(target)?;
                    if let NodeWeight::Prop(child_prop) = node_weight {
                        work_queue.push_back(child_prop.to_owned());
                    }
                }
            }
        }

        Ok(())
    }

    pub fn action_prototype_create(
        &mut self,
        _ctx: &DalContext,
        change_set: &ChangeSetPointer,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
        _kind: ActionKind,
    ) -> WorkspaceSnapshotResult<()> {
        let schema_variant_index = self
            .working_copy()?
            .get_node_index_by_id(schema_variant_id.into())?;

        let func_index = self.working_copy()?.get_node_index_by_id(func_id.into())?;

        self.working_copy()?.add_edge(
            schema_variant_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            func_index,
        )?;

        Ok(())
    }

    async fn schema_variant_get_content(
        &mut self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<(ContentHash, SchemaVariantContentV1)> {
        let id: Ulid = schema_variant_id.into();
        let node_index = self.working_copy()?.get_node_index_by_id(id)?;
        let node_weight = self.working_copy()?.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: SchemaVariantContent = ctx
            .content_store()
            .lock()
            .await
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let inner = match content {
            SchemaVariantContent::V1(inner) => inner,
        };

        Ok((hash, inner))
    }

    pub async fn schema_variant_list(
        &mut self,
        ctx: &DalContext,
    ) -> WorkspaceSnapshotResult<Vec<SchemaVariant>> {
        /*
        let schema_category_index = self.get_category(CategoryNodeKind::Schema)?;
        let schema_indices = self.outgoing_targets_for_edge_weight_kind_by_index(
            schema_category_index,
            EdgeWeightKindDiscriminants::Use,
        )?;

        // TODO(nick,zack,jacob,wendy): start here!
        let mut unchecked_node_weights = Vec::new();
        for schema_index in schema_indices {
            unchecked_node_weights.push(self.get_node_weight(schema_index)?);
        }
        let mut schemas = Vec::new();
        for unchecked_node_weight in unchecked_node_weights {
            if let NodeWeight::Content(content_node_weight) = unchecked_node_weight {
                let (_, content) = self
                    .schema_get_content(ctx, content_node_weight.id().into())
                    .await?;
                schemas.push(Schema::assemble(content_node_weight.id().into(), &content));
            }
            }*/

        Ok(vec![])
    }

    /// This _idempotent_ function "finalizes" a [`SchemaVariant`].
    ///
    /// Once a [`SchemaVariant`] has had all of its [`Props`](crate::Prop) created, there are a few
    /// things that need to happen before it is usable:
    ///
    /// * Create the default [`AttributePrototypes`](crate::AttributePrototype)
    /// * Create the _internally consuming_ [`InternalProviders`](crate::InternalProvider)
    ///   corresponding to every [`Prop`](crate::Prop) in the [`SchemaVariant`] that is not a
    ///   descendant of an Array or a Map.
    ///
    /// This method **MUST** be called once all the [`Props`](Prop) have been created for the
    /// [`SchemaVariant`]. It can be called multiple times while [`Props`](Prop) are being created,
    /// but it must be called once after all [`Props`](Prop) have been created.
    pub async fn schema_variant_finalize(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<()> {
        self.schema_variant_create_default_prototypes(ctx, change_set, schema_variant_id)
            .await?;
        self.schema_variant_create_implicit_internal_providers(ctx, change_set, schema_variant_id)
            .await?;

        // TODO(nick,jacob,zack): if we are going to copy the existing system (which we likely will), we need to
        // set "/root/si/type" and "/root/si/protected".

        Ok(())
    }
}
