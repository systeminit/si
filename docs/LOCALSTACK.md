# LocalStack

This document contains information related to using [LocalStack](https://github.com/localstack/localstack) when working on the System Initiative software.

## How to Use with the "AWS Credential" Builtin `SchemaVariant`

You can use the "AWS Credential" builtin `SchemaVariant` with LocalStack when running the System Initiative software with the following command:

```shell
buck2 run //dev:up
```

To use LocalStack with "AWS Credential", create a `Component` using the `SchemaVariant`.
After that, create a `Secret` and use it in the property editor.
The secret should have `http://0.0.0.0:4566` populated in the "Endpoint" field.

Now, you can use LocalStack in your development setup.
