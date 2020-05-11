use serde::{Deserialize, Serialize};
use si_data::error::{DataError, Result};
use si_data::{Reference, Storable};
use uuid::Uuid;

use super::SiStorable;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListData {
    pub id: String,
    pub name: String,
    pub si_storable: Option<SiStorable>,
}

impl ListData {
    pub fn new<T: Into<String>>(id: T, name: T) -> ListData {
        let name = name.into();
        ListData {
            id: id.into(),
            name: name.clone(),
            si_storable: Some(SiStorable {
                type_name: Some(String::from("list_data")),
                natural_key: Some(name),
                tenant_ids: vec!["global".to_string()],
            }),
        }
    }

    pub fn new_non_global<T: Into<String>>(id: T, name: T) -> ListData {
        let name = name.into();
        ListData {
            id: id.into(),
            name: name.clone(),
            si_storable: Some(SiStorable {
                type_name: Some(String::from("list_data")),
                natural_key: Some(name),
                tenant_ids: vec!["local".to_string()],
            }),
        }
    }
}

impl Storable for ListData {
    fn id(&self) -> Result<&str> {
        Ok(&self.id)
    }

    fn set_id(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.id = id;
    }

    fn type_name() -> &'static str {
        "list_data"
    }

    fn set_type_name(&mut self) {
        self.si_storable
            .as_mut()
            .unwrap()
            .type_name
            .replace(ListData::type_name().to_string());
    }

    fn set_natural_key(&mut self) -> Result<()> {
        self.si_storable
            .as_mut()
            .unwrap()
            .natural_key
            .replace(format!("{}:{}", ListData::type_name(), self.name));
        Ok(())
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        self.si_storable
            .as_mut()
            .unwrap()
            .tenant_ids
            .push(id.into());
    }

    fn natural_key(&self) -> Result<Option<&str>> {
        Ok(None)
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

    fn tenant_ids(&self) -> Result<&[String]> {
        Ok(&self.si_storable.as_ref().unwrap().tenant_ids)
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "name", "type_name", "natural_key"]
    }
}
