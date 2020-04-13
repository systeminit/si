use crate::error::CeaResult;
use crate::list::ListRequest;
use async_trait::async_trait;
use prost::Message;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use si_data::db::ListResult;
use si_data::{Db, Storable};

pub mod prelude {
    pub use crate::{
        gen_entity, gen_entity_event, CeaError, CeaResult, Entity as _, ListReply as _,
        ListRequest as _,
    };
    pub use si_data::{ListResult, Query, Storable as _};
    pub use uuid::Uuid;
}

pub enum EntityState {
    Uninitialized = 0,
    Error = 1,
    Ok = 2,
    Transition = 3,
}

#[async_trait]
pub trait Entity:
    std::fmt::Debug + Message + Serialize + DeserializeOwned + Clone + Storable + Default
{
    fn id(&self) -> &str;
    fn set_id(&mut self, id: impl Into<String>);
    fn state(&self) -> i32;
    fn set_state(&mut self, state: i32);
    fn component_id(&self) -> &str;
    fn set_component_id(&mut self, component_id: impl Into<String>);
    fn integration_id(&self) -> &str;
    fn set_integration_id(&mut self, integration_id: impl Into<String>);
    fn integration_service_id(&self) -> &str;
    fn set_integration_service_id(&mut self, integration_service_id: impl Into<String>);
    fn workspace_id(&self) -> &str;
    fn set_workspace_id(&mut self, workspace_id: impl Into<String>);
    fn organization_id(&self) -> &str;
    fn set_organization_id(&mut self, organization_id: impl Into<String>);
    fn billing_account_id(&self) -> &str;
    fn set_billing_account_id(&mut self, billing_account_id: impl Into<String>);
    fn validate(&self) -> CeaResult<()>;

    fn set_state_transition(&mut self) {
        self.set_state(EntityState::Transition as i32);
    }

    async fn save(&self, db: &Db) -> CeaResult<()> {
        db.upsert(self).await?;
        Ok(())
    }

    async fn get(db: &Db, id: &str) -> CeaResult<Self> {
        let entity = db.get(id).await?;
        Ok(entity)
    }

    async fn list<T: ListRequest>(db: &Db, list_request: &T) -> CeaResult<ListResult<Self>> {
        let result = if list_request.has_page_token() {
            db.list_by_page_token(list_request.page_token()).await?
        } else {
            db.list(
                list_request.query(),
                list_request.page_size(),
                list_request.order_by(),
                list_request.order_by_direction(),
                list_request.scope_by_tenant_id(),
                "",
            )
            .await?
        };
        Ok(result)
    }
}

#[macro_export]
macro_rules! gen_entity {
    (
       type_name: $type_name:tt
       ,
       order_by_fields: [ $($order_by_field:tt),* ],
       validate_fn: |$self_ident:ident| $validate_fn:block
    ) => {
        impl si_cea::Entity for Entity {
            fn id(&self) -> &str {
                &self.id
            }

            fn set_id(&mut self, id: impl Into<String>) {
                self.id = id.into();
            }

            fn state(&self) -> i32 {
                self.state
            }

            fn set_state(&mut self, state: i32) {
                self.state = state;
            }

            fn component_id(&self) -> &str {
                self.component_id.as_ref()
            }

            fn set_component_id(&mut self, component_id: impl Into<String>) {
                self.component_id = component_id.into();
            }

            fn integration_id(&self) -> &str {
                self.integration_id.as_ref()
            }

            fn set_integration_id(&mut self, integration_id: impl Into<String>) {
                self.integration_id = integration_id.into();
            }

            fn integration_service_id(&self) -> &str {
                self.integration_service_id.as_ref()
            }

            fn set_integration_service_id(&mut self, integration_service_id: impl Into<String>) {
                self.integration_service_id = integration_service_id.into();
            }

            fn workspace_id(&self) -> &str {
                self.workspace_id.as_ref()
            }

            fn set_workspace_id(&mut self, workspace_id: impl Into<String>) {
                self.workspace_id = workspace_id.into();
            }

            fn organization_id(&self) -> &str {
                self.organization_id.as_ref()
            }

            fn set_organization_id(&mut self, organization_id: impl Into<String>) {
                self.organization_id = organization_id.into();
            }

            fn billing_account_id(&self) -> &str {
                self.billing_account_id.as_ref()
            }

            fn set_billing_account_id(&mut self, billing_account_id: impl Into<String>) {
                self.billing_account_id = billing_account_id.into();
            }

            fn validate(&$self_ident) -> Result<(), si_cea::CeaError> {
                $validate_fn
            }
        }

        impl si_cea::ListReply for ListEntitiesReply {
            type Reply = Entity;

            fn items(&self) -> &Vec<Self::Reply> {
                &self.items
            }

            fn set_items(&mut self, items: Vec<Self::Reply>) {
                self.items = items;
            }

            fn total_count(&self) -> i32 {
                self.total_count
            }

            fn set_total_count(&mut self, total_count: i32) {
                self.total_count = total_count;
            }

            fn next_page_token(&self) -> &str {
                self.next_page_token.as_ref()
            }

            fn set_next_page_token(&mut self, page_token: impl Into<String>) {
                self.next_page_token = page_token.into();
            }
        }

        impl From<ListResult<Entity>> for ListEntitiesReply {
            fn from(list_result: ListResult<Entity>) -> Self {
                if list_result.items.len() == 0 {
                    ListEntitiesReply::default()
                } else {
                    ListEntitiesReply {
                        total_count: list_result.total_count(),
                        next_page_token: list_result.page_token().to_string(),
                        items: list_result.items,
                    }
                }
            }
        }

        impl si_cea::ListRequest for ListEntitiesRequest {
            fn query(&self) -> &Option<Query> {
                &self.query
            }

            fn set_query(&mut self, query: Option<Query>) {
                self.query = query;
            }

            fn page_size(&self) -> i32 {
                self.page_size
            }

            fn set_page_size(&mut self, page_size: i32) {
                self.page_size = page_size;
            }

            fn order_by(&self) -> &str {
                self.order_by.as_ref()
            }

            fn set_order_by(&mut self, order_by: impl Into<String>) {
                self.order_by = order_by.into();
            }

            fn order_by_direction(&self) -> i32 {
                self.order_by_direction
            }

            fn set_order_by_direction(&mut self, order_by_direction: i32) {
                self.order_by_direction = order_by_direction;
            }

            fn page_token(&self) -> &str {
                self.page_token.as_ref()
            }

            fn set_page_token(&mut self, page_token: impl Into<String>) {
                self.page_token = page_token.into()
            }

            fn scope_by_tenant_id(&self) -> &str {
                self.scope_by_tenant_id.as_ref()
            }

            fn set_scope_by_tenant_id(&mut self, scope_by_tenant_id: impl Into<String>) {
                self.scope_by_tenant_id = scope_by_tenant_id.into();
            }
        }

        impl si_data::Storable for Entity {
            fn get_id(&self) -> &str {
                &self.id
            }

            fn set_id(&mut self, id: impl Into<String>) {
                self.id = id.into();
            }

            fn type_name() -> &'static str {
                $type_name
            }

            fn set_type_name(&mut self) {
                self.type_name = <Self as si_data::Storable>::type_name().to_string();
            }

            fn generate_id(&mut self) {
                let uuid = uuid::Uuid::new_v4();
                self.id = format!("{}:{}", <Self as si_data::Storable>::type_name(), uuid);
            }

            fn validate(&self) -> si_data::error::Result<()> {
                match <Self as si_cea::Entity>::validate(&self) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(si_data::error::DataError::ValidationError(e.to_string())),
                }
            }

            fn get_tenant_ids(&self) -> &[String] {
                &self.tenant_ids
            }

            fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
                self.tenant_ids.push(id.into());
            }

            fn referential_fields(&self) -> Vec<si_data::Reference> {
                vec![
                    si_data::Reference::HasOne("component_id", &self.component_id),
                    si_data::Reference::HasOne("integration_id", &self.integration_id),
                    si_data::Reference::HasOne("integration_service_id", &self.integration_service_id),
                    si_data::Reference::HasOne("workspace_id", &self.workspace_id),
                    si_data::Reference::HasOne("organization_id", &self.organization_id),
                    si_data::Reference::HasOne("billing_account_id", &self.billing_account_id),
                ]
            }

            fn get_natural_key(&self) -> Option<&str> {
                Some(&self.natural_key)
            }

            fn set_natural_key(&mut self) {
                self.natural_key = format!(
                    "{}:{}:{}:{}:{}",
                    // This is safe *only* after the object has been created.
                    self.get_tenant_ids()[0],
                    self.get_tenant_ids()[1],
                    self.get_tenant_ids()[2],
                    <Self as si_data::Storable>::type_name(),
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
                    $($order_by_field),*
                ]
            }
        }
    };
}
