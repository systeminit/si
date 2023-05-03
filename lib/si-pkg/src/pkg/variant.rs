use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;
use std::collections::{hash_map::Entry, HashMap, VecDeque};
use std::future::Future;
use tokio::sync::Mutex;

use url::Url;

use super::{
    PkgResult, SiPkgCommandFunc, SiPkgError, SiPkgFuncDescription, SiPkgLeafFunction, SiPkgProp,
    SiPkgSocket, SiPkgWorkflow, Source,
};

use crate::{
    node::{PkgNode, PropChildNode, SchemaVariantChildNode},
    AttrFuncInputSpec, PropSpec, PropSpecBuilder, PropSpecKind, SchemaVariantSpec,
    SchemaVariantSpecComponentType,
};

#[derive(Clone, Debug)]
pub struct SiPkgSchemaVariant<'a> {
    name: String,
    link: Option<Url>,
    color: Option<String>,
    component_type: SchemaVariantSpecComponentType,

    hash: Hash,

    source: Source<'a>,
}

macro_rules! impl_variant_children_from_graph {
    ($fn_name:ident, SchemaVariantChildNode::$child_node:ident, $pkg_type:ident) => {
        pub fn $fn_name(&self) -> PkgResult<Vec<$pkg_type>> {
            let mut entries = vec![];
            if let Some(child_idxs) = self
                .source
                .graph
                .neighbors_directed(self.source.node_idx, Outgoing)
                .find(|node_idx| {
                    matches!(
                        &self.source.graph[*node_idx].inner(),
                        PkgNode::SchemaVariantChild(SchemaVariantChildNode::$child_node)
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

impl<'a> SiPkgSchemaVariant<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let schema_variant_hashed_node = &graph[node_idx];
        let schema_variant_node = match schema_variant_hashed_node.inner() {
            PkgNode::SchemaVariant(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::SCHEMA_VARIANT_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let schema_variant = Self {
            name: schema_variant_node.name,
            link: schema_variant_node.link,
            color: schema_variant_node.color,
            component_type: schema_variant_node.component_type,
            hash: schema_variant_hashed_node.hash(),
            source: Source::new(graph, node_idx),
        };

        Ok(schema_variant)
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn link(&self) -> Option<&Url> {
        self.link.as_ref()
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn component_type(&self) -> SchemaVariantSpecComponentType {
        self.component_type
    }

    impl_variant_children_from_graph!(sockets, SchemaVariantChildNode::Sockets, SiPkgSocket);
    impl_variant_children_from_graph!(
        func_descriptions,
        SchemaVariantChildNode::FuncDescriptions,
        SiPkgFuncDescription
    );
    impl_variant_children_from_graph!(workflows, SchemaVariantChildNode::Workflows, SiPkgWorkflow);
    impl_variant_children_from_graph!(
        leaf_functions,
        SchemaVariantChildNode::LeafFunctions,
        SiPkgLeafFunction
    );
    impl_variant_children_from_graph!(
        command_funcs,
        SchemaVariantChildNode::CommandFuncs,
        SiPkgCommandFunc
    );

    fn prop_stack_from_source<I>(
        source: Source<'a>,
        node_idx: NodeIndex,
        parent_id: Option<I>,
    ) -> PkgResult<Vec<(SiPkgProp, Option<I>)>>
    where
        I: ToOwned + Clone,
    {
        Ok(
            match source
                .graph
                .neighbors_directed(node_idx, Outgoing)
                .find(|node_idx| {
                    matches!(
                        &source.graph[*node_idx].inner(),
                        PkgNode::PropChild(PropChildNode::Props)
                    )
                }) {
                Some(prop_child_idxs) => {
                    let child_node_idxs: Vec<_> = source
                        .graph
                        .neighbors_directed(prop_child_idxs, Outgoing)
                        .collect();

                    let mut entries = vec![];
                    for child_idx in child_node_idxs {
                        entries.push((
                            SiPkgProp::from_graph(source.graph, child_idx)?,
                            parent_id.to_owned(),
                        ));
                    }

                    entries
                }
                None => vec![],
            },
        )
    }

    pub async fn visit_prop_tree<F, Fut, I, C, E>(
        &'a self,
        process_prop_fn: F,
        parent_id: Option<I>,
        context: &'a C,
    ) -> Result<(), E>
    where
        F: Fn(SiPkgProp<'a>, Option<I>, &'a C) -> Fut,
        Fut: Future<Output = Result<Option<I>, E>>,
        E: std::convert::From<SiPkgError>,
        I: ToOwned + Clone,
    {
        let domain_idxs = self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
            .find(|node_idx| {
                matches!(
                    &self.source.graph[*node_idx].inner(),
                    PkgNode::SchemaVariantChild(SchemaVariantChildNode::Domain)
                )
            })
            .ok_or(SiPkgError::CategoryNotFound(
                SchemaVariantChildNode::Domain.kind_str(),
            ))?;

        let mut child_node_idxs: Vec<_> = self
            .source
            .graph
            .neighbors_directed(domain_idxs, Outgoing)
            .collect();
        let domain_node_idx = match child_node_idxs.pop() {
            Some(idx) => idx,
            None => Err(SiPkgError::DomainPropNotFound(self.hash()))?,
        };
        if !child_node_idxs.is_empty() {
            Err(SiPkgError::DomainPropMultipleFound(self.hash()))?;
        }

        // Skip processing the domain prop as a `dal::SchemaVariant` already guarantees such a prop
        // has already been created. Rather, we will push all immediate children of the domain prop
        // to be ready for processing.
        let mut stack = Self::prop_stack_from_source(
            self.source.clone(),
            domain_node_idx,
            parent_id.to_owned(),
        )?;

        while let Some((prop, parent_id)) = stack.pop() {
            let node_idx = prop.source().node_idx;
            let new_id = process_prop_fn(prop, parent_id.to_owned(), context).await?;

            stack.extend(Self::prop_stack_from_source(
                self.source.clone(),
                node_idx,
                new_id.to_owned(),
            )?);
        }

        Ok(())
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub async fn to_spec(&self) -> PkgResult<SchemaVariantSpec> {
        let mut builder = SchemaVariantSpec::builder();

        builder
            .name(self.name())
            .component_type(self.component_type);

        if let Some(link) = self.link() {
            builder.link(link.to_owned());
        }

        if let Some(color) = self.color() {
            builder.color(color);
        }

        let context = PropSpecVisitContext {
            prop_stack: Mutex::new(VecDeque::new()),
            prop_parents: Mutex::new(HashMap::new()),
        };

        self.visit_prop_tree(create_prop_stack, None, &context)
            .await?;

        let prop_stack = context.prop_stack.into_inner();
        let prop_parents = context.prop_parents.into_inner();
        let mut prop_children: HashMap<String, Vec<PropSpec>> = HashMap::new();
        for (path, mut prop) in prop_stack {
            if let Some(children) = prop_children.get(&path) {
                match prop
                    .get_kind()
                    .ok_or(SiPkgError::prop_tree_invalid("Prop missing a kind"))?
                {
                    PropSpecKind::Array | PropSpecKind::Map => {
                        if children.len() > 1 {
                            return Err(SiPkgError::prop_tree_invalid(
                                "Array or map has more than one direct child",
                            ));
                        }
                        let type_prop = children.get(0).ok_or(SiPkgError::prop_tree_invalid(
                            "Array or map prop missing type prop",
                        ))?;
                        prop.type_prop(type_prop.clone());
                    }
                    PropSpecKind::Object => {
                        prop.entries(children.to_owned());
                    }
                    _ => {
                        return Err(SiPkgError::prop_tree_invalid(
                            "Leaf prop (String, Number, Boolean) cannot have children",
                        ))
                    }
                }
            }

            let spec = prop.build()?;

            match prop_parents.get(&path) {
                Some(parent_path) => match prop_children.entry(parent_path.clone()) {
                    Entry::Occupied(mut occupied) => {
                        occupied.get_mut().push(spec);
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(vec![spec]);
                    }
                },
                None => {
                    builder.prop(spec);
                }
            }
        }

        for command_func in self.command_funcs()? {
            builder.command_func(command_func.try_into()?);
        }

        for func_description in self.func_descriptions()? {
            builder.func_description(func_description.try_into()?);
        }

        for socket in self.sockets()? {
            builder.socket(socket.try_into()?);
        }

        for workflow in self.workflows()? {
            builder.workflow(workflow.try_into()?);
        }

        Ok(builder.build()?)
    }
}

#[derive(Debug)]
// These have to be some kind of tokio type with interior mutability because we need the Send trait
// for axum to be happy (it rejects using RefCell)
struct PropSpecVisitContext {
    // All the props in depth-first order
    prop_stack: Mutex<VecDeque<(String, PropSpecBuilder)>>,
    // A map from the prop path to prop's parent path
    prop_parents: Mutex<HashMap<String, String>>,
}

const PROP_PATH_SEPARATOR: &str = "\x0B";

async fn create_prop_stack(
    spec: SiPkgProp<'_>,
    parent_path: Option<String>,
    ctx: &PropSpecVisitContext,
) -> PkgResult<Option<String>> {
    let path = match &parent_path {
        Some(parent_path) => format!("{}{}{}", parent_path, PROP_PATH_SEPARATOR, spec.name()),
        None => spec.name().to_owned(),
    };

    let mut builder = PropSpec::builder();

    match &spec {
        SiPkgProp::String { default_value, .. } => {
            builder.kind(PropSpecKind::String);
            if let Some(dv) = default_value {
                builder.default_value(serde_json::to_value(dv)?);
            }
        }
        SiPkgProp::Boolean { default_value, .. } => {
            builder.kind(PropSpecKind::Boolean);
            if let Some(dv) = default_value {
                builder.default_value(serde_json::to_value(dv)?);
            }
        }
        SiPkgProp::Number { default_value, .. } => {
            builder.kind(PropSpecKind::Number);
            if let Some(dv) = default_value {
                builder.default_value(serde_json::to_value(dv)?);
            }
        }
        SiPkgProp::Object { .. } => {
            builder.kind(PropSpecKind::Object);
        }
        SiPkgProp::Array { .. } => {
            builder.kind(PropSpecKind::Array);
        }
        SiPkgProp::Map { .. } => {
            builder.kind(PropSpecKind::Map);
        }
    }

    match &spec {
        SiPkgProp::String {
            name,
            func_unique_id,
            widget_kind,
            widget_options,
            hidden,
            ..
        }
        | SiPkgProp::Map {
            name,
            func_unique_id,
            widget_kind,
            widget_options,
            hidden,
            ..
        }
        | SiPkgProp::Array {
            name,
            func_unique_id,
            widget_kind,
            widget_options,
            hidden,
            ..
        }
        | SiPkgProp::Number {
            name,
            func_unique_id,
            widget_kind,
            widget_options,
            hidden,
            ..
        }
        | SiPkgProp::Object {
            name,
            func_unique_id,
            widget_kind,
            widget_options,
            hidden,
            ..
        }
        | SiPkgProp::Boolean {
            name,
            func_unique_id,
            widget_kind,
            widget_options,
            hidden,
            ..
        } => {
            builder.name(name);
            builder.hidden(*hidden);
            builder.widget_kind(*widget_kind);

            if let Some(widget_options) = widget_options {
                builder.widget_options(widget_options.to_owned());
            }

            if let Some(func_unique_id) = func_unique_id {
                builder.func_unique_id(*func_unique_id);
                for input in spec.inputs()? {
                    builder.input(AttrFuncInputSpec::try_from(input)?);
                }
            }
        }
    }

    let mut stack = ctx.prop_stack.lock().await;
    stack.push_front((path.to_owned(), builder));

    if let Some(parent_path) = parent_path {
        match ctx.prop_parents.lock().await.entry(path.to_owned()) {
            Entry::Occupied(_) => {
                return Err(SiPkgError::prop_tree_invalid(
                    "Prop has more than one parent",
                ));
            }
            Entry::Vacant(entry) => {
                entry.insert(parent_path);
            }
        }
    }

    Ok(Some(path))
}
