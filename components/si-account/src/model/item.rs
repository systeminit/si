use si_data::{Db, ListResult, Result};

pub use crate::protobuf::{Item, ItemGetReply, ItemGetRequest, ItemListReply, ItemListRequest};
use tracing::debug;

// TODO: Make create and get changeset aware.

impl Item {
    pub async fn get(db: &Db, id: impl Into<String>) -> Result<Item> {
        let item = db.get(id.into()).await?;
        Ok(item)
    }

    pub async fn list(db: &Db, list_request: ItemListRequest) -> Result<ListResult<Item>> {
        let result = match list_request.page_token {
            Some(token) => db.list_by_page_token_raw(token).await?,
            None => {
                let page_size = match list_request.page_size {
                    Some(page_size) => page_size,
                    None => 10,
                };
                let order_by = match list_request.order_by {
                    Some(order_by) => order_by,
                    // The empty string is the signal for a default, thanks protobuf history
                    None => "".to_string(),
                };
                let contained_within = match list_request.scope_by_tenant_id {
                    Some(contained_within) => contained_within,
                    None => return Err(si_data::DataError::MissingScopeByTenantId),
                };
                db.list_raw(
                    &list_request.query,
                    page_size,
                    order_by,
                    list_request.order_by_direction,
                    contained_within,
                    "",
                )
                .await?
            }
        };
        let mut real_items: Vec<Item> = vec![];
        for i in result.items.into_iter() {
            let r = serde_json::from_value(i)?;
            real_items.push(r);
        }
        Ok(ListResult {
            items: real_items,
            total_count: result.total_count.clone(),
            next_item_id: result.next_item_id.clone(),
            page_token: result.page_token.clone(),
        })
    }
}
