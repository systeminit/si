use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use crate::diagram::DiagramResult;
use crate::socket::{SocketArity, SocketEdgeKind};
use crate::{DalContext, SchemaVariant, StandardModel};

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketDirection {
    Bidirectional,
    Input,
    Output,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum NodeSide {
    Left,
    Right,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SocketView {
    pub id: String,
    pub label: String,
    pub connection_annotations: Vec<String>,
    pub direction: SocketDirection,
    pub max_connections: Option<usize>,
    pub is_required: Option<bool>,
    pub node_side: NodeSide,
}

impl SocketView {
    pub async fn list(
        ctx: &DalContext,
        schema_variant: &SchemaVariant,
    ) -> DiagramResult<Vec<Self>> {
        Ok(schema_variant
            .sockets(ctx)
            .await?
            .into_iter()
            .filter_map(|socket| {
                (!socket.ui_hidden()).then(|| {
                    let connection_annotations =
                        serde_json::from_str(socket.connection_annotations())
                            .unwrap_or(vec![socket.name().to_owned()]);

                    Self {
                        id: socket.id().to_string(),
                        label: socket.human_name().unwrap_or(socket.name()).to_owned(),
                        connection_annotations,
                        // Note: it's not clear if this mapping is correct, and there is no backend support for bidirectional sockets for now
                        direction: match socket.edge_kind() {
                            SocketEdgeKind::ConfigurationOutput => SocketDirection::Output,
                            _ => SocketDirection::Input,
                        },
                        max_connections: match socket.arity() {
                            SocketArity::Many => None,
                            SocketArity::One => Some(1),
                        },
                        is_required: Some(socket.required()),
                        node_side: match socket.edge_kind() {
                            SocketEdgeKind::ConfigurationOutput => NodeSide::Right,
                            _ => NodeSide::Left,
                        },
                    }
                })
            })
            .collect())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GridPoint {
    pub x: isize,
    pub y: isize,
}

impl GridPoint {
    pub fn x(&self) -> isize {
        self.x
    }

    pub fn y(&self) -> isize {
        self.y
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Size2D {
    pub width: isize,
    pub height: isize,
}

impl Size2D {
    pub fn width(&self) -> isize {
        self.width
    }
    pub fn height(&self) -> isize {
        self.height
    }
}
