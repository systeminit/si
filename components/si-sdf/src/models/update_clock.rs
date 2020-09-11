use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::future::Future;
use std::pin::Pin;

use crate::data::Db;

#[derive(Error, Debug)]
pub enum UpdateClockError {
    #[error("couchbase error: {0}")]
    Couchbase(#[from] couchbase::error::CouchbaseError),
    #[error("update count for this clock exceeded; something is wrong!")]
    UpdateCountExceeded,
}

pub type UpdateClockResult<T> = Result<T, UpdateClockError>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClock {
    pub epoch: u64,
    pub update_count: u64,
}

impl UpdateClock {
    pub fn create_or_update(
        db: &Db,
        workspace_id: impl Into<String>,
        mut count: usize,
    ) -> Pin<Box<dyn Future<Output = UpdateClockResult<UpdateClock>> + Send + Sync>> {
        let db = db.clone();
        let workspace_id = workspace_id.into();
        Box::pin(async move {
            let update_clock_key = format!("{}:updateClock", workspace_id);
            let update_clock_item_result = db
                .bucket
                .default_collection()
                .get(&update_clock_key, None)
                .await;
            let (cas, mut update_clock) = match update_clock_item_result {
                Ok(couchbase_result) => {
                    let cas = couchbase_result.cas();
                    let update_clock: UpdateClock = couchbase_result.content_as()?;
                    (Some(cas), update_clock)
                }
                Err(couchbase::CouchbaseError::KeyDoesNotExist) => {
                    let new_update_clock = UpdateClock {
                        epoch: 1,
                        update_count: 0,
                    };
                    db.bucket
                        .default_collection()
                        .insert(&update_clock_key, new_update_clock, None)
                        .await?;
                    (None, new_update_clock)
                }
                Err(e) => {
                    return Err(UpdateClockError::from(e));
                }
            };
            if cas.is_some() {
                let real_cas = cas.unwrap();
                update_clock.update_count = update_clock.update_count + 1;
                match db
                    .bucket
                    .default_collection()
                    .replace(
                        &update_clock_key,
                        update_clock,
                        Some(couchbase::options::ReplaceOptions::new().set_cas(real_cas)),
                    )
                    .await
                {
                    Ok(_) => return Ok(update_clock),
                    Err(couchbase::CouchbaseError::KeyExists) => {
                        if count > 30 {
                            return Err(UpdateClockError::UpdateCountExceeded);
                        }
                        count = count + 1;
                        return UpdateClock::create_or_update(&db, workspace_id, count).await;
                    }
                    Err(err) => return Err(UpdateClockError::from(err)),
                }
            }
            Ok(update_clock)
        })
    }
}
