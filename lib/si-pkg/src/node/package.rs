use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use object_tree::{
    read_key_value_line, read_key_value_line_opt, write_key_value_line, GraphError, NameStr,
    NodeChild, NodeKind, NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::{PkgSpec, SiPkgKind};

use super::{category::PackageCategory, PkgNode};

const KEY_CREATED_AT_STR: &str = "created_at";
const KEY_CREATED_BY_STR: &str = "created_by";
const KEY_DEFAULT_CHANGE_SET: &str = "default_change_set";
const KEY_DESCRIPTION_STR: &str = "description";
const KEY_KIND_STR: &str = "kind";
const KEY_NAME_STR: &str = "name";
const KEY_VERSION_STR: &str = "version";
const KEY_WORKSPACE_PK_STR: &str = "workspace_pk";

#[derive(Clone, Debug)]
pub struct PackageNode {
    pub kind: SiPkgKind,
    pub name: String,
    pub version: String,

    pub description: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub default_change_set: Option<String>,
    pub workspace_pk: Option<String>,
}

impl NameStr for PackageNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for PackageNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, &self.kind.to_string())?;
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;
        write_key_value_line(writer, KEY_VERSION_STR, &self.version)?;
        write_key_value_line(writer, KEY_DESCRIPTION_STR, &self.description)?;
        write_key_value_line(writer, KEY_CREATED_AT_STR, self.created_at.to_rfc3339())?;
        write_key_value_line(writer, KEY_CREATED_BY_STR, &self.created_by)?;
        if let Some(default_change_set) = &self.default_change_set {
            write_key_value_line(writer, KEY_DEFAULT_CHANGE_SET, default_change_set.as_str())?;
        }
        if let Some(workspace_pk) = &self.workspace_pk {
            write_key_value_line(writer, KEY_WORKSPACE_PK_STR, workspace_pk.as_str())?;
        }
        Ok(())
    }
}

impl ReadBytes for PackageNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind = match read_key_value_line_opt(reader, KEY_KIND_STR)? {
            None => SiPkgKind::Module,
            Some(kind_str) => SiPkgKind::from_str(&kind_str).map_err(GraphError::parse)?,
        };
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let version = read_key_value_line(reader, KEY_VERSION_STR)?;
        let description = read_key_value_line(reader, KEY_DESCRIPTION_STR)?;
        let created_at_str = read_key_value_line(reader, KEY_CREATED_AT_STR)?;
        let created_at = created_at_str
            .parse::<DateTime<Utc>>()
            .map_err(GraphError::parse)?;
        let created_by = read_key_value_line(reader, KEY_CREATED_BY_STR)?;
        let default_change_set = read_key_value_line_opt(reader, KEY_DEFAULT_CHANGE_SET)?;
        let workspace_pk = read_key_value_line_opt(reader, KEY_WORKSPACE_PK_STR)?;

        Ok(Some(Self {
            kind,
            name,
            version,
            description,
            created_at,
            created_by,
            default_change_set,
            workspace_pk,
        }))
    }
}

impl NodeChild for PkgSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::Package(PackageNode {
                kind: self.kind,
                name: self.name.to_string(),
                version: self.version.to_string(),
                description: self.description.to_string(),
                created_at: self.created_at,
                created_by: self.created_by.to_owned(),
                default_change_set: self.default_change_set.to_owned(),
                workspace_pk: self.workspace_pk.to_owned(),
            }),
            match self.kind {
                SiPkgKind::Module => vec![
                    Box::new(PackageCategory::Schemas(self.schemas.clone()))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PackageCategory::Funcs(self.funcs.clone()))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
                SiPkgKind::WorkspaceBackup => {
                    vec![
                        Box::new(PackageCategory::ChangeSets(self.change_sets.clone()))
                            as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    ]
                }
            },
        )
    }
}
