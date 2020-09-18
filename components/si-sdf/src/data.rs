use nats::asynk::Connection;
use thiserror::Error;

pub use si_data::Db;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("database error: {0}")]
    Data(#[from] si_data::DataError),
    #[error("couchbase error: {0}")]
    Couchbase(#[from] couchbase::CouchbaseError),
}

pub type DataResult<T> = Result<T, DataError>;

pub async fn create_index(db: &Db, index: impl AsRef<str>) -> DataResult<()> {
    let index = index.as_ref();
    let mut result = db.cluster.query(index, None).await?;
    let meta = result.meta().await?;
    match meta.errors {
        Some(error) => tracing::debug!(?error, "index already exists"),
        None => tracing::debug!("created index"),
    }
    Ok(())
}

pub async fn create_indexes(db: &Db) -> DataResult<()> {
    create_index(
        &db,
        format!(
            "CREATE INDEX `idx_si_storable_typename` on `{bucket}`(siStorable.typeName)",
            bucket = db.bucket_name
        ),
    )
    .await?;
    create_index(
        &db,
        format!(
            "CREATE INDEX `idx_si_changeset_changesetid` on `{bucket}`(siChangeSet.changeSetId)",
            bucket = db.bucket_name
        ),
    )
    .await?;
    create_index(
        &db,
        format!(
            "CREATE INDEX `idx_id` on `{bucket}`(id)",
            bucket = db.bucket_name
        ),
    )
    .await?;

    Ok(())
}

pub async fn delete_data(db: &Db) -> DataResult<()> {
    let delete_query = format!(
        "DELETE FROM `{bucket}` WHERE id IS VALUED",
        bucket = db.bucket_name
    );
    let mut result = db.cluster.query(delete_query, None).await?;
    let meta = result.meta().await?;
    match meta.errors {
        Some(error) => tracing::error!("issue deleting: {}", error),
        None => (),
    }
    Ok(())
}
