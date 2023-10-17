//! This module is responsible for creating NodeMenus. At the moment, it only really makes
//! the node add menu. It creates a tree for the menu, and can create it from the
//! [`Schema`](crate::Schema)'s menu items based on the diagram context for the menu.

use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;

use crate::schema::SchemaUiMenu;
use crate::DalContext;
use crate::{SchemaId, StandardModel, StandardModelError};

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
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model: {0}")]
    StandardModel(#[from] StandardModelError),
}

pub type NodeMenuResult<T> = Result<T, NodeMenuError>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Category {
    pub name: String,
    // Whoa! I'm pretty sure this can be refactored to be
    // less intense. But it's working, and I'm a tired of looking
    // at it. So lets take care of that in a refactor the next
    // time we find a problem with this code, eh?
    //
    // Love,
    // Adam
    pub items: Rc<RefCell<Vec<Rc<RefCell<MenuItem>>>>>,
}

impl Category {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let items = Rc::new(RefCell::new(Vec::new()));
        Category { name, items }
    }

    pub fn push(&self, menu_item: MenuItem) {
        self.items
            .borrow_mut()
            .push(Rc::new(RefCell::new(menu_item)));
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MenuItems {
    // Same thing here - we probably need some of these, but
    // likely not all of them? -- Adam
    list: Rc<RefCell<Vec<Rc<RefCell<MenuItem>>>>>,
}

impl Default for MenuItems {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuItems {
    pub fn new() -> Self {
        MenuItems {
            list: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Search the list of menu items for the path given, and return the menu item
    /// when it is found.
    pub fn item_for_path(&self, path: &[String]) -> NodeMenuResult<Rc<RefCell<MenuItem>>> {
        let mut current_list = self.list.clone();
        let final_path_index = if path.is_empty() { 0 } else { path.len() - 1 };
        for (path_idx, path_part) in path.iter().enumerate() {
            let ref_list = current_list.clone();
            if let Some(menu_item) = ref_list
                .borrow()
                .iter()
                .find(|i| i.borrow().name() == *path_part)
            {
                if path_idx == final_path_index {
                    return Ok(menu_item.clone());
                } else {
                    current_list = menu_item.clone().borrow().inner_category()?.items.clone();
                }
            } else {
                return Err(NodeMenuError::PathDoesNotExist(path.join(".")));
            }; // <- ensures the borrow above doesn't live too long
        }
        Err(NodeMenuError::PathDoesNotExist(path.join(".")))
    }

    /// Insert a new menu item into the list, creating any categories that might not exist along
    /// the way, and eventually adding the `menu_item` to the list for the correct categories.
    pub fn insert_menu_item(&self, path: &[String], menu_item: MenuItem) -> NodeMenuResult<()> {
        let fallback_path = vec![menu_item.name().to_string()];
        let path_to_check = if path.is_empty() {
            path
        } else {
            &fallback_path
        };
        match self.item_for_path(path_to_check) {
            Ok(parent) => {
                let pb = parent.borrow();
                let f = pb.inner_category()?;
                f.push(menu_item);
            }
            Err(NodeMenuError::PathDoesNotExist(_)) => {
                if path.is_empty() {
                    self.list
                        .borrow_mut()
                        .push(Rc::new(RefCell::new(menu_item)));
                } else {
                    let mut insert_into = self.list.clone();
                    for (path_idx, path_part) in path.iter().enumerate() {
                        match self.item_for_path(&path[0..=path_idx]) {
                            Ok(parent) => {
                                insert_into = parent.borrow().inner_category()?.items.clone();
                            }
                            _ => {
                                let new_category =
                                    Rc::new(RefCell::new(MenuItem::category(path_part.clone())));
                                insert_into.borrow_mut().push(new_category.clone());
                                insert_into = new_category.borrow().inner_category()?.items.clone();
                            }
                        }
                    }
                    insert_into
                        .borrow_mut()
                        .push(Rc::new(RefCell::new(menu_item)));
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn list(&self) -> Rc<RefCell<Vec<Rc<RefCell<MenuItem>>>>> {
        self.list.clone()
    }

    pub fn to_json_value(&self) -> NodeMenuResult<serde_json::Value> {
        Ok(serde_json::to_value(self.list.clone())?)
    }
}

/// Used to generate a [`serde_json::Value`] of menu items.
#[derive(Deserialize, Serialize, Debug)]
pub struct GenerateMenuItem {
    pub raw_items: Vec<(Vec<String>, Item)>,
    menu_items: MenuItems,
}

impl GenerateMenuItem {
    /// Generates raw items and initializes menu items as an empty vec.
    pub async fn new(ctx: &DalContext, include_ui_hidden: bool) -> NodeMenuResult<Self> {
        let mut item_list = Vec::new();

        // NOTE(nick): currently, we only generate ui menus for schemas.
        let mut ui_menus = SchemaUiMenu::list(ctx).await?;

        // Ensure the names and categories are alphabetically sorted.
        ui_menus.sort_by(|a, b| a.name().cmp(b.name()));
        ui_menus.sort_by(|a, b| a.category().cmp(b.category()));

        for ui_menu in ui_menus.into_iter() {
            if let Some(schema) = ui_menu.schema(ctx).await? {
                if !include_ui_hidden && schema.ui_hidden() {
                    continue;
                }
                item_list.push((
                    ui_menu.category_path(),
                    Item::new(ui_menu.name(), *schema.id()),
                ));
            }
        }

        Ok(Self {
            raw_items: item_list,
            menu_items: MenuItems::new(),
        })
    }

    /// Create a usable [`serde_json::Value`] from the raw menu items assembled from
    /// [`Self::new()`].
    pub fn create_menu_json(self) -> NodeMenuResult<serde_json::Value> {
        for (path, item) in self.raw_items {
            self.menu_items
                .insert_menu_item(&path, MenuItem::Item(item))?;
        }
        self.menu_items.to_json_value()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn menu_item_for_top_level_path() {
        let menu_items = MenuItems::new();
        menu_items
            .insert_menu_item(&Vec::new(), MenuItem::category("valorant"))
            .expect("cannot insert menu item");
        let item = menu_items
            .item_for_path(&["valorant".to_string()])
            .expect("cannot find valorant in menu");
        assert_eq!(item.borrow().name(), "valorant");
    }

    #[test]
    fn nested_menu_items_for_top_level_path() {
        let menu_items = MenuItems::new();
        menu_items
            .insert_menu_item(
                &["planes".to_string(), "snakes".to_string()],
                MenuItem::item("ninjas", SchemaId::generate()),
            )
            .expect("cannot insert menu item");
        let item = menu_items
            .item_for_path(&[
                "planes".to_string(),
                "snakes".to_string(),
                "ninjas".to_string(),
            ])
            .expect("cannot find planes.snakes in menu");
        assert_eq!(item.borrow().name(), "ninjas".to_string());
    }

    #[test]
    fn multiple_nested_menu_items_for_top_level_path() {
        let menu_items = MenuItems::new();
        menu_items
            .insert_menu_item(
                &["planes".to_string(), "snakes".to_string()],
                MenuItem::item("ninjas", SchemaId::generate()),
            )
            .expect("cannot insert menu item");
        menu_items
            .insert_menu_item(
                &["planes".to_string(), "snakes".to_string()],
                MenuItem::item("dragons", SchemaId::generate()),
            )
            .expect("cannot insert menu item");
        let ninjas = menu_items
            .item_for_path(&[
                "planes".to_string(),
                "snakes".to_string(),
                "ninjas".to_string(),
            ])
            .expect("cannot find planes.snakes in menu");
        assert_eq!(ninjas.borrow().name(), "ninjas".to_string());
        let dragons = menu_items
            .item_for_path(&[
                "planes".to_string(),
                "snakes".to_string(),
                "dragons".to_string(),
            ])
            .expect("cannot find planes.snakes in menu");
        assert_eq!(dragons.borrow().name(), "dragons".to_string());
    }
}
