use crate::error::{CeaError, CeaResult};
use crate::list::ListRequest;
use async_trait::async_trait;
use prost::Message;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use si_data::{Db, ListResult, Migrateable, Storable};

pub use crate::protobuf::ComponentSiProperties;

pub mod codegen_prelude {
    pub use crate::component::Component;
    pub use crate::list::{ListReply, ListRequest};
    pub use si_data::error::{DataError, Result as DataResult};
    pub use si_data::{uuid_string, ListResult, Migrateable, Storable};
}

pub mod prelude {
    pub use super::MigrateComponent;
    pub use crate::component::Component;
    pub use crate::error::CeaResult;
    pub use si_account::{Integration, IntegrationService};
    pub use si_data::Db;
    pub use si_data::Storable;
}

#[async_trait]
pub trait Component:
    Message + Storable + Migrateable + Serialize + DeserializeOwned + std::fmt::Debug + Default
{
    fn validate(&self) -> si_data::error::Result<()>;

    fn display_type_name() -> &'static str;
    fn set_display_type_name(&mut self);

    async fn get(db: &Db, id: &str) -> CeaResult<Self> {
        let entity = db.get(id).await?;
        Ok(entity)
    }

    async fn list<T: ListRequest>(db: &Db, list_request: &T) -> CeaResult<ListResult<Self>> {
        let result = match list_request.page_token() {
            Some(token) => db.list_by_page_token(token).await?,
            None => {
                db.list(
                    list_request.query(),
                    list_request.page_size(),
                    list_request.order_by(),
                    list_request.order_by_direction(),
                    list_request.scope_by_tenant_id(),
                    "",
                )
                .await?
            }
        };
        Ok(result)
    }

    async fn pick_by_expressions(
        db: &Db,
        items: Vec<si_data::DataQueryItems>,
        boolean_term: si_data::DataQueryBooleanTerm,
    ) -> CeaResult<Self> {
        let query = si_data::DataQuery {
            items,
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
            let query = si_data::DataQuery::generate_for_string(
                field.clone(),
                si_data::DataQueryItemsExpressionComparison::Equals,
                value.clone(),
            );
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
