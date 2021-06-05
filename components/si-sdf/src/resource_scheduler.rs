use std::time::Duration;

use futures::{future::BoxFuture, FutureExt};
use thiserror::Error;
use tokio::time;

use si_data::{NatsConn, PgPool};
use si_model::{EdgeError, Entity, EntityError, Resource, ResourceError, Veritech};

#[derive(Error, Debug)]
pub enum ResourceSchedulerError {
    #[error("deadpool error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
}

pub type ResourceSchedulerResult<T> = Result<T, ResourceSchedulerError>;

pub async fn start(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> ResourceSchedulerResult<()> {
    let mut interval = time::interval(Duration::from_secs(30));
    loop {
        dbg!("waiting for a new resource sync run to be timed");
        interval.tick().await;
        dbg!("starting a new resource sync run");
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let all_entities = Entity::all_head(&txn)
            .await?
            .into_iter()
            .filter(|e| e.entity_type != "system");
        txn.commit().await?;
        for entity in all_entities {
            match sync_resource(&pg, &nats_conn, &veritech, &entity).await {
                Ok(()) => {}
                Err(e) => {
                    dbg!("**** Scheduled resource run failed ***");
                    dbg!(&e);
                }
            }
            //tokio::spawn(sync_future);
        }
    }
}

pub fn sync_resource(
    pg: &PgPool,
    nats_conn: &NatsConn,
    veritech: &Veritech,
    entity: &Entity,
) -> BoxFuture<'static, ResourceSchedulerResult<()>> {
    let entity = entity.clone();
    let pg = pg.clone();
    let veritech = veritech.clone();
    let nats_conn = nats_conn.clone();
    let r = async move {
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let systems: Vec<Entity> = Entity::get_head_by_name_and_entity_type(
            &txn,
            "production",
            "system",
            &entity.si_storable.workspace_id,
        )
        .await?
        .into_iter()
        .filter(|s| s.si_storable.workspace_id == entity.si_storable.workspace_id)
        .collect();
        let system_id = systems.first().unwrap().id.clone();
        let mut r = match Resource::get_by_entity_and_system(&txn, &entity.id, &system_id).await? {
            Some(r) => r,
            None => {
                Resource::new(
                    &pg,
                    &nats_conn,
                    serde_json::json!([]),
                    &entity.id,
                    &system_id,
                    &entity.si_storable.workspace_id,
                )
                .await?
            }
        };
        r.await_sync(pg.clone(), nats_conn.clone(), veritech.clone())
            .await?;
        txn.commit().await?;
        Ok(())
    };
    r.boxed()
}
