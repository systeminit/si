use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

use crate::diagram::DiagramResult;
use crate::schema::SchemaUiMenu;
use crate::socket::{SocketArity, SocketEdgeKind};
use crate::{
    DalContext, DiagramError, Node, NodePosition, SchemaError, SchemaVariant, StandardModel,
};

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
    Input,
    Output,
    Bidirectional,
}

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
    id: String,
    label: String,
    #[serde(rename = "type")]
    ty: String,
    direction: SocketDirection,
    max_connections: Option<usize>,
    is_required: Option<bool>,
    node_side: NodeSide,
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
            .map(|socket| {
                Self {
                    id: socket.id().to_string(),
                    label: socket.name().to_owned(),
                    ty: socket.name().to_owned(),
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
            .collect())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GridPoint {
    x: isize,
    y: isize,
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
    width: isize,
    height: isize,
}

impl Size2D {
    pub fn width(&self) -> isize {
        self.width
    }
    pub fn height(&self) -> isize {
        self.height
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramNodeView {
    id: String,
    #[serde(rename = "type")]
    ty: Option<String>,
    title: String,
    category: Option<String>,
    subtitle: Option<String>,
    content: Option<String>,
    sockets: Option<Vec<SocketView>>,
    position: GridPoint,
    size: Option<Size2D>,
    color: Option<String>,
}

impl DiagramNodeView {
    pub async fn new(
        ctx: &DalContext,
        node: &Node,
        position: &NodePosition,
        schema_variant: &SchemaVariant,
    ) -> DiagramResult<Self> {
        let component = node
            .component(ctx)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;
        let schema = schema_variant
            .schema(ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;
        let diagram_kind = schema
            .diagram_kind()
            .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
        let category =
            SchemaUiMenu::get_by_schema_and_diagram_kind(ctx, *schema.id(), diagram_kind)
                .await?
                .map(|um| um.category().to_string());

        let size = if let (Some(w), Some(h)) = (position.width(), position.height()) {
            Some(Size2D {
                height: h.parse()?,
                width: w.parse()?,
            })
        } else {
            None
        };

        let x = position.x().parse::<f64>()?;
        let y = position.y().parse::<f64>()?;

        Ok(Self {
            id: node.id().to_string(),
            ty: None,
            title: schema.name().to_owned(),
            category,
            subtitle: Some(component.name(ctx).await?),
            content: None,
            sockets: Some(SocketView::list(ctx, schema_variant).await?),
            position: GridPoint {
                x: x.round() as isize,
                y: y.round() as isize,
            },
            size,
            color: component
                .find_value_by_json_pointer::<String>(ctx, "/root/si/Color")
                .await?
                .or_else(|| {
                    schema_variant
                        .color()
                        .map(|color_int| format!("#{color_int:x}"))
                }),
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn position(&self) -> &GridPoint {
        &self.position
    }

    pub fn size(&self) -> &Option<Size2D> {
        &self.size
    }
}
