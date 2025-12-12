# Authentication Functions

Authentication functions are a kind of [function](./function.md) that handles
credential validation and setup before other functions execute. They run
automatically when a function needs to use [secrets](./secrets.md), ensuring
that authentication happens securely, transparently, and consistently.

## How Authentication Functions Work

Authentication functions are defined on schemas that use secrets (configured
using the
[SecretDefinitionBuilder API](./typescript/asset_builder/classes/SecretDefinitionBuilder.md)).
When any other function needs to use a secret:

1. System Initiative identifies which secret is required
2. The authentication function for that secret runs first
3. The authentication function sets up credentials (primarily via local storage)
4. The original function executes with access to the authenticated session

This ensures that credentials are always properly configured before any API
calls or operations that require authentication.

## When Authentication Functions Run

Authentication functions execute automatically:

- Before any action function that uses a secret
- Before any attribute function that requires authentication
- Once per request, even if multiple functions use the same secret

You never need to manually invoke authentication functions - System Initiative
handles this automatically based on function dependencies.

## What Authentication Functions Do

Authentication functions typically:

- **Store session data**: Use the requestStorage API to pass authentication data
  between functions
- **Handle multiple authentication methods**: Support different authentication
  flows (API keys, assume role, OAuth, etc.)
- **Prepare configuration data**: Write credential data that command-line tools
  require (like Docker config or kubeconfig)

Authentication functions always return nothing.

## Authentication Function Arguments

Authentication functions receive a single `secret` argument that contains the
properties defined in the SecretDefinitionBuilder for that secret type.

For example, an AWS credential secret might have:

- `AccessKeyId`
- `SecretAccessKey`
- `SessionToken` (optional)
- `AssumeRole` (optional)

The authentication function receives these properties and uses them to set up
the authenticated session.

## The requestStorage API

Authentication functions typically make use of the requestStorage API, which
allows you to:

- Set environment variables with `setEnv`
- Get environment variables with `getEnv`
- Store a javascript object as an item by key with `setItem`
- Get items by their key with `getItem`
- Check for the existence of an environment key with `getEnvKey` or an item with
  `getKeys`

This API ensures that credentials are available to all functions in the same
request context while maintaining security isolation between different requests.

## Common Use Cases

Authentication functions are commonly used for:

- **Cloud provider credentials**: Setting up AWS, Azure, or GCP authentication
- **API tokens**: Configuring authorization headers for REST APIs
- **Container registries**: Authenticating with Docker Hub, ECR, or other
  registries
- **Role assumption**: Implementing complex authentication flows like AWS
  AssumeRole
- **Service account setup**: Configuring service account credentials for
  Kubernetes or other platforms
- **Multi-method authentication**: Supporting multiple ways to authenticate with
  the same service

## Security Considerations

Authentication functions handle sensitive data. Keep in mind:

- Secrets are encrypted at rest and in transit
- Authentication functions run in isolated sandbox environments
- Credentials set via requestStorage are only available within the same request
- Environment variables are scoped to the function execution context
- System Initiative never logs or returns secret values in function output

## See Also

For detailed examples and technical implementation details, see the
[Authentication Function Examples](/reference/function#authentication-function-examples)
section in the Functions Reference.
