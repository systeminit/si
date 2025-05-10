# SI SDF API Tester

## Running

Run all tests (with auth api running locally):

```shell
export SDF_API_URL="http://localhost:8080"
export AUTH_API_URL="http://localhost:9001"

deno task run --workspace-id $WORKSPACE_ID \
              --userId $EMAIL \
              --password $PASSWORD \
              --profile '{"maxDuration": "5", "rate": "1", "useJitter": false}'
              --tests create_and_use_variant,get_head_changeset

Usage: deno run main.ts [options]

Options:
  --workspaceId, -w   Workspace ID (required)
  --userId, -u        User ID (optional, if token is provided)
  --password, -p      User password (optional, if token is provided)
  --token, -t         User token (optional, if userId and password are provided)
  --tests, -t         Test names to run (comma-separated, optional)
  --profile, -l       Test profile in JSON format (optional)
  --help              Show this help message
```

Alternately, you can skip the password argument, pass in a userId in place of
the email and set a jwt private key, such as
[dev.jwt_signing_private_key.pem](../../config/keys/dev.jwt_signing_private_key.pem)
in our config/keys folder, to the JWT_PRIVATE_KEY env variable. This is good for
local development, but not how we'll do it in GitHub actions.

## Adding new tests

Add a new file into ./tests/<something>.ts and then invoke it using the --tests
param in the binary execution

## Benchmarking

To run a benchmark test, specify `--tests /benchmark/{testName}` when invoking
Deno. We only allow you to run a single benchmark test at a time right now.
