use crate::protobuf::deployment::{
    Constraints, ListComponentsReply, ListComponentsRequest, PickComponentRequest,
};
use si_cea::component::prelude::*;
use si_data::Db;
use std::fmt;

pub use crate::protobuf::deployment::Component;

// NOTE(fnichol): this for sure should not live here long term; it needs to be rolled up into
// top module level info, metadata, etc.
const DEFAULT_KUBERNETES_VERSION: &str = "1.15";
const VALID_KUBERNETES_VERSION_VALUES: &[&str] = &["1.15", "1.14", "1.13", "1.12"];

// gen_component!(
//     type_name: "kubernetes_deployment",
//     order_by_fields: [
//         "kubernetesVersion"
//     ],
//     validate_fn: |self| {
//         if self.display_name.is_none() {
//             return Err(DataError::ValidationError(format!("missing {}", Field::DisplayName)));
//         }
//         if self.name.is_none() {
//             return Err(DataError::ValidationError(format!("missing {}", Field::Name)));
//         }
//         // validate the version is right? seems real.
//         Ok(())
//     }
// );

impl si_cea::Component for Component {
    fn validate(&self) -> si_data::error::Result<()> {
        if self.display_name.is_none() {
            return Err(DataError::ValidationError(format!(
                "missing {}",
                Field::DisplayName
            )));
        }
        if self.name.is_none() {
            return Err(DataError::ValidationError(format!(
                "missing {}",
                Field::Name
            )));
        }
        // validate the version is right? seems real.
        Ok(())
    }

    fn display_type_name() -> &'static str {
        "Kubernetes Deployment"
    }

    fn set_display_type_name(&mut self) {
        self.display_type_name = Some(Self::display_type_name().to_string());
    }
}

impl si_data::Storable for Component {
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
        if let None = self.storable {
            self.storable = Some(Default::default());
        }

        let storable = self.storable.as_mut().unwrap();
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
        match &self.storable {
            Some(storable) => &storable.tenant_ids,
            None => &[],
        }
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        if let None = self.storable {
            self.storable = Some(Default::default());
        }

        let storable = self.storable.as_mut().unwrap();
        storable.tenant_ids.push(id.into());
    }

    fn referential_fields(&self) -> Vec<si_data::Reference> {
        const NO_INTEGRATION_ID: &str = "<NO_INTEGRATION_ID_HERE_ROLAND>";
        const NO_INTEGRATION_SERVICE_ID: &str = "<NO_INTEGRATION_SERVICE_ID_HERE_ROLAND>";

        let integration_id = match &self.component_si_properties {
            Some(cip) => cip
                .integration_id
                .as_ref()
                .map(String::as_ref)
                .unwrap_or(NO_INTEGRATION_ID),
            None => NO_INTEGRATION_ID,
        };
        let integration_service_id = match &self.component_si_properties {
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
        self.storable
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
        if let None = self.storable {
            self.storable = Some(Default::default());
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

        let mut storable = self.storable.as_mut().unwrap();
        storable.natural_key = Some(natural_key);
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
            "kubernetesVersion",
        ]
    }
}

impl si_data::Migrateable for Component {
    fn get_version(&self) -> i32 {
        self.component_si_properties
            .as_ref()
            .and_then(|cip| cip.version)
            .unwrap_or(0)
    }
}

impl si_cea::ListReply for ListComponentsReply {
    type Reply = Component;

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

impl From<si_data::ListResult<Component>> for ListComponentsReply {
    fn from(list_result: si_data::ListResult<Component>) -> Self {
        if list_result.items.len() == 0 {
            ListComponentsReply::default()
        } else {
            let next_page_token = if list_result.page_token().is_empty() {
                None
            } else {
                Some(list_result.page_token().to_string())
            };

            ListComponentsReply {
                total_count: Some(list_result.total_count()),
                next_page_token,
                items: list_result.items,
            }
        }
    }
}

impl si_cea::ListRequest for ListComponentsRequest {
    fn query(&self) -> &Option<si_data::Query> {
        &self.query
    }

    fn set_query(&mut self, query: Option<si_data::Query>) {
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

impl Component {
    pub async fn pick(db: &Db, req: &PickComponentRequest) -> CeaResult<(Constraints, Self)> {
        match &req.constraints {
            None => Err(CeaError::InvalidPickMissingConstraints),
            Some(constraints) => {
                if let Some(found) = Self::pick_by_component_name(db, &constraints).await? {
                    return Ok(found);
                }
                if let Some(found) = Self::pick_by_component_display_name(db, &constraints).await? {
                    return Ok(found);
                }

                let mut implicit_constraints = Constraints::default();
                let mut query_items = Vec::new();

                let kubernetes_version = match &constraints.kubernetes_version {
                    Some(value) => {
                        if Field::is_valid_kubernetes_version(&value) {
                            value.clone()
                        } else {
                            return Err(CeaError::PickComponent(format!(
                                "invalid {}: {}",
                                Field::KubernetesVersion,
                                value
                            )));
                        }
                    }
                    None => {
                        implicit_constraints.kubernetes_version =
                            Some(DEFAULT_KUBERNETES_VERSION.to_string());
                        DEFAULT_KUBERNETES_VERSION.to_string()
                    }
                };
                query_items.push(si_data::QueryItems::generate_expression_for_string(
                    Field::KubernetesVersion.to_string(),
                    si_data::QueryItemsExpressionComparison::Equals,
                    kubernetes_version,
                ));

                let component =
                    Self::pick_by_expressions(db, query_items, si_data::QueryBooleanTerm::And)
                        .await?;

                Ok((implicit_constraints, component))
            }
        }
    }

    // NOTE(fnichol): This can likely be common/cea code?
    async fn pick_by_component_name(
        db: &Db,
        req: &Constraints,
    ) -> CeaResult<Option<(Constraints, Self)>> {
        match &req.component_name {
            Some(name) => {
                match Self::pick_by_string_field(db, Field::Name.to_string(), name).await? {
                    Some(component) => Ok(Some((Constraints::default(), component))),
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    // NOTE(fnichol): This can likely be common/cea code?
    async fn pick_by_component_display_name(
        db: &Db,
        req: &Constraints,
    ) -> CeaResult<Option<(Constraints, Self)>> {
        match &req.component_display_name {
            Some(display_name) => {
                match Self::pick_by_string_field(db, Field::DisplayName.to_string(), display_name)
                    .await?
                {
                    Some(component) => Ok(Some((Constraints::default(), component))),
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }
}

#[async_trait::async_trait]
impl MigrateComponent for Component {
    async fn migrate(db: &Db) -> CeaResult<()> {
        // Should these be internal model calls? Pretty sure they def should.
        let aws_integration: Integration =
            db.lookup_by_natural_key("global:integration:aws").await?;

        let aws_eks_integration_service_id = format!(
            "global:{}:integration_service:eks_kubernetes",
            aws_integration.get_id()
        );

        let aws_eks_integration_service: IntegrationService = db
            .lookup_by_natural_key(aws_eks_integration_service_id)
            .await?;

        for kubernetes_version in VALID_KUBERNETES_VERSION_VALUES {
            let name = format!("AWS EKS Kubernetes {} Deployment", kubernetes_version);
            let mut c = Component {
                name: Some(name.clone()),
                display_name: Some(name.clone()),
                description: Some(name.clone()),

                // integration_id: aws_integration.get_id().to_string(),
                // integration_service_id: aws_eks_integration_service.get_id().to_string(),
                // version: 2,
                // kubernetes_version: kubernetes_version.to_string(),
                ..Default::default()
            };
            c.add_to_tenant_ids("global".to_string());
            c.add_to_tenant_ids(aws_integration.get_id().to_string());
            c.add_to_tenant_ids(aws_eks_integration_service.get_id().to_string());
            db.migrate(&mut c).await?;
        }

        Ok(())
    }
}

enum Field {
    DisplayName,
    KubernetesVersion,
    Name,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::DisplayName => "displayName",
            Self::KubernetesVersion => "kubernetesVersion",
            Self::Name => "name",
        };
        write!(f, "{}", msg)
    }
}

impl Field {
    fn is_valid_kubernetes_version(s: &str) -> bool {
        VALID_KUBERNETES_VERSION_VALUES.contains(&s)
    }
}
