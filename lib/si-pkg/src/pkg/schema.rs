use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, SiPkgSchemaVariant, Source};

use crate::{node::PkgNode, SchemaSpec, SchemaVariantSpec};

#[derive(Clone, Debug)]
pub struct SiPkgSchema<'a> {
    name: String,
    category: String,
    category_name: Option<String>,

    hash: Hash,

    source: Source<'a>,
}

impl<'a> SiPkgSchema<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let schema_hashed_node = &graph[node_idx];
        let schema_node = match schema_hashed_node.inner() {
            PkgNode::Schema(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::SCHEMA_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let schema = Self {
            name: schema_node.name,
            category: schema_node.category,
            category_name: schema_node.category_name,
            hash: schema_hashed_node.hash(),
            source: Source::new(graph, node_idx),
        };

        Ok(schema)
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn category(&self) -> &str {
        self.category.as_ref()
    }

    pub fn category_name(&self) -> Option<&str> {
        self.category_name.as_deref()
    }

    pub fn variants(&self) -> PkgResult<Vec<SiPkgSchemaVariant<'a>>> {
        let mut variants = vec![];
        for schema_variant_idx in self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
        {
            variants.push(SiPkgSchemaVariant::from_graph(
                self.source.graph,
                schema_variant_idx,
            )?);
        }

        Ok(variants)
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }
}

impl<'a> TryFrom<SiPkgSchema<'a>> for SchemaSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgSchema<'a>) -> Result<Self, Self::Error> {
        let mut builder = SchemaSpec::builder();

        builder.name(value.name()).category(value.category());
        if let Some(category_name) = value.category_name() {
            builder.category_name(category_name);
        }

        for variant in value.variants()? {
            builder.variant(SchemaVariantSpec::try_from(variant)?);
        }

        Ok(builder.build()?)
    }
}
