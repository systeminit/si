# DynamoDB Local

This directory contains our [DynamoDB Local](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.html) image.
It should only be used for local development and testing.

## Configuration

We use the `-inMemory` [option](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.UsageNotes.html) to avoid mounting a volume to the local filesystem.

## Is it Up and Running?

Check if the container is up and running by executing the following command:

```shell
aws dynamodb list-tables --endpoint-url http://localhost:8000
```