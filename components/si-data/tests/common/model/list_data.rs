use serde::{Deserialize, Serialize};
use si_data::error::{DataError, Result};
use si_data::{Reference, Storable};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListData {
    pub id: String,
    pub name: String,
    pub natural_key: String,
    pub type_name: String,
    pub tenant_ids: Vec<String>,
}

impl ListData {
    pub fn new<T: Into<String>>(id: T, name: T) -> ListData {
        let name = name.into();
        ListData {
            id: id.into(),
            name: name.clone(),
            type_name: String::from("list_data"),
            natural_key: name,
            tenant_ids: vec!["global".to_string()],
        }
    }
}

impl Storable for ListData {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id<T: Into<String>>(&mut self, id: T) {
        let id = id.into();
        self.id = id;
    }

    fn type_name() -> &'static str {
        "list_data"
    }

    fn set_type_name(&mut self) {
        self.type_name = ListData::type_name().to_string();
    }

    fn set_natural_key(&mut self) {
        self.natural_key = format!("{}:{}", ListData::type_name(), self.name);
    }

    fn add_to_tenant_ids(&mut self, id: String) {
        self.tenant_ids.push(id);
    }

    fn get_natural_key(&self) -> Option<&str> {
        None
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", ListData::type_name(), uuid);
    }

    fn validate(&self) -> Result<()> {
        if self.name == "mr bean" {
            return Err(DataError::ValidationError("no mr bean here".to_string()));
        }
        Ok(())
    }

    fn referential_fields(&self) -> Vec<Reference> {
        Vec::new()
    }

    fn get_tenant_ids(&self) -> &[String] {
        &self.tenant_ids
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "name", "type_name", "natural_key"]
    }
}
