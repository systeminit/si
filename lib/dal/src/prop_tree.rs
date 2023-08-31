use crate::ChangeSetPk;
use crate::{
    property_editor::schema::WidgetKind, DalContext, InternalProviderId, Prop, PropId, PropKind,
    SchemaError, SchemaVariant, SchemaVariantError, SchemaVariantId, StandardModel,
    StandardModelError, TransactionsError,
};
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const PROP_TREE_FOR_ALL_SCHEMA_VARIANTS: &str =
    include_str!("queries/prop/tree_for_all_schema_variants.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PropTreeError {
    #[error("Prop {0} is an array but has no element prop")]
    ArrayMissingElementProp(PropId),
    #[error("Prop {0} is a map but has no element prop")]
    MapMissingElementProp(PropId),
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type PropTreeResult<T> = Result<T, PropTreeError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropTreeNode {
    pub children: Vec<PropTreeNode>,
    pub parent_id: PropId,
    pub prop_id: PropId,
    pub visibility_change_set_pk: ChangeSetPk,
    pub kind: PropKind,
    pub schema_variant_id: SchemaVariantId,
    pub internal_provider_id: Option<InternalProviderId>,
    pub path: String,
    pub name: String,
    pub hidden: bool,
    pub widget_kind: WidgetKind,
    pub widget_options: Option<serde_json::Value>,
    pub doc_link: Option<String>,
}

impl PropTreeNode {
    pub fn ts_type(&self) -> PropTreeResult<String> {
        Ok(match self.kind {
            PropKind::Array => {
                let array_element_type = self
                    .children
                    .get(0)
                    .ok_or(PropTreeError::ArrayMissingElementProp(self.prop_id))?;

                format!("{}[] | null | undefined", array_element_type.ts_type()?)
            }
            PropKind::Boolean => "boolean | null | undefined".into(),
            PropKind::Integer => "number | null | undefined".into(),
            PropKind::Object => {
                let mut object_interface = "{\n".to_string();
                for child in &self.children {
                    // We serialize the object key as a JSON string because its
                    // the easiest way to ensure we create a valid TS interface
                    // even with keys that are not valid javascript identifiers.
                    // (e.g., we escape quotes in the prop name this way)
                    let name_value = serde_json::to_value(&child.name)?;
                    let name_serialized = serde_json::to_string(&name_value)?;
                    object_interface.push_str(
                        format!("{}: {};\n", &name_serialized, child.ts_type()?).as_str(),
                    );
                }
                object_interface.push('}');

                object_interface
            }
            PropKind::Map => {
                let map_element_type = self
                    .children
                    .get(0)
                    .ok_or(PropTreeError::MapMissingElementProp(self.prop_id))?;

                format!(
                    "Record<string, {}> | null | undefined",
                    map_element_type.ts_type()?
                )
            }
            PropKind::String => "string | null | undefined".into(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropTree {
    pub root_props: Vec<PropTreeNode>,
}

// recursively insert a prop node into the tree. We should never see
// prop tree depths big enough for a stack issue here (famous last words!)
fn insert_prop_into(root: &mut PropTreeNode, node: &PropTreeNode) {
    if node.parent_id == root.prop_id {
        root.children.push(node.clone());
    } else {
        for child in root.children.iter_mut() {
            insert_prop_into(child, node)
        }
    }
}

impl PropTree {
    pub async fn new(
        ctx: &DalContext,
        include_hidden: bool,
        schema_variant_id_filter: Option<Vec<SchemaVariantId>>,
        root_prop: Option<PropId>,
    ) -> PropTreeResult<Self> {
        let mut root_props = vec![];

        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                PROP_TREE_FOR_ALL_SCHEMA_VARIANTS,
                &[ctx.tenancy(), ctx.visibility(), &root_prop, &include_hidden],
            )
            .await?;

        for row in rows {
            let prop_json: serde_json::Value = row.try_get("object")?;
            let prop: Prop = serde_json::from_value(prop_json)?;
            if prop.hidden() && !include_hidden {
                continue;
            }
            let parent_id: PropId = row.try_get("parent_id")?;
            let schema_variant_id: SchemaVariantId = row.try_get("schema_variant_id")?;
            let visibility_change_set_pk: ChangeSetPk = row.try_get("visibility_change_set_pk")?;

            if let Some(schema_variant_id_filter) = &schema_variant_id_filter {
                if !schema_variant_id_filter.contains(&schema_variant_id) {
                    continue;
                }
            }

            let internal_provider_id: Option<InternalProviderId> =
                row.try_get("internal_provider_id")?;
            let path: String = row.try_get("path")?;
            let name: String = row.try_get("name")?;
            let root_id: PropId = row.try_get("root_id")?;

            let node = PropTreeNode {
                children: vec![],
                parent_id,
                prop_id: *prop.id(),
                kind: *prop.kind(),
                schema_variant_id,
                internal_provider_id,
                path,
                name,
                hidden: prop.hidden(),
                widget_kind: *prop.widget_kind(),
                widget_options: prop.widget_options().cloned(),
                doc_link: prop.doc_link().map(|l| l.to_owned()),
                visibility_change_set_pk,
            };

            // The ordering of the query ensures parent nodes will always come before their children
            if parent_id.is_none() {
                root_props.push(node);
            } else {
                for root in root_props.iter_mut() {
                    if root.prop_id == root_id {
                        insert_prop_into(root, &node);
                    }
                }
            }
        }

        Ok(PropTree { root_props })
    }

    pub async fn ts_types(&self, ctx: &DalContext) -> PropTreeResult<Vec<(String, String)>> {
        let mut toplevels = vec![];

        for root in &self.root_props {
            let variant = SchemaVariant::get_by_id(ctx, &root.schema_variant_id)
                .await?
                .ok_or(SchemaVariantError::NotFound(root.schema_variant_id))?;

            let schema = variant
                .schema(ctx)
                .await?
                .ok_or(SchemaVariantError::MissingSchema(root.schema_variant_id))?;

            let type_name = format!(
                "{}_{}_{}",
                schema.name().to_case(Case::Pascal),
                variant.name().to_case(Case::Pascal),
                root.name.to_case(Case::Pascal),
            );

            let ts_type = match root.kind {
                PropKind::Object => {
                    format!("interface {} {}", &type_name, root.ts_type()?)
                }
                _ => format!("type {} = {};", &type_name, root.ts_type()?),
            };
            toplevels.push((type_name, ts_type));
        }

        Ok(toplevels)
    }
}
