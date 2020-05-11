use serde::{Deserialize, Serialize};
use si_data::error::{DataError, Result};
use si_data::{Reference, Storable};
use uuid::Uuid;

// Test relationsip expansion and update
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipData {
    pub id: String,
    pub name: String,
    pub natural_key: String,
    pub type_name: String,
    pub belongs_to_test_id: String,
    pub tenant_ids: Vec<String>,
}

impl RelationshipData {
    pub fn new<T: Into<String>>(id: T, name: T, belongs_to_test_id: T) -> RelationshipData {
        let name = name.into();
        RelationshipData {
            id: id.into(),
            name: name.clone(),
            type_name: String::from("relationship_data"),
            belongs_to_test_id: belongs_to_test_id.into(),
            natural_key: name,
            tenant_ids: vec!["global".to_string()],
        }
    }
}

impl Storable for RelationshipData {
    fn id(&self) -> Result<&str> {
        Ok(&self.id)
    }

    fn set_id(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.id = id;
    }

    fn type_name() -> &'static str {
        "relationship_data"
    }

    fn set_type_name(&mut self) {
        self.type_name = RelationshipData::type_name().to_string();
    }

    fn set_natural_key(&mut self) -> Result<()> {
        self.natural_key = format!(
            "{}:{}:{}",
            // This is safe *only* after the object has been created.
            self.tenant_ids()?[0],
            RelationshipData::type_name(),
            self.name
        );
        Ok(())
    }

    fn natural_key(&self) -> Result<Option<&str>> {
        Ok(Some(&self.natural_key))
    }

    fn tenant_ids(&self) -> Result<&[String]> {
        Ok(&self.tenant_ids)
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        self.tenant_ids.push(id.into());
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", RelationshipData::type_name(), uuid);
    }

    fn validate(&self) -> Result<()> {
        if self.name == "mr bean" {
            return Err(DataError::ValidationError("no mr bean here".to_string()));
        }
        Ok(())
    }

    fn referential_fields(&self) -> Vec<Reference> {
        vec![Reference::HasOne(
            "belongs_to_test_id",
            &self.belongs_to_test_id,
        )]
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "name", "type_name", "natural_key"]
    }
}
