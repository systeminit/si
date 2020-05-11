use serde::{Deserialize, Serialize};
use si_data::error::{DataError, Result};
use si_data::Migrateable;
use si_data::{Reference, Storable};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestData {
    pub id: String,
    pub name: String,
    pub natural_key: String,
    pub type_name: String,
    pub reference_id: String,
    pub tenant_ids: Vec<String>,
}

impl TestData {
    pub fn new<T: Into<String>>(id: T, name: T) -> TestData {
        let name = name.into();
        TestData {
            id: id.into(),
            name: name.clone(),
            type_name: String::from("test_data"),
            reference_id: String::from("reference:1"),
            natural_key: name,
            tenant_ids: vec!["global".to_string()],
        }
    }
}

impl Storable for TestData {
    fn id(&self) -> Result<&str> {
        Ok(&self.id)
    }

    fn set_id(&mut self, id: impl Into<String>) {
        let id = id.into();
        self.id = id;
    }

    fn type_name() -> &'static str {
        "test_data"
    }

    fn set_type_name(&mut self) {
        self.type_name = TestData::type_name().to_string();
    }

    fn set_natural_key(&mut self) -> Result<()> {
        self.natural_key = format!("{}:{}", TestData::type_name(), self.name);
        Ok(())
    }

    fn natural_key(&self) -> Result<Option<&str>> {
        Ok(None)
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", TestData::type_name(), uuid);
    }

    fn referential_fields(&self) -> Vec<Reference> {
        Vec::new()
    }

    fn tenant_ids(&self) -> Result<&[String]> {
        Ok(&self.tenant_ids)
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        self.tenant_ids.push(id.into());
    }

    fn validate(&self) -> Result<()> {
        if self.name == "mr bean" {
            return Err(DataError::ValidationError("no mr bean here".to_string()));
        }
        Ok(())
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec!["id", "name", "type_name", "natural_key"]
    }
}

impl Migrateable for TestData {
    fn get_version(&self) -> i32 {
        1
    }
}
