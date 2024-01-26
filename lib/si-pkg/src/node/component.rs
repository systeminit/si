use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line,
    /*read_key_value_line_opt,*/ write_key_value_line, /*write_key_value_line_opt,*/
    GraphError, NameStr, NodeChild, NodeKind, NodeWithChildren, ReadBytes, WriteBytes,
};

use super::{component_child::ComponentChild, PkgNode, KEY_DELETED_STR, KEY_UNIQUE_ID_STR};
use crate::{ComponentSpec, ComponentSpecVariant};

const KEY_NAME_STR: &str = "name";
const KEY_VARIANT_STR: &str = "variant";
const KEY_NEEDS_DESTROY_STR: &str = "needs_destroy";
const KEY_DELETION_USER_PK_STR: &str = "deletion_user_pk";
// const KEY_HIDDEN_STR: &str = "hidden";

#[derive(Clone, Debug)]
pub struct ComponentNode {
    pub name: String,
    pub variant: ComponentSpecVariant,
    pub needs_destroy: bool,
    pub deletion_user_pk: Option<String>,
    pub unique_id: String,
    pub deleted: bool,
    pub hidden: bool,
}

impl NameStr for ComponentNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for ComponentNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;
        write_key_value_line(
            writer,
            KEY_VARIANT_STR,
            serde_json::to_string(&self.variant).map_err(GraphError::parse)?,
        )?;
        write_key_value_line(writer, KEY_NEEDS_DESTROY_STR, self.needs_destroy)?;

        let deletion_user_pk_str: String = if let Some(deletion_user_pk) = &self.deletion_user_pk {
            deletion_user_pk.into()
        } else {
            "".into()
        };
        write_key_value_line(writer, KEY_DELETION_USER_PK_STR, deletion_user_pk_str)?;
        write_key_value_line(writer, KEY_UNIQUE_ID_STR, &self.unique_id)?;
        write_key_value_line(writer, KEY_DELETED_STR, self.deleted)?;
        // write_key_value_line_opt(writer, KEY_HIDDEN_STR, Some(self.hidden))?;

        Ok(())
    }
}

impl ReadBytes for ComponentNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let variant_str = read_key_value_line(reader, KEY_VARIANT_STR)?;
        let variant: ComponentSpecVariant =
            serde_json::from_str(&variant_str).map_err(GraphError::parse)?;
        let needs_destroy = bool::from_str(&read_key_value_line(reader, KEY_NEEDS_DESTROY_STR)?)
            .map_err(GraphError::parse)?;

        let deletion_user_pk_str = read_key_value_line(reader, KEY_DELETION_USER_PK_STR)?;
        let deletion_user_pk = if deletion_user_pk_str.is_empty() {
            None
        } else {
            Some(deletion_user_pk_str.to_owned())
        };
        let unique_id = read_key_value_line(reader, KEY_UNIQUE_ID_STR)?;
        let deleted = bool::from_str(&read_key_value_line(reader, KEY_DELETED_STR)?)
            .map_err(GraphError::parse)?;

        // TODO: fix this
        let hidden = false;
        /*
        let hidden = read_key_value_line_opt(reader, KEY_HIDDEN_STR)
            .map_err(GraphError::parse)?
            .map(|hidden| bool::from_str(&hidden)).transpose()
            .map_err(GraphError::parse)?
            .unwrap_or_default();
        */

        Ok(Some(Self {
            name,
            variant,
            needs_destroy,
            deletion_user_pk,
            unique_id,
            deleted,
            hidden,
        }))
    }
}

impl NodeChild for ComponentSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::Component(ComponentNode {
                name: self.name.to_owned(),
                variant: self.variant.to_owned(),
                needs_destroy: self.needs_destroy,
                deletion_user_pk: self.deletion_user_pk.to_owned(),
                unique_id: self.unique_id.to_owned(),
                deleted: self.deleted,
                hidden: self.hidden,
            }),
            vec![
                Box::new(ComponentChild::Attributes(self.attributes.to_owned()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                Box::new(ComponentChild::InputSockets(self.input_sockets.to_owned()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                Box::new(ComponentChild::OutputSockets(
                    self.output_sockets.to_owned(),
                )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                Box::new(ComponentChild::Position(self.position.to_owned()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            ],
        )
    }
}
