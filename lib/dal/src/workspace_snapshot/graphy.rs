use si_events::ulid::Ulid;
use thiserror::Error;
use petgraph::prelude::*;
use crate::{DalContext, EdgeWeightKindDiscriminants, NodeWeightDiscriminants};
use super::{content_address::{ContentAddress, ContentAddressDiscriminants}, graph::WorkspaceSnapshotGraphError, node_weight::{category_node_weight::CategoryNodeKind, ContentNodeWeight, NodeWeight}, WorkspaceSnapshotError};

mod action;
mod attribute_prototype;
mod category;
mod component;
mod context;
mod dependent_value_root;
mod func;
mod module;
mod node;
mod ordering;
mod root;
mod schema;
mod secret;

pub use action::*;
pub use attribute_prototype::*;
pub use category::*;
pub use component::*;
pub use context::*;
pub use dependent_value_root::*;
pub use func::*;
pub use module::*;
pub use node::*;
pub use ordering::*;
pub use root::*;
pub use schema::*;
pub use secret::*;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum GraphyError {
    #[error("category node not found for kind: {0}")]
    CategoryNotFound(CategoryNodeKind),
    #[error("node has children but no ordering: {0:?}")]
    MissingOrdering(NodeIndex),
    #[error("multiple matching nodes: {0:?} {1:?}")]
    MultipleMatchingNodes(NodeIndex, EdgeWeightKindDiscriminants),
    #[error("node id not found: {0:?}")]
    NodeIdNotFound(NodeIndex),
    #[error("node not found: {0:?}")]
    NodeNotFound(NodeIndex),
    #[error("no matching node: {0:?} {1:?}")]
    NoMatchingNode(NodeIndex, EdgeWeightKindDiscriminants),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("workspace snapshot graph error: {0}")]
    WorkspaceSnapshotGraph(#[from] WorkspaceSnapshotGraphError),
    #[error("wrong category kind: {0}")]
    WrongCategory(CategoryNodeKind),
    #[error("wrong content type: {0}")]
    WrongContentType(ContentAddressDiscriminants),
    #[error("wrong node type: {0}")]
    WrongNodeType(NodeWeightDiscriminants),
}

pub type GraphyResult<T> = Result<T, GraphyError>;

///
/// Graphy node common implementation methods.
/// 
pub trait GraphyNodeType<'a>: Copy+Clone+Into<GraphyNode<'a>>+AsRef<GraphyNode<'a>> {
    ///
    /// Id type for this node (e.g. ComponentId, or Ulid if no specific id type).
    /// 
    type Id: From<Ulid>+Into<Ulid>;

    ///
    /// NodeWeight type for this node.
    /// 
    type Weight;

    ///
    /// Type of node weight for this node.
    /// 
    fn node_kind() -> NodeWeightDiscriminants;

    ///
    /// Construct this node type from a graph node. *Assumes* the node is the correct kind:
    /// user will find out later if that's not the case when they try to get weights or do
    /// other things with it.
    /// 
    fn construct(node: GraphyNode<'a>) -> Self;
    fn weight_as(node: &NodeWeight) -> GraphyResult<&Self::Weight>;

    fn id(self) -> GraphyResult<Self::Id> { self.into().id().map(Into::into) }
    fn weight(self) -> GraphyResult<&'a Self::Weight> {
        Self::weight_as(self.as_ref().node_weight()?)
    }
}

pub trait GraphyContentNodeType<'a>: GraphyNodeType<'a, Weight = ContentNodeWeight> {
    fn content_kind() -> ContentAddressDiscriminants;
    fn content_address(self) -> GraphyResult<ContentAddress> {
        self.weight().map(|weight| weight.content_address())
    }
    fn content_weight_as(weight: &NodeWeight) -> GraphyResult<&ContentNodeWeight> {
        match weight {
            NodeWeight::Content(weight) => {
                let address = weight.content_address();
                if Self::content_kind() == address.into() {
                    Ok(weight)
                } else {
                    Err(GraphyError::WrongContentType(address.into()))
                }
            }
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

pub trait GraphyCategoryNodeType<'a>: GraphyNodeType<'a> {
    fn category_kind() -> CategoryNodeKind;
}


pub async fn do_it(ctx: &DalContext) -> GraphyResult<()>{
    let ctx = GraphyContext::new(ctx).await?;
    for component in ctx.root().components()? {
        let id = component.id()?;
        let weight = component.weight()?;
        println!("Component: {} {:?}", id, weight);
    }
    Ok(())
}