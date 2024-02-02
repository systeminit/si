use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgAttributeValue, SiPkgError, SiPkgPosition, Source};

use crate::{
    node::{ComponentChildNode, PkgNode},
    AttributeValueSpec, ComponentSpec, ComponentSpecVariant, PositionSpec,
};

#[derive(Clone, Debug)]
pub struct SiPkgComponent<'a> {
    name: String,
    variant: ComponentSpecVariant,
    needs_destroy: bool,
    deletion_user_pk: Option<String>,
    unique_id: String,
    deleted: bool,

    hash: Hash,
    source: Source<'a>,
}

macro_rules! impl_component_children_from_graph {
    ($fn_name:ident, ComponentChildNode::$child_node:ident, $pkg_type:ident) => {
        pub fn $fn_name(&self) -> PkgResult<Vec<$pkg_type>> {
            let mut entries = vec![];
            if let Some(child_idxs) = self
                .source
                .graph
                .neighbors_directed(self.source.node_idx, Outgoing)
                .find(|node_idx| {
                    matches!(
                        &self.source.graph[*node_idx].inner(),
                        PkgNode::ComponentChild(ComponentChildNode::$child_node)
                    )
                })
            {
                let child_node_idxs: Vec<_> = self
                    .source
                    .graph
                    .neighbors_directed(child_idxs, Outgoing)
                    .collect();

                for child_idx in child_node_idxs {
                    entries.push($pkg_type::from_graph(self.source.graph, child_idx)?);
                }
            }

            Ok(entries)
        }
    };
}

impl<'a> SiPkgComponent<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::Component(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::COMPONENT_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            name: node.name,
            variant: node.variant,
            needs_destroy: node.needs_destroy,
            deletion_user_pk: node.deletion_user_pk,
            deleted: node.deleted,
            unique_id: node.unique_id,

            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    impl_component_children_from_graph!(
        attributes,
        ComponentChildNode::Attributes,
        SiPkgAttributeValue
    );

    impl_component_children_from_graph!(
        input_sockets,
        ComponentChildNode::InputSockets,
        SiPkgAttributeValue
    );

    impl_component_children_from_graph!(
        output_sockets,
        ComponentChildNode::OutputSockets,
        SiPkgAttributeValue
    );

    impl_component_children_from_graph!(position, ComponentChildNode::Position, SiPkgPosition);

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn variant(&self) -> &ComponentSpecVariant {
        &self.variant
    }

    pub fn needs_destroy(&self) -> bool {
        self.needs_destroy
    }

    pub fn deletion_user_pk(&self) -> Option<&str> {
        self.deletion_user_pk.as_deref()
    }

    pub fn unique_id(&self) -> &str {
        self.unique_id.as_str()
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> TryFrom<SiPkgComponent<'a>> for ComponentSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgComponent<'a>) -> Result<Self, Self::Error> {
        let mut builder = ComponentSpec::builder();

        let position = value
            .position()?
            .pop()
            .ok_or(SiPkgError::ComponentMissingPosition(value.name().into()))?;

        builder
            .name(value.name())
            .position(PositionSpec::try_from(position)?)
            .variant(value.variant().to_owned())
            .needs_destroy(value.needs_destroy())
            .deletion_user_pk(value.deletion_user_pk().map(ToString::to_string))
            .unique_id(value.unique_id())
            .deleted(value.deleted());

        for attribute in value.attributes()? {
            builder.attribute(AttributeValueSpec::try_from(attribute)?);
        }
        for input_socket in value.input_sockets()? {
            builder.input_socket(AttributeValueSpec::try_from(input_socket)?);
        }
        for output_socket in value.output_sockets()? {
            builder.output_socket(AttributeValueSpec::try_from(output_socket)?);
        }

        Ok(builder.build()?)
    }
}
