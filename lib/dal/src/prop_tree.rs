use crate::{
    property_editor::schema::WidgetKind, DalContext, InternalProviderId, Prop, PropId, PropKind,
    SchemaVariantId, StandardModel, StandardModelError, TransactionsError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const PROP_TREE_FOR_ALL_SCHEMA_VARIANTS: &str =
    include_str!("queries/prop/tree_for_all_schema_variants.sql");

#[derive(Error, Debug)]
pub enum PropTreeError {
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
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
        schema_variant_id_filter: Option<SchemaVariantId>,
    ) -> PropTreeResult<Self> {
        let mut root_props = vec![];

        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                PROP_TREE_FOR_ALL_SCHEMA_VARIANTS,
                &[ctx.tenancy(), ctx.visibility()],
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

            if let Some(schema_variant_id_filter) = schema_variant_id_filter {
                if schema_variant_id_filter != schema_variant_id {
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
}
