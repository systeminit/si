use std::{
    collections::{
        HashMap,
        VecDeque,
        hash_map::Entry,
    },
    future::Future,
};

use itertools::Itertools;
use object_tree::{
    Hash,
    HashedNode,
};
use petgraph::prelude::*;
use tokio::sync::Mutex;
use url::Url;

use super::{
    PkgResult,
    SiPkgActionFunc,
    SiPkgError,
    SiPkgLeafFunction,
    SiPkgManagementFunc,
    SiPkgProp,
    SiPkgPropData,
    SiPkgSiPropFunc,
    SiPkgSocket,
    Source,
};
use crate::{
    AttrFuncInputSpec,
    MapKeyFuncSpec,
    PropSpec,
    PropSpecBuilder,
    PropSpecKind,
    SchemaVariantSpec,
    SchemaVariantSpecBuilder,
    SchemaVariantSpecComponentType,
    SchemaVariantSpecData,
    SchemaVariantSpecPropRoot,
    SiPkgAuthFunc,
    SiPkgRootPropFunc,
    node::{
        PkgNode,
        PropChildNode,
        SchemaVariantChildNode,
    },
};

#[derive(Clone, Debug)]
pub struct SiPkgSchemaVariantData {
    version: String,
    link: Option<Url>,
    color: Option<String>,
    component_type: SchemaVariantSpecComponentType,
    func_unique_id: String,
    description: Option<String>,
}

impl SiPkgSchemaVariantData {
    pub fn version(&self) -> &str {
        self.version.as_ref()
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

    pub fn func_unique_id(&self) -> &str {
        self.func_unique_id.as_str()
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

#[derive(Clone, Debug)]
pub struct SiPkgSchemaVariant<'a> {
    version: String,
    pub data: Option<SiPkgSchemaVariantData>,
    unique_id: Option<String>,
    deleted: bool,
    is_builtin: bool,

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
                ));
            }
        };

        let schema_variant = Self {
            version: schema_variant_node.version.to_owned(),
            data: schema_variant_node.data.map(|data| SiPkgSchemaVariantData {
                version: schema_variant_node.version,
                link: data.link,
                color: data.color,
                component_type: data.component_type,
                func_unique_id: data.func_unique_id,
                description: data.description,
            }),
            unique_id: schema_variant_node.unique_id,
            deleted: schema_variant_node.deleted,
            is_builtin: schema_variant_node.is_builtin,
            hash: schema_variant_hashed_node.hash(),
            source: Source::new(graph, node_idx),
        };

        Ok(schema_variant)
    }

    pub fn data(&self) -> Option<&SiPkgSchemaVariantData> {
        self.data.as_ref()
    }

    pub fn version(&self) -> &str {
        self.version.as_ref()
    }

    pub fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }

    pub fn source(&self) -> Source<'a> {
        self.source.clone()
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn is_builtin(&self) -> bool {
        self.is_builtin
    }

    impl_variant_children_from_graph!(sockets, SchemaVariantChildNode::Sockets, SiPkgSocket);
    impl_variant_children_from_graph!(
        leaf_functions,
        SchemaVariantChildNode::LeafFunctions,
        SiPkgLeafFunction
    );
    impl_variant_children_from_graph!(
        action_funcs,
        SchemaVariantChildNode::ActionFuncs,
        SiPkgActionFunc
    );
    impl_variant_children_from_graph!(auth_funcs, SchemaVariantChildNode::AuthFuncs, SiPkgAuthFunc);
    impl_variant_children_from_graph!(
        si_prop_funcs,
        SchemaVariantChildNode::SiPropFuncs,
        SiPkgSiPropFunc
    );
    impl_variant_children_from_graph!(secrets, SchemaVariantChildNode::Secrets, SiPkgProp);
    impl_variant_children_from_graph!(
        secret_definitions,
        SchemaVariantChildNode::SecretDefinition,
        SiPkgProp
    );
    impl_variant_children_from_graph!(
        root_prop_funcs,
        SchemaVariantChildNode::RootPropFuncs,
        SiPkgRootPropFunc
    );

    impl_variant_children_from_graph!(
        management_funcs,
        SchemaVariantChildNode::ManagementFuncs,
        SiPkgManagementFunc
    );

    fn prop_stack_from_source<I>(
        source: &Source<'a>,
        node_idx: NodeIndex,
        parent_info: Option<I>,
    ) -> PkgResult<Vec<(SiPkgProp<'a>, Option<I>)>>
    where
        I: ToOwned + Clone + std::fmt::Debug,
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
                    // Create the child props
                    let child_props = source
                        .graph
                        .neighbors_directed(prop_child_idxs, Outgoing)
                        .map(|child_idx| SiPkgProp::from_graph(source.graph, child_idx))
                        .map_ok(|child_prop| (child_prop, parent_info.to_owned()));

                    // If the node has a child order, honor it
                    let node = source
                        .graph
                        .node_weight(node_idx)
                        .ok_or(SiPkgError::prop_tree_invalid("node index not found"))?
                        .inner();
                    match node.child_order() {
                        Some(child_order) => {
                            // Map the child props by name so we can collect them in order
                            let mut props_by_name = HashMap::new();
                            for child_prop in child_props {
                                let child_prop = child_prop?;
                                if props_by_name
                                    .insert(child_prop.0.name().to_owned(), child_prop)
                                    .is_some()
                                {
                                    return Err(SiPkgError::prop_tree_invalid("duplicate prop"));
                                }
                            }

                            // Collect the child props in reverse order
                            let child_props = child_order
                                .iter()
                                .rev()
                                .map(|name| {
                                    props_by_name
                                        .remove(name)
                                        .ok_or(SiPkgError::prop_tree_invalid("dup prop order"))
                                })
                                .try_collect()?;
                            if !props_by_name.is_empty() {
                                return Err(SiPkgError::prop_tree_invalid("prop order missing"));
                            }
                            child_props
                        }
                        // If there is no child order, just return the child props
                        None => child_props.try_collect()?,
                    }
                }
                // If there are no child props, return empty vec
                None => vec![],
            },
        )
    }

    async fn get_prop_root_idx(
        &self,
        prop_root: SchemaVariantSpecPropRoot,
    ) -> PkgResult<Option<NodeIndex>> {
        let maybe_node_index = self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
            .find(|node_idx| {
                let node = &self.source.graph[*node_idx].inner();
                match prop_root {
                    SchemaVariantSpecPropRoot::Domain => matches!(
                        node,
                        PkgNode::SchemaVariantChild(SchemaVariantChildNode::Domain)
                    ),
                    SchemaVariantSpecPropRoot::ResourceValue => matches!(
                        node,
                        PkgNode::SchemaVariantChild(SchemaVariantChildNode::ResourceValue)
                    ),
                    SchemaVariantSpecPropRoot::Secrets => matches!(
                        node,
                        PkgNode::SchemaVariantChild(SchemaVariantChildNode::Secrets)
                    ),
                    SchemaVariantSpecPropRoot::SecretDefinition => {
                        matches!(
                            node,
                            PkgNode::SchemaVariantChild(SchemaVariantChildNode::SecretDefinition)
                        )
                    }
                    SchemaVariantSpecPropRoot::Resource
                    | SchemaVariantSpecPropRoot::Code
                    | SchemaVariantSpecPropRoot::Qualification => false,
                }
            });

        // NOTE(victor, fletcher): Previous versions of prop trees didn't have the Secrets field
        // under root, so we need to successfully ignore them for backwards compatibility
        if maybe_node_index.is_some() || prop_root == SchemaVariantSpecPropRoot::Secrets {
            Ok(maybe_node_index)
        } else {
            let maybe_kind = match prop_root {
                SchemaVariantSpecPropRoot::Domain => SchemaVariantChildNode::Domain.kind_str(),
                SchemaVariantSpecPropRoot::ResourceValue => {
                    SchemaVariantChildNode::ResourceValue.kind_str()
                }
                SchemaVariantSpecPropRoot::Secrets => SchemaVariantChildNode::Secrets.kind_str(),
                SchemaVariantSpecPropRoot::SecretDefinition => {
                    SchemaVariantChildNode::SecretDefinition.kind_str()
                }
                _ => "resource",
            };

            Err(SiPkgError::SchemaVariantChildNotFound(maybe_kind))
        }
    }

    pub async fn visit_prop_tree<F, Fut, I, C, E>(
        &self,
        prop_root: SchemaVariantSpecPropRoot,
        process_prop_fn: F,
        parent_info: Option<I>,
        context: C,
    ) -> Result<(), E>
    where
        F: Fn(SiPkgProp<'a>, Option<I>, C) -> Fut,
        Fut: Future<Output = Result<Option<I>, E>>,
        E: std::convert::From<SiPkgError>,
        I: ToOwned + Clone + std::fmt::Debug,
        C: 'a + Copy,
    {
        if let Some(prop_root_idx) = self.get_prop_root_idx(prop_root).await? {
            let mut child_node_idxs: Vec<_> = self
                .source
                .graph
                .neighbors_directed(prop_root_idx, Outgoing)
                .collect();
            let prop_root_node_idx = match child_node_idxs.pop() {
                Some(idx) => idx,
                None => Err(SiPkgError::PropRootNotFound(prop_root, self.hash()))?,
            };
            if !child_node_idxs.is_empty() {
                Err(SiPkgError::PropRootMultipleFound(prop_root, self.hash()))?;
            }

            // Skip processing the "root" prop for this tree as a `dal::SchemaVariant::new` already
            // guarantees such a prop has already been created. Rather, we will push all immediate
            // children of the domain prop to be ready for processing.
            let mut stack =
                Self::prop_stack_from_source(&self.source, prop_root_node_idx, parent_info)?;

            while let Some((prop, parent_info)) = stack.pop() {
                let node_idx = prop.source().node_idx;
                let new_parent_info =
                    process_prop_fn(prop, parent_info.to_owned(), context).await?;

                stack.extend(Self::prop_stack_from_source(
                    &self.source,
                    node_idx,
                    new_parent_info,
                )?);
            }
        }

        Ok(())
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    async fn build_prop_specs(
        &self,
        prop_root: SchemaVariantSpecPropRoot,
        builder: &mut SchemaVariantSpecBuilder,
    ) -> PkgResult<()> {
        let context = PropSpecVisitContext {
            prop_stack: Mutex::new(VecDeque::new()),
            prop_parents: Mutex::new(HashMap::new()),
        };

        self.visit_prop_tree(prop_root, create_prop_stack, None, &context)
            .await?;

        let prop_stack = context.prop_stack.into_inner();
        let prop_parents = context.prop_parents.into_inner();
        let mut prop_children: HashMap<String, Vec<PropSpec>> = HashMap::new();
        let mut root_children = vec![];
        // This is a depth-first post-order traversal (parents after children), with the
        // hitch that the child order is reversed because we're reversing a depth-first
        // pre-order traversal
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
                        let type_prop = children.first().ok_or(SiPkgError::prop_tree_invalid(
                            "Array or map prop missing type prop",
                        ))?;
                        prop.type_prop(type_prop.clone());
                    }
                    PropSpecKind::Object => {
                        let mut children = children.clone();
                        // We have to reverse the children to be able to process them in the correct
                        // order, because we processed the children in reverse order (the prop stack
                        // traverses props depth-first, post-order with reverse child order due
                        // to how things get pushed on it)
                        children.reverse();
                        prop.entries(children);
                    }
                    _ => {
                        return Err(SiPkgError::prop_tree_invalid(
                            "Leaf prop (String, Number, Boolean) cannot have children",
                        ));
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
                    root_children.push(spec);
                }
            }
        }

        // We have to reverse the children to be able to process them in the correct
        // order, because we processed the children in reverse order (the prop stack
        // traverses props depth-first, post-order with reverse child order due
        // to how things get pushed on it)
        root_children.reverse();
        for spec in root_children {
            builder.prop(prop_root, spec);
        }

        Ok(())
    }

    pub async fn to_spec(&self) -> PkgResult<SchemaVariantSpec> {
        let mut builder = SchemaVariantSpec::builder();

        builder
            .version(self.version())
            .deleted(self.deleted)
            .is_builtin(self.is_builtin);

        if let Some(unique_id) = self.unique_id() {
            builder.unique_id(unique_id);
        }

        if let Some(data) = self.data() {
            let mut data_builder = SchemaVariantSpecData::builder();

            data_builder.version(self.version());
            data_builder.component_type(data.component_type());

            if let Some(link) = data.link() {
                data_builder.link(link.to_owned());
            }

            if let Some(color) = data.color() {
                data_builder.color(color);
            }

            data_builder.description(data.description().map(ToOwned::to_owned));
            data_builder.func_unique_id(data.func_unique_id());
            builder.data(data_builder.build()?);
        }

        for action_func in self.action_funcs()? {
            builder.action_func(action_func.try_into()?);
        }

        for func in self.auth_funcs()? {
            builder.auth_func(func.try_into()?);
        }

        for func in self.leaf_functions()? {
            builder.leaf_function(func.try_into()?);
        }

        for socket in self.sockets()? {
            builder.socket(socket.try_into()?);
        }

        for si_prop_func in self.si_prop_funcs()? {
            builder.si_prop_func(si_prop_func.try_into()?);
        }

        for management_func in self.management_funcs()? {
            builder.management_func(management_func.try_into()?);
        }

        for func in self.root_prop_funcs()? {
            builder.root_prop_func(func.try_into()?);
        }

        self.build_prop_specs(SchemaVariantSpecPropRoot::Domain, &mut builder)
            .await?;
        self.build_prop_specs(SchemaVariantSpecPropRoot::Secrets, &mut builder)
            .await?;
        self.build_prop_specs(SchemaVariantSpecPropRoot::ResourceValue, &mut builder)
            .await?;

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
    builder.has_data(false);

    let default_value = match &spec {
        SiPkgProp::String { data, .. }
        | SiPkgProp::Boolean { data, .. }
        | SiPkgProp::Number { data, .. } => {
            data.as_ref().and_then(|data| data.default_value.to_owned())
        }
        _ => None,
    };

    match &spec {
        SiPkgProp::String { .. } => {
            builder.kind(PropSpecKind::String);
            if let Some(dv) = default_value {
                builder.default_value(dv);
            }
        }
        SiPkgProp::Json { .. } => {
            builder.kind(PropSpecKind::Json);
            if let Some(dv) = default_value {
                builder.default_value(serde_json::to_value(dv)?);
            }
        }
        SiPkgProp::Boolean { .. } => {
            builder.kind(PropSpecKind::Boolean);
            if let Some(dv) = default_value {
                builder.default_value(serde_json::to_value(dv)?);
            }
        }
        SiPkgProp::Number { .. } => {
            builder.kind(PropSpecKind::Number);
            if let Some(dv) = default_value {
                builder.default_value(dv);
            }
        }
        SiPkgProp::Float { .. } => {
            builder.kind(PropSpecKind::Float);
            if let Some(dv) = default_value {
                builder.default_value(dv);
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
            for map_key_func in spec.map_key_funcs()? {
                builder.map_key_func(MapKeyFuncSpec::try_from(map_key_func)?);
            }
        }
    }

    match &spec {
        SiPkgProp::String { name, data, .. }
        | SiPkgProp::Json { name, data, .. }
        | SiPkgProp::Map { name, data, .. }
        | SiPkgProp::Array { name, data, .. }
        | SiPkgProp::Number { name, data, .. }
        | SiPkgProp::Float { name, data, .. }
        | SiPkgProp::Object { name, data, .. }
        | SiPkgProp::Boolean { name, data, .. } => {
            builder.name(name);

            if let Some(SiPkgPropData {
                name,
                hidden,
                widget_kind,
                widget_options,
                func_unique_id,
                doc_link,
                documentation,
                default_value,
                validation_format,
                ui_optionals,
            }) = data
            {
                builder
                    .has_data(true)
                    .name(name)
                    .hidden(*hidden)
                    .widget_kind(*widget_kind)
                    .ui_optionals(ui_optionals.clone());

                if let Some(widget_options) = widget_options {
                    builder.widget_options(widget_options.to_owned());
                }

                if let Some(func_unique_id) = func_unique_id {
                    builder.func_unique_id(func_unique_id.as_str());
                    for input in spec.inputs()? {
                        builder.input(AttrFuncInputSpec::try_from(input)?);
                    }
                }

                if let Some(doc_link) = doc_link {
                    builder.doc_link(doc_link.to_owned());
                }

                if let Some(documentation) = documentation {
                    builder.documentation(documentation.to_owned());
                }

                if let Some(default_value) = default_value {
                    builder.default_value(default_value.to_owned());
                }

                if let Some(validation_format) = validation_format {
                    builder.validation_format(validation_format.to_owned());
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
