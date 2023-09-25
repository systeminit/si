use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, SiPkgSchemaVariant, Source};

use crate::SchemaSpecData;
use crate::{node::PkgNode, SchemaSpec};

#[derive(Clone, Debug)]
pub struct SiPkgSchemaData {
    pub name: String,
    pub category: String,
    pub category_name: Option<String>,
    pub ui_hidden: bool,
    pub default_schema_variant: Option<String>,
}

impl SiPkgSchemaData {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn category(&self) -> &str {
        self.category.as_str()
    }

    pub fn category_name(&self) -> Option<&str> {
        self.category_name.as_deref()
    }

    pub fn ui_hidden(&self) -> bool {
        self.ui_hidden
    }

    pub fn default_schema_variant(&self) -> Option<&str> {
        self.default_schema_variant.as_deref()
    }
}

#[derive(Clone, Debug)]
pub struct SiPkgSchema<'a> {
    name: String,
    data: Option<SiPkgSchemaData>,
    unique_id: Option<String>,
    deleted: bool,

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
            data: schema_node.data.map(|data| SiPkgSchemaData {
                name: data.name,
                category: data.category,
                category_name: data.category_name,
                ui_hidden: data.ui_hidden,
                default_schema_variant: data.default_schema_variant,
            }),
            unique_id: schema_node.unique_id,
            deleted: schema_node.deleted,
            hash: schema_hashed_node.hash(),
            source: Source::new(graph, node_idx),
        };

        Ok(schema)
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn data(&self) -> Option<&SiPkgSchemaData> {
        self.data.as_ref()
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

    pub async fn to_spec(&self) -> PkgResult<SchemaSpec> {
        let mut builder = SchemaSpec::builder();

        builder.name(self.name());
        if let Some(unique_id) = self.unique_id() {
            builder.unique_id(unique_id);
        }

        if let Some(data) = self.data() {
            let mut data_builder = SchemaSpecData::builder();
            data_builder.name(self.name());
            if let Some(category_name) = data.category_name() {
                data_builder.category_name(category_name);
            }
            if let Some(default_schema_variant) = data.default_schema_variant() {
                data_builder.default_schema_variant(default_schema_variant);
            }
            data_builder.ui_hidden(data.ui_hidden());
            data_builder.category(data.category());
            builder.data(data_builder.build()?);
        }

        for variant in self.variants()? {
            builder.variant(variant.to_spec().await?);
        }

        Ok(builder.build()?)
    }
}
