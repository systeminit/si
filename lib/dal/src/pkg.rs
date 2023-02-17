use std::path::Path;

use object_tree::{FsError, HashedNode, NameStr, ObjectTree, TreeFileSystemReader};
use petgraph::prelude::*;
use si_pkg::schema::node::PropNode;
use thiserror::Error;

use crate::{
    component::ComponentKind,
    schema::{
        variant::definition::{hex_color_to_i64, SchemaVariantDefinitionError},
        SchemaUiMenu,
    },
    DalContext, Prop, PropError, PropId, PropKind, Schema, SchemaError, SchemaVariant,
    SchemaVariantError, StandardModel, StandardModelError,
};

#[derive(Debug, Error)]
pub enum PkgError {
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    SchemaVariantDefinition(#[from] SchemaVariantDefinitionError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
}

pub type PkgResult<T> = Result<T, PkgError>;

pub async fn import_schema(ctx: &mut DalContext, name: &str, tar_path: &Path) -> PkgResult<()> {
    #[derive(Debug)]
    struct StackEntry {
        hashed_node: HashedNode<PropNode>,
        node_idx: NodeIndex,
        parent_prop_id: PropId,
    }

    let tree: ObjectTree<PropNode> = TreeFileSystemReader::tar(tar_path).await?.read().await?;

    let (graph, root_idx) = tree.as_petgraph();

    let mut schema = Schema::new(ctx, name, &ComponentKind::Standard).await?;

    let ui_menu = SchemaUiMenu::new(ctx, name, "Packageland").await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema
        .set_default_schema_variant_id(ctx, Some(schema_variant.id()))
        .await?;
    schema_variant
        .set_color(ctx, Some(hex_color_to_i64("ff0000")?))
        .await?;
    let domain_prop_id = root_prop.domain_prop_id;

    let mut stack = Vec::new();
    for child_idx in graph.neighbors_directed(root_idx, Outgoing) {
        stack.push(StackEntry {
            hashed_node: graph[child_idx].clone(),
            node_idx: child_idx,
            parent_prop_id: domain_prop_id,
        });
    }

    while let Some(entry) = stack.pop() {
        let prop = Prop::new(
            ctx,
            entry.hashed_node.name(),
            match entry.hashed_node.inner() {
                PropNode::String { .. } => PropKind::String,
                PropNode::Integer { .. } => PropKind::Integer,
                PropNode::Boolean { .. } => PropKind::Boolean,
                PropNode::Map { .. } => PropKind::Map,
                PropNode::Array { .. } => PropKind::Array,
                PropNode::Object { .. } => PropKind::Object,
            },
            None,
        )
        .await?;
        prop.set_parent_prop(ctx, entry.parent_prop_id).await?;

        for child_idx in graph.neighbors_directed(entry.node_idx, Outgoing) {
            let child_node = graph[child_idx].clone();

            stack.push(StackEntry {
                hashed_node: child_node,
                node_idx: child_idx,
                parent_prop_id: *prop.id(),
            });
        }
    }

    schema_variant.finalize(ctx, None).await?;

    Ok(())
}
