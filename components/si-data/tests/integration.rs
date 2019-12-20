use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio;
use tracing::{self, debug};
use tracing_subscriber::{self, EnvFilter, FmtSubscriber};
use ureq;
use uuid::Uuid;

use std::env;

pub mod common;

use si_data::{
    data::{
        query_expression_option::Qe, OrderByDirection, Query, QueryComparison, QueryExpression,
        QueryExpressionOption,
    },
    error::{DataError, Result},
    Db, ListResult, Migrateable, Reference, Storable,
};
use si_settings::Settings;

use crate::common::model::{
    list_data::ListData, relationship_data::RelationshipData, test_data::TestData,
};

lazy_static! {
    pub static ref SETTINGS: Settings = {
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish();

        tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");
        env::set_var("RUN_ENV", "testing");
        Settings::new().expect("Failed to load settings")
    };
    pub static ref DB: Db = {
        // Delete the data on first use!
        if ureq::post("http://si:bugbear@localhost:8091/pools/default/buckets/si_integration/controller/doFlush").call().error() {
           panic!("Cannot flush si_integration bucket");
        };
        Db::new(&SETTINGS).expect("failed to connect to database cluster")
    };
}

#[test]
fn connect_to_cluster() {
    Db::new(&SETTINGS).expect("failed to connect to database cluster");
}

#[tokio::test]
async fn insert() {
    let item = TestData::new("1", "adam jacob");
    DB.insert(&item).await.expect("cannot insert item");
}

#[tokio::test]
async fn validate_and_insert_as_new() {
    let mut item = TestData::new("22", "adam jacob");
    DB.validate_and_insert_as_new(&mut item)
        .await
        .expect("cannot insert item - first test data");
    let id_parts: Vec<&str> = item.get_id().split(":").collect();
    assert_eq!(
        id_parts[0], "test_data",
        "type_name is not the first part of the id"
    );
    Uuid::parse_str(id_parts[1]).expect("ID was not turned into a guid");

    // We fail if the name of the data is "mr bean" - because, I mean... Mr Bean is always failing,
    // right? Only it's actually not failing, it's leading us to the thing we needed all along,
    // because... Mr Bean.
    //
    // You're welcome.
    let mut fail_item = TestData::new("23", "mr bean");
    let fail_result = DB.validate_and_insert_as_new(&mut fail_item).await;
    match fail_result {
        Err(DataError::ValidationError(s)) => assert_eq!(s, "no mr bean here"),
        Err(e) => panic!("validation returned an incorrect error: {}", e),
        Ok(_) => panic!("validation passed when it should have failed"),
    }

    // Checking referential integrity - passing
    let mut ref_item = RelationshipData::new("1", "adam jacob", &item.id);
    DB.validate_and_insert_as_new(&mut ref_item)
        .await
        .expect("cannot insert item - relationship data first");

    // Check referential integrity - failing
    let mut ref_fail_item = RelationshipData::new("2", "archie mcfee", "not today");
    let fail_result = DB.validate_and_insert_as_new(&mut ref_fail_item).await;
    match fail_result {
        Err(DataError::ReferentialIntegrity(_, _)) => true,
        Err(e) => panic!(
            "validation returned an incorrect error on ref integrity: {}",
            e
        ),
        Ok(_) => panic!("validation passed on bad referential integrity"),
    };

    // Check for natural key overlap
    let mut dup_nat_key_item = RelationshipData::new("3", "adam jacob", &item.id);
    match DB.validate_and_insert_as_new(&mut dup_nat_key_item).await {
        Err(DataError::NaturalKeyExists(_)) => true,
        Err(e) => panic!(
            "validation returned an incorrect error on natual key check: {}",
            e
        ),
        Ok(_) => panic!("validation passed on duplicate natural key"),
    };
}

#[tokio::test]
async fn remove() {
    let item = TestData::new("2", "ghostface");
    DB.insert(&item).await.expect("cannot insert item");
    DB.remove("2").await.expect("cannot remove item");
}

#[tokio::test]
async fn get() {
    let item = TestData::new("3", "ozzy");
    DB.insert(&item).await.expect("cannot insert item");
    let result: TestData = DB.get("3").await.expect("cannot get item");
    assert_eq!(result.id, "3");
    assert_eq!(result.name, "ozzy");
}

#[tokio::test]
async fn upsert() {
    let mut item = TestData::new("4", "floopydoopy");
    DB.upsert(&item)
        .await
        .expect("cannot upsert non-existent item");
    item.name = String::from("flooglehorn");
    DB.upsert(&item).await.expect("cannot upsert existing item");
    let result: TestData = DB.get("4").await.expect("cannot get item");
    assert_eq!(result.id, "4");
    assert_eq!(result.name, "flooglehorn");
}

#[tokio::test]
async fn list() {
    let items = vec![
        ListData::new("5", "king diamond"),
        ListData::new("6", "slayer"),
        ListData::new("7", "ghost"),
        ListData::new("8", "agalloch"),
        ListData::new("9", "skid row"),
    ];

    for x in items.iter() {
        DB.insert(x).await.expect("cannot insert items");
    }

    let result: ListResult<ListData> = DB
        .list(&None, 2, "name", OrderByDirection::Asc, "")
        .await
        .expect("Cannot list items");

    assert_eq!(result.len(), 2, "results should match requested count");
    assert_eq!(
        result.total_count(),
        5,
        "results should have the correct total count"
    );
    assert_eq!(result[0].name, "agalloch", "results include first entry");
    assert_eq!(result[1].name, "ghost", "results include second entry");

    // Paging
    let next_result: ListResult<ListData> = DB
        .list_by_page_token(&result.page_token)
        .await
        .expect("Cannot fetch by page token");

    assert_eq!(
        next_result.len(),
        2,
        "next results should match requested count"
    );
    assert_eq!(
        next_result.total_count(),
        5,
        "next results should have the correct total count"
    );
    assert_eq!(
        next_result[0].name, "king diamond",
        "next results include first entry"
    );
    assert_eq!(
        next_result[1].name, "skid row",
        "next results include second entry"
    );

    let last_result: ListResult<ListData> = DB
        .list_by_page_token(&next_result.page_token)
        .await
        .expect("Cannot fetch by page token");
    assert_eq!(
        last_result.len(),
        1,
        "last results should match requested count"
    );
    assert_eq!(
        last_result.total_count(),
        5,
        "last results should have the correct total count"
    );
    assert_eq!(
        last_result[0].name, "slayer",
        "last results include first entry"
    );
    assert_eq!(last_result.page_token(), "");

    // With a Query
    let query = Some(Query {
        items: vec![QueryExpressionOption {
            qe: Some(Qe::Expression(QueryExpression {
                field: "name".to_string(),
                comparison: QueryComparison::Equals as i32,
                value: "slayer".to_string(),
                ..Default::default()
            })),
        }],
        ..Default::default()
    });

    let query_result: ListResult<ListData> = DB
        .list(&query, 2, "name", OrderByDirection::Asc, "")
        .await
        .expect("Cannot list queried items");
    assert_eq!(
        query_result.len(),
        1,
        "query results should match requested count"
    );
    assert_eq!(
        query_result.total_count(),
        1,
        "query results should have the correct total count"
    );
    assert_eq!(
        query_result[0].name, "slayer",
        "query results include the entry"
    );

    // Errors on bad order_by
    let bad_order_by: Result<ListResult<ListData>> = DB
        .list(&query, 2, "katatonia", OrderByDirection::Asc, "")
        .await;
    match bad_order_by {
        Err(DataError::InvalidOrderBy) => true,
        Err(e) => panic!("returned wrong error for bad order by field: {}", e),
        Ok(_) => panic!("returned results on bad order by field"),
    };
}
