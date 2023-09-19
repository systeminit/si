use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, read_key_value_line_opt, write_key_value_line, GraphError, NameStr,
    ReadBytes, WriteBytes,
};

mod action_func;
mod attr_func_input;
mod category;
mod change_set;
mod change_set_child;
mod func;
mod func_argument;
mod leaf_function;
mod map_key_func;
mod package;
mod prop;
mod prop_child;
mod schema;
mod schema_variant;
mod schema_variant_child;
mod si_prop_func;
mod socket;
mod validation;

pub(crate) use self::{
    action_func::ActionFuncNode,
    attr_func_input::AttrFuncInputNode,
    category::CategoryNode,
    change_set::ChangeSetNode,
    change_set_child::{ChangeSetChild, ChangeSetChildNode},
    func::FuncNode,
    func_argument::FuncArgumentNode,
    leaf_function::LeafFunctionNode,
    map_key_func::MapKeyFuncNode,
    package::PackageNode,
    prop::{PropNode, PropNodeData},
    prop_child::PropChildNode,
    schema::SchemaNode,
    schema_variant::SchemaVariantNode,
    schema_variant_child::{SchemaVariantChild, SchemaVariantChildNode},
    si_prop_func::SiPropFuncNode,
    socket::SocketNode,
    validation::ValidationNode,
};

const NODE_KIND_ACTION_FUNC: &str = "action_func";
const NODE_KIND_ATTR_FUNC_INPUT: &str = "attr_func_input";
const NODE_KIND_CATEGORY: &str = "category";
const NODE_KIND_CHANGE_SET: &str = "change_set";
const NODE_KIND_CHANGE_SET_CHILD: &str = "change_set_child";
const NODE_KIND_FUNC: &str = "func";
const NODE_KIND_FUNC_ARGUMENT: &str = "func_argument";
const NODE_KIND_LEAF_FUNCTION: &str = "leaf_function";
const NODE_KIND_MAP_KEY_FUNC: &str = "map_key_func";
const NODE_KIND_PACKAGE: &str = "package";
const NODE_KIND_PROP: &str = "prop";
const NODE_KIND_PROP_CHILD: &str = "prop_child";
const NODE_KIND_SCHEMA: &str = "schema";
const NODE_KIND_SCHEMA_VARIANT: &str = "schema_variant";
const NODE_KIND_SCHEMA_VARIANT_CHILD: &str = "schema_variant_child";
const NODE_KIND_SOCKET: &str = "socket";
const NODE_KIND_SI_PROP_FUNC: &str = "si_prop_func";
const NODE_KIND_VALIDATION: &str = "validation";

const KEY_NODE_KIND_STR: &str = "node_kind";

const KEY_UNIQUE_ID_STR: &str = "unique_id";
const KEY_DELETED_STR: &str = "deleted";

pub(crate) fn read_unique_id<R: BufRead>(reader: &mut R) -> Result<Option<String>, GraphError> {
    let unique_id_opt_str = read_key_value_line_opt(reader, KEY_UNIQUE_ID_STR)?;
    Ok(unique_id_opt_str.and_then(|unique_id| {
        if unique_id.is_empty() {
            None
        } else {
            Some(unique_id)
        }
    }))
}

fn read_common_fields<R: BufRead>(reader: &mut R) -> Result<(Option<String>, bool), GraphError> {
    let unique_id = read_unique_id(reader)?;

    let deleted = match read_key_value_line_opt(reader, KEY_DELETED_STR)? {
        None => false,
        Some(deleted_str) => bool::from_str(&deleted_str).map_err(GraphError::parse)?,
    };

    Ok((unique_id, deleted))
}

fn write_unique_id<W: Write>(writer: &mut W, unique_id: Option<&str>) -> Result<(), GraphError> {
    write_key_value_line(writer, KEY_UNIQUE_ID_STR, unique_id.unwrap_or(""))?;
    Ok(())
}

fn write_common_fields<W: Write>(
    writer: &mut W,
    unique_id: Option<&str>,
    deleted: bool,
) -> Result<(), GraphError> {
    write_unique_id(writer, unique_id)?;
    write_key_value_line(writer, KEY_DELETED_STR, deleted)?;

    Ok(())
}

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum PkgNode {
    ActionFunc(ActionFuncNode),
    AttrFuncInput(AttrFuncInputNode),
    Category(CategoryNode),
    ChangeSet(ChangeSetNode),
    ChangeSetChild(ChangeSetChildNode),
    Func(FuncNode),
    FuncArgument(FuncArgumentNode),
    LeafFunction(LeafFunctionNode),
    MapKeyFunc(MapKeyFuncNode),
    Package(PackageNode),
    Prop(PropNode),
    PropChild(PropChildNode),
    Schema(SchemaNode),
    SchemaVariant(SchemaVariantNode),
    SchemaVariantChild(SchemaVariantChildNode),
    SiPropFunc(SiPropFuncNode),
    Socket(SocketNode),
    Validation(ValidationNode),
}

impl PkgNode {
    pub const ACTION_FUNC_KIND_STR: &str = NODE_KIND_ACTION_FUNC;
    pub const ATTR_FUNC_INPUT_KIND_STR: &str = NODE_KIND_ATTR_FUNC_INPUT;
    pub const CATEGORY_KIND_STR: &str = NODE_KIND_CATEGORY;
    pub const CHANGE_SET_KIND_STR: &str = NODE_KIND_CHANGE_SET;
    pub const CHANGE_SET_CHILD_KIND_STR: &str = NODE_KIND_CHANGE_SET_CHILD;
    pub const FUNC_KIND_STR: &str = NODE_KIND_FUNC;
    pub const FUNC_ARGUMENT_KIND_STR: &str = NODE_KIND_FUNC_ARGUMENT;
    pub const LEAF_FUNCTION_KIND_STR: &str = NODE_KIND_LEAF_FUNCTION;
    pub const MAP_KEY_FUNC_KIND_STR: &str = NODE_KIND_MAP_KEY_FUNC;
    pub const PACKAGE_KIND_STR: &str = NODE_KIND_PACKAGE;
    pub const PROP_KIND_STR: &str = NODE_KIND_PROP;
    pub const PROP_CHILD_KIND_STR: &str = NODE_KIND_PROP_CHILD;
    pub const SCHEMA_KIND_STR: &str = NODE_KIND_SCHEMA;
    pub const SCHEMA_VARIANT_KIND_STR: &str = NODE_KIND_SCHEMA_VARIANT;
    pub const SCHEMA_VARIANT_KIND_CHILD_STR: &str = NODE_KIND_SCHEMA_VARIANT_CHILD;
    pub const SOCKET_KIND_STR: &str = NODE_KIND_SOCKET;
    pub const SI_PROP_FUNC_KIND_STR: &str = NODE_KIND_SI_PROP_FUNC;
    pub const VALIDATION_KIND_STR: &str = NODE_KIND_VALIDATION;

    pub fn node_kind_str(&self) -> &'static str {
        match self {
            Self::AttrFuncInput(_) => NODE_KIND_ATTR_FUNC_INPUT,
            Self::Category(_) => NODE_KIND_CATEGORY,
            Self::ChangeSet(_) => NODE_KIND_CHANGE_SET,
            Self::ChangeSetChild(_) => NODE_KIND_CHANGE_SET_CHILD,
            Self::ActionFunc(_) => NODE_KIND_ACTION_FUNC,
            Self::Func(_) => NODE_KIND_FUNC,
            Self::FuncArgument(_) => NODE_KIND_FUNC_ARGUMENT,
            Self::LeafFunction(_) => NODE_KIND_LEAF_FUNCTION,
            Self::MapKeyFunc(_) => NODE_KIND_MAP_KEY_FUNC,
            Self::Package(_) => NODE_KIND_PACKAGE,
            Self::Prop(_) => NODE_KIND_PROP,
            Self::PropChild(_) => NODE_KIND_PROP_CHILD,
            Self::Schema(_) => NODE_KIND_SCHEMA,
            Self::SchemaVariant(_) => NODE_KIND_SCHEMA_VARIANT,
            Self::SchemaVariantChild(_) => NODE_KIND_SCHEMA_VARIANT_CHILD,
            Self::Socket(_) => NODE_KIND_SOCKET,
            Self::SiPropFunc(_) => NODE_KIND_SI_PROP_FUNC,
            Self::Validation(_) => NODE_KIND_VALIDATION,
        }
    }
}

impl NameStr for PkgNode {
    fn name(&self) -> &str {
        match self {
            Self::AttrFuncInput(node) => node.name(),
            Self::Category(node) => node.name(),
            Self::ChangeSet(node) => node.name(),
            Self::ChangeSetChild(node) => node.name(),
            Self::ActionFunc(_) => NODE_KIND_ACTION_FUNC,
            Self::Func(node) => node.name(),
            Self::FuncArgument(node) => node.name(),
            Self::LeafFunction(_) => NODE_KIND_LEAF_FUNCTION,
            Self::MapKeyFunc(_) => NODE_KIND_MAP_KEY_FUNC,
            Self::Package(node) => node.name(),
            Self::Prop(node) => node.name(),
            Self::PropChild(node) => node.name(),
            Self::Schema(node) => node.name(),
            Self::SchemaVariant(node) => node.name(),
            Self::SchemaVariantChild(node) => node.name(),
            Self::Socket(node) => node.name(),
            Self::SiPropFunc(_) => NODE_KIND_SI_PROP_FUNC,
            Self::Validation(_) => NODE_KIND_VALIDATION,
        }
    }
}

impl WriteBytes for PkgNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NODE_KIND_STR, self.node_kind_str())?;

        match self {
            Self::AttrFuncInput(node) => node.write_bytes(writer)?,
            Self::Category(node) => node.write_bytes(writer)?,
            Self::ChangeSet(node) => node.write_bytes(writer)?,
            Self::ChangeSetChild(node) => node.write_bytes(writer)?,
            Self::ActionFunc(node) => node.write_bytes(writer)?,
            Self::Func(node) => node.write_bytes(writer)?,
            Self::FuncArgument(node) => node.write_bytes(writer)?,
            Self::LeafFunction(node) => node.write_bytes(writer)?,
            Self::MapKeyFunc(node) => node.write_bytes(writer)?,
            Self::Package(node) => node.write_bytes(writer)?,
            Self::Prop(node) => node.write_bytes(writer)?,
            Self::PropChild(node) => node.write_bytes(writer)?,
            Self::Schema(node) => node.write_bytes(writer)?,
            Self::SchemaVariant(node) => node.write_bytes(writer)?,
            Self::SchemaVariantChild(node) => node.write_bytes(writer)?,
            Self::Socket(node) => node.write_bytes(writer)?,
            Self::SiPropFunc(node) => node.write_bytes(writer)?,
            Self::Validation(node) => node.write_bytes(writer)?,
        };

        Ok(())
    }
}

impl ReadBytes for PkgNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let node_kind_str = read_key_value_line(reader, KEY_NODE_KIND_STR)?;

        let node = match node_kind_str.as_str() {
            NODE_KIND_ACTION_FUNC => ActionFuncNode::read_bytes(reader)?.map(Self::ActionFunc),
            NODE_KIND_ATTR_FUNC_INPUT => {
                AttrFuncInputNode::read_bytes(reader)?.map(Self::AttrFuncInput)
            }
            NODE_KIND_CATEGORY => CategoryNode::read_bytes(reader)?.map(Self::Category),
            NODE_KIND_CHANGE_SET => ChangeSetNode::read_bytes(reader)?.map(Self::ChangeSet),
            NODE_KIND_CHANGE_SET_CHILD => {
                ChangeSetChildNode::read_bytes(reader)?.map(Self::ChangeSetChild)
            }
            NODE_KIND_FUNC => FuncNode::read_bytes(reader)?.map(Self::Func),
            NODE_KIND_FUNC_ARGUMENT => {
                FuncArgumentNode::read_bytes(reader)?.map(Self::FuncArgument)
            }
            NODE_KIND_LEAF_FUNCTION => {
                LeafFunctionNode::read_bytes(reader)?.map(Self::LeafFunction)
            }
            NODE_KIND_MAP_KEY_FUNC => MapKeyFuncNode::read_bytes(reader)?.map(Self::MapKeyFunc),
            NODE_KIND_PACKAGE => PackageNode::read_bytes(reader)?.map(Self::Package),
            NODE_KIND_PROP => PropNode::read_bytes(reader)?.map(Self::Prop),
            NODE_KIND_PROP_CHILD => PropChildNode::read_bytes(reader)?.map(Self::PropChild),
            NODE_KIND_SCHEMA => SchemaNode::read_bytes(reader)?.map(Self::Schema),
            NODE_KIND_SCHEMA_VARIANT => {
                SchemaVariantNode::read_bytes(reader)?.map(Self::SchemaVariant)
            }
            NODE_KIND_SCHEMA_VARIANT_CHILD => {
                SchemaVariantChildNode::read_bytes(reader)?.map(Self::SchemaVariantChild)
            }
            NODE_KIND_SOCKET => SocketNode::read_bytes(reader)?.map(Self::Socket),
            NODE_KIND_SI_PROP_FUNC => SiPropFuncNode::read_bytes(reader)?.map(Self::SiPropFunc),
            NODE_KIND_VALIDATION => ValidationNode::read_bytes(reader)?.map(Self::Validation),
            invalid_kind => {
                dbg!(format!("invalid package node kind: {invalid_kind}"));
                None
            }
        };

        Ok(node)
    }
}
