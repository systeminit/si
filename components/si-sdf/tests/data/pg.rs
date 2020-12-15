use crate::SETTINGS;
use si_sdf::data::pg::PgPool;

#[tokio::test]
async fn pg_pool_new() {
    let pg = PgPool::new(&SETTINGS).await.expect("pool creation to work");
    for i in 1..10 {
        let client = pg
            .pool
            .get()
            .await
            .expect("cannot get a connection from the pool");
        let stmt = client
            .prepare("SELECT 1 + $1")
            .await
            .expect("cannot prepare a statement");
        let rows = client
            .query(&stmt, &[&i])
            .await
            .expect("cannot run a query");
        let value: i32 = rows[0].get(0);
        assert_eq!(value, i + 1);
    }
}

#[tokio::test]
async fn pg_pool_si_id_check() {
    let pg = PgPool::new(&SETTINGS).await.expect("pool creation to work");
    pg.drop_and_create_public_schema()
        .await
        .expect("delete the schema");
    pg.migrate().await.expect("migrations to succeed");
    let conn = pg.pool.get().await.expect("cannot connect to pg");
    let row = conn
        .query_one("SELECT result FROM next_si_id_v1()", &[])
        .await
        .expect("getting an si_id has failed");
    let result: i64 = row.try_get("result").expect("cannot get the result");
    assert!(result > 0);
}
