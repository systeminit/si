use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NodeChild, NodeKind, NodeWithChildren,
    ReadBytes, WriteBytes,
};

use super::{PkgNode, KEY_DELETED_STR, KEY_UNIQUE_ID_STR};
use crate::{EdgeSpec, EdgeSpecKind};

const KEY_EDGE_KIND_STR: &str = "edge_kind";
const KEY_FROM_COMPONENT_UNIQUE_ID_STR: &str = "from_component_unique_id";
const KEY_FROM_SOCKET_NAME_STR: &str = "from_socket_name";
const KEY_TO_COMPONENT_UNIQUE_ID_STR: &str = "to_component_unique_id";
const KEY_TO_SOCKET_NAME_STR: &str = "to_socket_name";
const KEY_DELETION_USER_PK_STR: &str = "deletion_user_pk";
const KEY_CREATION_USER_PK_STR: &str = "creation_user_pk";
const KEY_DELETED_IMPLICITLY_STR: &str = "deleted_implicitly";

#[derive(Clone, Debug)]
pub struct EdgeNode {
    pub edge_kind: EdgeSpecKind,
    pub from_component_unique_id: String,
    pub from_socket_name: String,
    pub to_component_unique_id: String,
    pub to_socket_name: String,
    pub creation_user_pk: Option<String>,
    pub deletion_user_pk: Option<String>,
    pub deleted_implicitly: bool,

    pub unique_id: String,
    pub deleted: bool,
}

impl WriteBytes for EdgeNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_EDGE_KIND_STR, self.edge_kind)?;

        write_key_value_line(
            writer,
            KEY_FROM_COMPONENT_UNIQUE_ID_STR,
            &self.from_component_unique_id,
        )?;
        write_key_value_line(writer, KEY_FROM_SOCKET_NAME_STR, &self.from_socket_name)?;
        write_key_value_line(
            writer,
            KEY_TO_COMPONENT_UNIQUE_ID_STR,
            &self.to_component_unique_id,
        )?;
        write_key_value_line(writer, KEY_TO_SOCKET_NAME_STR, &self.to_socket_name)?;

        let creation_user_pk_str: String = if let Some(creation_user_pk) = &self.creation_user_pk {
            creation_user_pk.into()
        } else {
            "".into()
        };
        write_key_value_line(writer, KEY_CREATION_USER_PK_STR, creation_user_pk_str)?;

        let deletion_user_pk_str: String = if let Some(deletion_user_pk) = &self.deletion_user_pk {
            deletion_user_pk.into()
        } else {
            "".into()
        };
        write_key_value_line(writer, KEY_DELETION_USER_PK_STR, deletion_user_pk_str)?;

        write_key_value_line(writer, KEY_DELETED_IMPLICITLY_STR, self.deleted_implicitly)?;

        write_key_value_line(writer, KEY_UNIQUE_ID_STR, &self.unique_id)?;
        write_key_value_line(writer, KEY_DELETED_STR, self.deleted)?;

        Ok(())
    }
}

impl ReadBytes for EdgeNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let edge_kind_str = read_key_value_line(reader, KEY_EDGE_KIND_STR)?;
        let edge_kind = EdgeSpecKind::from_str(&edge_kind_str).map_err(GraphError::parse)?;

        let from_component_unique_id =
            read_key_value_line(reader, KEY_FROM_COMPONENT_UNIQUE_ID_STR)?;
        let from_socket_name = read_key_value_line(reader, KEY_FROM_SOCKET_NAME_STR)?;
        let to_component_unique_id = read_key_value_line(reader, KEY_TO_COMPONENT_UNIQUE_ID_STR)?;
        let to_socket_name = read_key_value_line(reader, KEY_TO_SOCKET_NAME_STR)?;

        let creation_user_pk_str = read_key_value_line(reader, KEY_CREATION_USER_PK_STR)?;
        let creation_user_pk = if creation_user_pk_str.is_empty() {
            None
        } else {
            Some(creation_user_pk_str.to_owned())
        };

        let deletion_user_pk_str = read_key_value_line(reader, KEY_DELETION_USER_PK_STR)?;
        let deletion_user_pk = if deletion_user_pk_str.is_empty() {
            None
        } else {
            Some(deletion_user_pk_str.to_owned())
        };

        let deleted_implicitly =
            bool::from_str(&read_key_value_line(reader, KEY_DELETED_IMPLICITLY_STR)?)
                .map_err(GraphError::parse)?;

        let unique_id = read_key_value_line(reader, KEY_UNIQUE_ID_STR)?;
        let deleted = bool::from_str(&read_key_value_line(reader, KEY_DELETED_STR)?)
            .map_err(GraphError::parse)?;

        Ok(Some(Self {
            edge_kind,
            from_component_unique_id,
            from_socket_name,
            to_component_unique_id,
            to_socket_name,
            creation_user_pk,
            deletion_user_pk,
            deleted_implicitly,

            unique_id,
            deleted,
        }))
    }
}

impl NodeChild for EdgeSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::Edge(EdgeNode {
                edge_kind: self.edge_kind,
                from_component_unique_id: self.from_component_unique_id.to_owned(),
                from_socket_name: self.from_socket_name.to_owned(),
                to_component_unique_id: self.to_component_unique_id.to_owned(),
                to_socket_name: self.to_socket_name.to_owned(),

                creation_user_pk: self.creation_user_pk.to_owned(),
                deletion_user_pk: self.deletion_user_pk.to_owned(),
                deleted_implicitly: self.deleted_implicitly,

                unique_id: self.unique_id.to_owned(),
                deleted: self.deleted,
            }),
            vec![],
        )
    }
}
