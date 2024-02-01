//! This module is responsible for creating NodeMenus. At the moment, it only really makes
//! the node add menu. It creates a tree for the menu, and can create it from the
//! [`Schema`](crate::Schema)'s menu items based on the diagram context for the menu.

use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::collections::{hash_map, HashMap};
use thiserror::Error;

use crate::schema::variant::SchemaVariantError;
use crate::{DalContext, Schema, SchemaVariant};
use crate::{SchemaError, SchemaId, StandardModel, StandardModelError};

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum NodeMenuError {
    #[error("cannot get inner category for non category menu item")]
    NoInnerCategory,
    #[error("cannot find menu entry; path does not exist: {0}")]
    PathDoesNotExist(String),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model: {0}")]
    StandardModel(#[from] StandardModelError),
}

pub type NodeMenuResult<T> = Result<T, NodeMenuError>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Category {
    pub name: String,
    pub items: Vec<MenuItem>,
}

impl Category {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let items = Vec::new();
        Category { name, items }
    }

    pub fn push(&mut self, menu_item: MenuItem) {
        self.items.push(menu_item);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    pub name: String,
    pub schema_id: SchemaId,
}

impl Item {
    pub fn new(name: impl Into<String>, schema_id: SchemaId) -> Self {
        let name = name.into();
        Item { name, schema_id }
    }
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MenuItem {
    Category(Category),
    Item(Item),
}

impl MenuItem {
    pub fn category(name: impl Into<String>) -> MenuItem {
        MenuItem::Category(Category::new(name))
    }

    pub fn item(name: impl Into<String>, schema_id: SchemaId) -> MenuItem {
        MenuItem::Item(Item::new(name, schema_id))
    }

    pub fn name(&self) -> &str {
        match self {
            MenuItem::Category(c) => &c.name,
            MenuItem::Item(i) => &i.name,
        }
    }

    pub fn inner_category(&self) -> NodeMenuResult<&Category> {
        match self {
            MenuItem::Category(c) => Ok(c),
            _ => Err(NodeMenuError::NoInnerCategory),
        }
    }
}

/// Used to generate a [`serde_json::Value`] of menu items.
#[derive(Deserialize, Serialize, Debug)]
pub struct GenerateMenuItem {
    menu_items: Vec<MenuItem>,
}

impl GenerateMenuItem {
    /// Generates raw items and initializes menu items as an empty vec.
    pub async fn new(ctx: &DalContext, include_ui_hidden: bool) -> NodeMenuResult<Self> {
        let mut item_map: HashMap<String, Vec<MenuItem>> = HashMap::new();

        for schema in Schema::list(ctx).await? {
            for variant in SchemaVariant::list_for_schema(ctx, schema.id()).await? {
                let category = variant.category().to_owned();

                let item = MenuItem::Item(Item {
                    name: schema.name().to_owned(),
                    schema_id: schema.id(),
                });

                item_map
                    .entry(category)
                    .and_modify(|items| items.push(item.to_owned()))
                    .or_insert(vec![item]);
            }
        }

        let mut menu_items: Vec<MenuItem> = item_map
            .into_iter()
            .map(|(name, mut items)| {
                items.sort_by_key(|item| item.name().to_owned());
                MenuItem::Category(Category { name, items })
            })
            .collect();

        menu_items.sort_by_key(|item| item.name().to_owned());

        Ok(Self { menu_items })
    }

    /// Create a usable [`serde_json::Value`] from the raw menu items assembled from
    /// [`Self::new()`].
    pub fn create_menu_json(self) -> NodeMenuResult<serde_json::Value> {
        Ok(serde_json::to_value(self.menu_items.clone())?)
    }
}
