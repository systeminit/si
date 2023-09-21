//! This crate provides the ability to communicate with [DynamoDB](https://aws.amazon.com/dynamodb/).
//!
//! TODO(nick): put this somewhere --> https://docs.rs/aws-sdk-dynamodb/latest/aws_sdk_dynamodb/

#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

use aws_sdk_dynamodb::types::{
    AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
    TableClass,
};
use aws_sdk_dynamodb::Client as UpstreamClient;

use crate::error::DynamoResult;

mod error;

pub struct Client {
    inner: UpstreamClient,
}

impl Client {
    pub async fn new() -> DynamoResult<Self> {
        // NOTE(nick): keep this local for as long as possible.
        let config = aws_config::from_env()
            .endpoint_url("http://localhost:8000")
            .load()
            .await;
        let client = UpstreamClient::new(&config);
        Ok(Self { inner: client })
    }

    pub async fn delete_table(&self) -> DynamoResult<()> {
        let output = self
            .inner
            .delete_table()
            .table_name("ContentStore")
            .send()
            .await?;
        dbg!(output);
        Ok(())
    }

    pub async fn create_table(&self) -> DynamoResult<()> {
        let output = self
            .inner
            .create_table()
            .table_name("ContentStore")
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("Key")
                    .attribute_type(ScalarAttributeType::S)
                    .build(),
            )
            .attribute_definitions(
                AttributeDefinition::builder()
                    .attribute_name("Value")
                    .attribute_type(ScalarAttributeType::S)
                    .build(),
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("Key")
                    .key_type(KeyType::Hash)
                    .build(),
            )
            .key_schema(
                KeySchemaElement::builder()
                    .attribute_name("Value")
                    .key_type(KeyType::Range)
                    .build(),
            )
            .provisioned_throughput(
                ProvisionedThroughput::builder()
                    .read_capacity_units(5)
                    .write_capacity_units(5)
                    .build(),
            )
            .table_class(TableClass::Standard)
            .send()
            .await?;
        dbg!(output);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio::test;

    use super::*;

    #[test]
    async fn new_client() {
        let _client = Client::new().await.expect("could not create client");
    }

    #[test]
    async fn recreate_table() {
        let client = Client::new().await.expect("could not create client");
        if let Err(e) = client.delete_table().await {
            dbg!(e);
        }
        client.create_table().await.expect("could not create table");
    }
}
