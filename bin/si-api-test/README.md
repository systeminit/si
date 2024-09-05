# SI API Tester

## Running

Run all tests (with auth api running locally):

```shell
SDF_API_URL="http://localhost:8080" AUTH_API_URL="http://localhost:9001" deno task run $WORKSPACE_ID $EMAIL $PASSWORD
```

Alternately, you can skip the password argument, pass in a userId in place of the email and set a jwt private key,
such as [dev.jwt_signing_private_key.pem](../../config/keys/dev.jwt_signing_private_key.pem) in our config/keys folder,
to the JWT_PRIVATE_KEY env variable. This is good for local development, but not how we'll do it in GitHub actions.

## Adding new tests

