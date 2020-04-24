use prost::Message;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

use crate::Authentication;
use si_data::query::DataQuery;

pub trait ListReply: Message + std::fmt::Debug + Default {
    type Reply: Message + std::fmt::Debug + Default;

    fn items(&self) -> &Vec<Self::Reply>;
    fn set_items(&mut self, items: Vec<Self::Reply>);
    fn total_count(&self) -> u32;
    fn set_total_count(&mut self, total_count: u32);
    fn next_page_token(&self) -> Option<&str>;
    fn set_next_page_token(&mut self, page_token: Option<impl Into<String>>);
}

pub trait ListRequest: Message + Serialize + DeserializeOwned + std::fmt::Debug + Default {
    fn query(&self) -> &Option<DataQuery>;
    fn set_query(&mut self, query: Option<DataQuery>);
    fn page_size(&self) -> u32;
    fn set_page_size(&mut self, page_size: u32);
    fn order_by(&self) -> &str;
    fn set_order_by(&mut self, order_by: impl Into<String>);
    fn order_by_direction(&self) -> i32;
    fn set_order_by_direction(&mut self, order_by_direction: i32);
    fn page_token(&self) -> Option<&str>;
    fn set_page_token(&mut self, page_token: Option<impl Into<String>>);
    fn scope_by_tenant_id(&self) -> &str;
    fn set_scope_by_tenant_id(&mut self, scope_by_tenant_id: impl Into<String>);

    fn default_scope_by_tenant_id(&mut self, auth: &Authentication) {
        if self.scope_by_tenant_id() == "" {
            self.set_scope_by_tenant_id(auth.billing_account_id());
        }
    }
}
