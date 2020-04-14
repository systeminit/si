use crate::error::{CeaError, CeaResult};
use crate::list::ListRequest;
use async_trait::async_trait;
use prost::Message;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use si_data::{Db, ListResult, Migrateable, Storable};

pub mod prelude {
    pub use crate::error::{CeaError, CeaResult};
    pub use crate::{gen_component, Component as _, ImplicitConstraints, MigrateComponent};
    pub use si_account::{Integration, IntegrationService};
    pub use si_data::error::DataError;
    pub use si_data::Storable as _;
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImplicitConstraint {
    pub field: String,
    pub value: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImplicitConstraints {
    pub constraints: Vec<ImplicitConstraint>,
}

impl ImplicitConstraints {
    pub fn new() -> ImplicitConstraints {
        ImplicitConstraints::default()
    }

    pub fn add(&mut self, field: impl Into<String>, value: impl Into<String>) {
        self.constraints.push(ImplicitConstraint {
            field: field.into(),
            value: value.into(),
        });
    }
}

pub trait ImplicitConstraintItem {
    fn field(&self) -> &str;
    fn value(&self) -> &str;
    fn set(&mut self, field: impl Into<String>, value: impl Into<String>);
}

impl ImplicitConstraintItem for ImplicitConstraint {
    fn field(&self) -> &str {
        &self.field
    }

    fn value(&self) -> &str {
        &self.field
    }

    fn set(&mut self, field: impl Into<String>, value: impl Into<String>) {
        self.field = field.into();
        self.value = value.into();
    }
}

impl<T: ImplicitConstraintItem + Default> std::convert::From<ImplicitConstraints> for Vec<T> {
    fn from(implicit_constraints: ImplicitConstraints) -> Self {
        let mut return_constraints = Vec::new();
        for constraint in implicit_constraints.constraints.iter() {
            let mut rconstraint = T::default();
            rconstraint.set(constraint.field(), constraint.value());
            return_constraints.push(rconstraint);
        }
        return_constraints
    }
}

pub trait PickComponentRequest {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn integration_id(&self) -> &str;
    fn integration_service_name(&self) -> &str;
}

pub trait PickComponentReply {
    type Component: Component;

    fn component(&self) -> &Self::Component;
    fn implicit_constraints(&self) -> &Vec<ImplicitConstraint>;
}

#[async_trait]
pub trait Component:
    Message + Storable + Migrateable + Serialize + DeserializeOwned + std::fmt::Debug + Default
{
    fn validate(&self) -> si_data::error::Result<()>;

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

    async fn pick_by_expressions(
        db: &Db,
        expressions: Vec<si_data::QueryExpressionOption>,
        boolean_term: si_data::QueryBooleanTerm,
    ) -> CeaResult<Self> {
        let query = si_data::Query {
            items: expressions,
            boolean_term: boolean_term as i32,
            ..Default::default()
        };

        let mut check_result: si_data::ListResult<Self> =
            db.list(&Some(query), 1, "", 0, "global", "").await?;
        if check_result.len() == 1 {
            return Ok(check_result.items.pop().unwrap());
        } else {
            return Err(CeaError::PickComponent("a match was not found".to_string()));
        }
    }

    async fn pick_by_string_field<F, V>(
        db: &si_data::Db,
        field: F,
        value: V,
    ) -> CeaResult<Option<Self>>
    where
        F: Into<String> + Send,
        V: Into<String> + Send,
    {
        let value = value.into();
        let field = field.into();

        if value != "" {
            let query = si_data::Query {
                items: vec![si_data::QueryExpressionOption {
                    expression: Some(si_data::QueryExpressionOptionExpression {
                        field: field.clone(),
                        comparison: si_data::QueryExpressionOptionExpressionComparison::Equals
                            as i32,
                        value: value.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                boolean_term: si_data::QueryBooleanLogic::And as i32,
                ..Default::default()
            };
            let mut check_result: si_data::ListResult<Self> =
                db.list(&Some(query), 1, "", 0, "global", "").await?;
            if check_result.len() == 1 {
                return Ok(Some(check_result.items.pop().unwrap()));
            } else {
                return Err(CeaError::PickComponent(format!(
                    "{}={} must match exactly, and was not found",
                    field, value
                )));
            }
        }
        Ok(None)
    }
}

#[async_trait]
pub trait MigrateComponent: Component {
    async fn migrate(db: &Db) -> CeaResult<()>;
}

#[macro_export]
macro_rules! gen_component {
    (
    type_name: $type_name: tt
    ,
    order_by_fields: [ $($order_by_field:tt),* ]
    ,
    validate_fn: |$self_ident:ident| $validate_fn:block
    ) => {
        impl si_cea::Component for Component {
            fn validate(&$self_ident) -> si_data::error::Result<()> $validate_fn
        }

        impl si_data::Storable for Component {
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
                match <Self as si_cea::Component>::validate(&self) {
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
                    si_data::Reference::HasOne("integration_id", &self.integration_id),
                    si_data::Reference::HasOne("integration_service_id", &self.integration_service_id),
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
                    "displayTypeName",
                    $($order_by_field),*
                ]
            }
        }

        impl si_data::Migrateable for Component {
            fn get_version(&self) -> i32 {
                self.version
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

        impl From<si_data::ListResult<Component>> for ListComponentsReply {
            fn from(list_result: si_data::ListResult<Component>) -> Self {
                if list_result.items.len() == 0 {
                    ListComponentsReply::default()
                } else {
                    ListComponentsReply {
                        total_count: list_result.total_count(),
                        next_page_token: list_result.page_token().to_string(),
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

        impl si_cea::component::ImplicitConstraintItem for ImplicitConstraint {
            fn field(&self) -> &str {
                &self.field
            }

            fn value(&self) -> &str {
                &self.field
            }

            fn set(&mut self, field: impl Into<String>, value: impl Into<String>) {
                self.field = field.into();
                self.value = value.into();
            }
        }

    };
}
