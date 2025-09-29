# Luminork

`luminork` is an API server for automating SI and for making requests to its internal API directly.
It is the API server that SI's [MCP Server](../si-mcp-server) communicates with.

## Running Locally

Start by [running the local stack](../../DEV_DOCS.md) with the following environment variable set:

```shell
export SI_BASE_URL="http://localhost:5380"
```

If testing with the local MCP server, ensure that it is also using the same base URL.

## Generated API Documentation

Our [Swagger UI](https://api.systeminit.com/swagger-ui) contains the generated API documentation for `luminork` as of its last deployment.
