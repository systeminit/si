use si_data::{error::DataError, Migrateable, Reference, Storable};
use uuid::Uuid;

use crate::error::AwsEksClusterRuntimeError;
pub use crate::protobuf::Component;

impl Storable for Component {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "aws_eks_cluster_runtime"
    }

    fn set_type_name(&mut self) {
        self.type_name = Component::type_name().to_string();
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", Component::type_name(), uuid);
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.display_name == "" {
            return Err(DataError::ValidationError(
                AwsEksClusterRuntimeError::InvalidMissingDisplayName.to_string(),
            ));
        }
        if self.name == "" {
            return Err(DataError::ValidationError(
                AwsEksClusterRuntimeError::InvalidMissingName.to_string(),
            ));
        }
        Ok(())
    }

    fn get_tenant_ids(&self) -> &[String] {
        &self.tenant_ids
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        self.tenant_ids.push(id.into());
    }

    fn referential_fields(&self) -> Vec<Reference> {
        vec![
            Reference::HasOne("integration_id", &self.integration_id),
            Reference::HasOne("integration_service_id", &self.integration_service_id),
        ]
    }

    fn get_natural_key(&self) -> Option<&str> {
        Some(&self.natural_key)
    }

    fn set_natural_key(&mut self) {
        self.natural_key = format!(
            "{}:{}:{}",
            // This is safe *only* after the object has been created.
            self.get_tenant_ids()[0],
            Component::type_name(),
            self.name
        );
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec![
            "id",
            "naturalKey",
            "typeName",
            "displayName",
            "name",
            "description",
            "displayTypeName",
            "kubernetes_version",
        ]
    }
}

impl Migrateable for Component {
    fn get_version(&self) -> i32 {
        self.version
    }
}
