use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::{PgError, PgTxn};

use crate::schema::UiMenu;
use crate::{
    standard_model, ComponentId, SchemaError, SchemaId, SchematicKind, StandardModelError, Tenancy,
    Visibility,
};

const UI_MENUS_FOR_NODE_MENU: &str = include_str!("./queries/ui_menus_for_node_menu.sql");

#[derive(Error, Debug)]
pub enum NodeMenuError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MenuItem {
    Category(Category),
    Item(Item),
}

impl MenuItem {
    pub fn add_item_if_category(&mut self, item: MenuItem) {
        if let MenuItem::Category(c) = self {
            c.items.push(item);
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MenuItems {
    list: Vec<MenuItem>,
}

impl Default for MenuItems {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuItems {
    pub fn new() -> Self {
        MenuItems { list: Vec::new() }
    }

    pub async fn update_from_ui_menu(
        &mut self,
        txn: &PgTxn<'_>,
        visibility: &Visibility,
        ui_menu: UiMenu,
    ) -> NodeMenuResult<()> {
        if ui_menu.usable_in_menu(txn, visibility).await? {}
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MenuFilter {
    pub schematic_kind: SchematicKind,
    pub root_component_id: ComponentId,
}

pub async fn get_node_menu(
    txn: &PgTxn<'_>,
    visibility: &Visibility,
    tenancy: &Tenancy,
    filter: &MenuFilter,
) -> NodeMenuResult<Vec<MenuItem>> {
    let rows = txn
        .query(
            UI_MENUS_FOR_NODE_MENU,
            &[
                &tenancy,
                &visibility,
                &filter.root_component_id,
                &filter.schematic_kind.to_string(),
            ],
        )
        .await?;
    let _result: Vec<UiMenu> = standard_model::objects_from_rows(rows)?;
    let _categories: HashMap<String, Category> = HashMap::new();
    //for menu_entry in result.into_iter() {
    //    if menu_entry.name().is_none() {
    //        dbg!("skipping menu entry with no name", &menu_entry);
    //        continue;
    //    }
    //    if let Some(category_full) = menu_entry.category() {
    //        let category_parts: Vec<&str> = category_full.split('.').collect();
    //        let category_parts_count = category_parts.len();
    //        for (category_parts_index, category_name) in category_parts.into_iter().enumerate() {
    //            let category_path = category_parts[0..=category_parts_index];
    //            let category = categories
    //                .entry(category_parts[0..=category_parts_index].join("."))
    //                .or_insert(Category::new(category_name));

    //            if category_parts_index == category_parts_count - 1 {
    //                if let Some(schema) = menu_entry.schema(&txn, &visibility).await? {
    //                    category.items.push(MenuItem::Item(Item::new(
    //                        menu_entry.name().unwrap(),
    //                        *schema.id(),
    //                    )));
    //                } else {
    //                    dbg!("skipping menu entry with no schema");
    //                    dbg!(&menu_entry);
    //                }
    //            }
    //        }
    //    }
    //}

    // TODO: Walk the resulting menu entries, translating them into the right shape.
    //
    // 1. Create all the categories, and populate sub-categories.
    // 2. Add all the items to the categories
    //Ok(result)

    todo!()
}
