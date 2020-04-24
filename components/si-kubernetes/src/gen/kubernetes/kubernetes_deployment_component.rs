// Auth-generated code!
// No touchy!

use si_cea::component::prelude::*;

pub use crate::protobuf::{
    KubernetesDeploymentComponent, KubernetesDeploymentComponentConstraints,
    KubernetesDeploymentComponentListReply, KubernetesDeploymentComponentListRequest,
    KubernetesDeploymentComponentPickRequest,
};

impl si_cea::Component for KubernetesDeploymentComponent {
    // Validates only that top-level required arguments exist.
    // Deep validation of arguments happens at the GraphQL layer.
    fn validate(&self) -> si_data::error::Result<()> {
        if self.id.is_none() {
            return Err(DataError::ValidationError(
                "missing required id value".into(),
            ));
        }
        if self.name.is_none() {
            return Err(DataError::ValidationError(
                "missing required name value".into(),
            ));
        }
        if self.display_name.is_none() {
            return Err(DataError::ValidationError(
                "missing required display_name value".into(),
            ));
        }
        if self.description.is_none() {
            return Err(DataError::ValidationError(
                "missing required description value".into(),
            ));
        }
        if self.display_type_name.is_none() {
            return Err(DataError::ValidationError(
                "missing required display_type_name value".into(),
            ));
        }
        if self.si_storable.is_none() {
            return Err(DataError::ValidationError(
                "missing required si_storable value".into(),
            ));
        }
        if self.si_properties.is_none() {
            return Err(DataError::ValidationError(
                "missing required si_properties value".into(),
            ));
        }
        Ok(())
    }

    fn display_type_name() -> &'static str {
        "Kubernetes Deployment Object"
    }

    fn set_display_type_name(&mut self) {
        self.display_type_name = Some(Self::display_type_name().to_string());
    }
}

impl si_data::Storable for KubernetesDeploymentComponent {
    /// # Panics
    ///
    /// * When a component's `id` is not set (`Component::generate_id()` must be called first)
    fn get_id(&self) -> &str {
        (self.id.as_ref())
            .expect("Component::generate_id() must be called before Component::get_id")
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    fn type_name() -> &'static str {
        "kubernetes_deployment"
    }

    fn set_type_name(&mut self) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }

        let storable = self.si_storable.as_mut().unwrap();
        storable.type_name = Some(<Self as si_data::Storable>::type_name().to_string());
    }

    fn generate_id(&mut self) {
        self.set_id(format!(
            "{}:{}",
            <Self as si_data::Storable>::type_name(),
            uuid::Uuid::new_v4(),
        ));
    }

    fn validate(&self) -> si_data::error::Result<()> {
        match <Self as si_cea::Component>::validate(&self) {
            Ok(()) => Ok(()),
            Err(e) => Err(si_data::error::DataError::ValidationError(e.to_string())),
        }
    }

    fn get_tenant_ids(&self) -> &[String] {
        match &self.si_storable {
            Some(storable) => &storable.tenant_ids,
            None => &[],
        }
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }

        let storable = self.si_storable.as_mut().unwrap();
        storable.tenant_ids.push(id.into());
    }

    fn referential_fields(&self) -> Vec<si_data::Reference> {
        const NO_INTEGRATION_ID: &str = "<NO_INTEGRATION_ID_HERE_ROLAND>";
        const NO_INTEGRATION_SERVICE_ID: &str = "<NO_INTEGRATION_SERVICE_ID_HERE_ROLAND>";

        let integration_id = match &self.si_properties {
            Some(cip) => cip
                .integration_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or(NO_INTEGRATION_ID),
            None => NO_INTEGRATION_ID,
        };
        let integration_service_id = match &self.si_properties {
            Some(cip) => cip
                .integration_service_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or(NO_INTEGRATION_ID),
            None => NO_INTEGRATION_SERVICE_ID,
        };

        vec![
            si_data::Reference::HasOne("integration_id", integration_id),
            si_data::Reference::HasOne("integration_service_id", integration_service_id),
        ]
    }

    fn get_natural_key(&self) -> Option<&str> {
        self.si_storable
            .as_ref()
            .and_then(|s| s.natural_key.as_ref().map(String::as_ref))
    }

    /// # Panics
    ///
    /// This method will panic if any required information is missing to generate a natural key:
    ///
    /// * When a component's `tenant_ids` are not set
    /// * When a component's `name` is not set
    fn set_natural_key(&mut self) {
        if let None = self.si_storable {
            self.si_storable = Some(Default::default());
        }
        let natural_key = format!(
            "{}:{}:{}",
            self.get_tenant_ids().first().expect(
                "Component's tenant_ids must be set with Component.set_natural_key() is called"
            ),
            <Self as si_data::Storable>::type_name(),
            self.name
                .as_ref()
                .expect("Component.name must be set when Component.set_natural_key() is called")
        );

        let mut storable = self.si_storable.as_mut().unwrap();
        storable.natural_key = Some(natural_key);
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "displayName",
            "description",
            "displayTypeName",
            "constraints.componentName",
            "constraints.componentDisplayName",
            "constraints.kubernetesVersion",
        ]
    }
}

impl si_data::Migrateable for KubernetesDeploymentComponent {
    fn get_version(&self) -> i32 {
        self.si_properties
            .as_ref()
            .and_then(|cip| cip.version)
            .unwrap_or(0)
    }
}

impl si_cea::ListReply for KubernetesDeploymentComponentListReply {
    type Reply = KubernetesDeploymentComponent;

    fn items(&self) -> &Vec<Self::Reply> {
        &self.items
    }

    fn set_items(&mut self, items: Vec<Self::Reply>) {
        self.items = items;
    }

    fn total_count(&self) -> u32 {
        self.total_count.unwrap_or(0)
    }

    fn set_total_count(&mut self, total_count: u32) {
        self.total_count = Some(total_count);
    }

    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_ref().map(String::as_ref)
    }

    fn set_next_page_token(&mut self, page_token: Option<impl Into<String>>) {
        self.next_page_token = page_token.map(|s| s.into());
    }
}

impl From<si_data::ListResult<KubernetesDeploymentComponent>>
    for KubernetesDeploymentComponentListReply
{
    fn from(list_result: si_data::ListResult<KubernetesDeploymentComponent>) -> Self {
        if list_result.items.len() == 0 {
            Self::default()
        } else {
            let next_page_token = if list_result.page_token().is_empty() {
                None
            } else {
                Some(list_result.page_token().to_string())
            };

            Self {
                total_count: Some(list_result.total_count()),
                next_page_token,
                items: list_result.items,
            }
        }
    }
}

impl si_cea::ListRequest for KubernetesDeploymentComponentListRequest {
    fn query(&self) -> &Option<si_data::DataQuery> {
        &self.query
    }

    fn set_query(&mut self, query: Option<si_data::DataQuery>) {
        self.query = query;
    }

    fn page_size(&self) -> u32 {
        self.page_size.unwrap_or(0)
    }

    fn set_page_size(&mut self, page_size: u32) {
        self.page_size = Some(page_size);
    }

    fn order_by(&self) -> &str {
        self.order_by.as_ref().map(String::as_ref).unwrap_or("ASC")
    }

    fn set_order_by(&mut self, order_by: impl Into<String>) {
        self.order_by = Some(order_by.into());
    }

    fn order_by_direction(&self) -> i32 {
        self.order_by_direction
    }

    fn set_order_by_direction(&mut self, order_by_direction: i32) {
        self.order_by_direction = order_by_direction;
    }

    fn page_token(&self) -> Option<&str> {
        self.page_token.as_ref().map(String::as_ref)
    }

    fn set_page_token(&mut self, page_token: Option<impl Into<String>>) {
        self.page_token = page_token.map(|s| s.into());
    }

    fn scope_by_tenant_id(&self) -> &str {
        self.scope_by_tenant_id
            .as_ref()
            .map(String::as_ref)
            .unwrap_or("")
    }

    fn set_scope_by_tenant_id(&mut self, scope_by_tenant_id: impl Into<String>) {
        self.scope_by_tenant_id = Some(scope_by_tenant_id.into());
    }
}
