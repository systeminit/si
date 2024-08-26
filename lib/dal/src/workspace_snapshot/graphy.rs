use super::{
    content_address::{ContentAddress, ContentAddressDiscriminants},
    graph::WorkspaceSnapshotGraphError,
    node_weight::{category_node_weight::CategoryNodeKind, ContentNodeWeight, NodeWeight},
    WorkspaceSnapshotError,
};
use crate::{DalContext, NodeWeightDiscriminants};
use petgraph::stable_graph::NodeIndex;
use si_events::ulid::Ulid;
use thiserror::Error;

//
// Graphy primitives
//
mod context;
mod iter;
mod node_ref;

pub use context::*;
pub use iter::*;
pub use node_ref::*;

//
// SI node types
//
mod action;
mod action_prototype;
mod attribute_prototype;
mod attribute_prototype_argument;
mod attribute_value;
mod category;
mod component;
mod dependent_value_root;
mod func;
mod func_argument;
mod module;
mod ordering;
mod prop;
mod root;
mod schema;
mod schema_variant;
mod secret;
mod socket;
mod static_argument_value;

pub use action::*;
pub use action_prototype::*;
pub use attribute_prototype::*;
pub use attribute_prototype_argument::*;
pub use attribute_value::*;
pub use category::*;
pub use component::*;
pub use dependent_value_root::*;
pub use func::*;
pub use func_argument::*;
pub use module::*;
pub use ordering::*;
pub use prop::*;
pub use root::*;
pub use schema::*;
pub use schema_variant::*;
pub use secret::*;
pub use socket::*;
pub use static_argument_value::*;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum GraphyError {
    #[error("category node not found for kind: {0}")]
    CategoryNotFound(CategoryNodeKind),
    #[error("node has children but no ordering: {0:?}")]
    MissingOrdering(NodeIndex),
    #[error("multiple nodes when no more than one was expected")]
    MultipleNodes,
    #[error("no argument targets found for link")]
    NoArgumentTargets,
    #[error("node id not found: {0:?}")]
    NodeIdNotFound(NodeIndex),
    #[error("node not found: {0:?}")]
    NodeNotFound(NodeIndex),
    #[error("no nodes found when at least one was expected")]
    NoNodes,
    #[error("argument targets found for non-link argument")]
    UnexpectedArgumentTargets,
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
pub trait GraphyNode<'a>:
    Copy + Clone + Into<GraphyNodeRef<'a>> + AsRef<GraphyNodeRef<'a>> + Sized + 'a
{
    ///
    /// Id type for this node (e.g. ComponentId, or Ulid if no specific id type).
    ///
    type Id: From<Ulid> + Into<Ulid>;

    ///
    /// NodeWeight type for this node.
    ///
    type Weight;

    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self;
    fn try_as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> GraphyResult<Self> {
        let result = Self::as_node(node);
        result.weight()?;
        Ok(result)
    }
    fn weight_as(node: &NodeWeight) -> GraphyResult<&Self::Weight>;

    fn id(self) -> GraphyResult<Self::Id> {
        self.into().id().map(Into::into)
    }
    fn weight(self) -> GraphyResult<&'a Self::Weight> {
        Self::weight_as(self.as_ref().node_weight()?)
    }
}

pub trait GraphyContentNode<'a>: GraphyNode<'a, Weight = ContentNodeWeight> {
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

pub trait GraphyCategoryElement<'a>: GraphyNode<'a> {
    fn category_kind() -> CategoryNodeKind;
}

pub async fn do_it(ctx: &DalContext) -> GraphyResult<()> {
    let ctx = GraphyContext::new(ctx).await?;
    for component in ctx.root().category::<Component>()? {
        for child_prop in component.schema_variant()?.root_prop()?.children()? {
            println!("Prop: {}", child_prop?.id()?);
        }
    }
    Ok(())
}

macro_rules! impl_graphy_node {
    () => {};
    (impl From<$node:ident> for $($inner:ident),*; $($t:tt)*) => {
        $(
            impl<'a> From<$node<'a>> for $inner<'a> {
                fn from(from: $node<'a>) -> Self {
                    Self::from(from.0)
                }
            }
        )*
        impl_graphy_node! { $($t)* }
    };
    (impl TryFrom<$($inner:ident),*> for $node:ident; $($t:tt)*) => {
        $(
            impl<'a> TryFrom<$inner<'a>> for $node<'a> {
                type Error = crate::workspace_snapshot::graphy::GraphyError;
                fn try_from(from: $inner<'a>) -> Result<Self, Self::Error> {
                    let node = AsRef::<GraphyNodeRef<'a>>::as_ref(&from);
                    Self::weight_as(node.node_weight()?)?;
                    Ok(Self(from))
                }
            }
        )*
        impl_graphy_node! { $($t)* }
    };
    (impl TryFrom<$($inner_inner:ident),*> for $node:ident through $inner:ident; $($t:tt)*) => {
        $(
            impl<'a> TryFrom<$inner_inner<'a>> for $node<'a> {
                type Error = crate::workspace_snapshot::graphy::GraphyError;
                fn try_from(from: $inner_inner<'a>) -> Result<Self, Self::Error> {
                    let inner = $inner::try_from(from)?;
                    Self::try_from(inner)
                }
            }
        )*
        impl_graphy_node! { $($t)* }
    };
    (impl AsRef<$($inner:ident),*> for $node:ident; $($t:tt)*) => {
        $(
            impl<'a> AsRef<$inner<'a>> for $node<'a> {
                fn as_ref(&self) -> &$inner<'a> {
                    self.0.as_ref()
                }
            }
        )*
        impl_graphy_node! { $($t)* }
    };
}

pub(self) use impl_graphy_node;

macro_rules! impl_inherited_graphy_node {
    (impl * for $node:ident { $super:ident }) => {
       impl_inherited_graphy_node! { impl GraphyNode+From+AsRef for $node { $super } }
    };
    (impl $interface1:ident$(+$interface:ident)* for $node:ident { $super:ident }) => {
        impl_inherited_graphy_node! { impl! $interface1 for $node { $super } }
        $(impl_inherited_graphy_node! { impl! $interface for $node { $super } })*
    };
    (impl! GraphyNode for $node:ident { $super:ident }) => {
        impl<'a> GraphyNode<'a> for $node<'a> {
            type Id = <$super<'a> as GraphyNode<'a>>::Id;
            type Weight = <$super<'a> as GraphyNode<'a>>::Weight;
            fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
                Self($super::as_node(node))
            }
            fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
                $super::weight_as(weight)
            }
        }
    };
    (impl! From for $node:ident { $super:ident }) => {
        impl<'a> From<$node<'a>> for GraphyNodeRef<'a> {
            fn from(node: $node<'a>) -> Self {
                node.0.into()
            }
        }
    };
    (impl! AsRef for $node:ident { $super:ident }) => {
        impl<'a> AsRef<GraphyNodeRef<'a>> for $node<'a> {
            fn as_ref(&self) -> &GraphyNodeRef<'a> {
                self.0.as_ref()
            }
        }
    };
}

pub(self) use impl_inherited_graphy_node;

pub trait GraphyNodeId: Copy+Clone+Into<Ulid> {
    type Node<'a>: GraphyNode<'a>;
}
