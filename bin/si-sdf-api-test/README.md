# SI SDF API Tester

## Running

Run all tests (with auth api running locally):

```shell
export SDF_API_URL="http://localhost:8080"
export AUTH_API_URL="http://localhost:9001"

export BEARER_TOKEN="<your-bearer-token>"
export WORKSPACE_ID="<your-workspace-id>"
export CHANGE_SET_ID="<your-change-set-id>"

deno task run \
  -w $WORKSPACE_ID \ # required
  -c $CHANGE_SET_ID \ # optional depending on the test
  -k $BEARER_TOKEN \ # can replace with user and password
  -t 8-check_mjolnir # comma separated list (runs all if nothing is provied)
```

> [!TIP]
> Run `deno task run --help` for help options.

Alternately, you can skip the password argument, pass in a userId in place of
the email and set a jwt private key, such as
[dev.jwt_signing_private_key.pem](../../config/keys/dev.jwt_signing_private_key.pem)
in our config/keys folder, to the JWT_PRIVATE_KEY env variable. This is good for
local development, but not how we'll do it in GitHub actions.

> [!TIP]
> You can pass in a profile as well to customize test execution setup.
>
> ```shell
> --profile '{"maxDuration": "5", "rate": "1", "useJitter": false}'
> ```

## Adding new tests

Add a new file into `./tests/<something>.ts` and then invoke it using the --tests
param in the binary execution

## Benchmarking

To run a benchmark test, specify `--tests /benchmark/{testName}` when invoking
Deno. We only allow you to run a single benchmark test at a time right now.
