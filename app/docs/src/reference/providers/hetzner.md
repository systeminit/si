---
outline:
  level: [1, 2, 3, 4]
---

# Hetzner Cloud Support in System Initiative

Updated November 17, 2025.

System Initiative supports Hetzner Cloud resources through their
[OpenAPI Specification](https://docs.hetzner.cloud/cloud.spec.json). This allows
us to automatically generate and maintain accurate coverage of available
services and resources.

We support authentication through:

- [API Tokens](https://docs.hetzner.cloud/reference/cloud#authentication)

If this does not cover your use cases, please
[contact us](https://calendly.com/d/cw8r-6rq-b3n/share-your-use-case-with-system-initiative)
so we can extend support as needed.

## Connecting System Initiative to your Hetzner Cloud Account

In order to use Hetzner Cloud from within System Initiative, you need to use a
`Hetzner::Credential::ApiToken` component. Creating that component will then
prompt you for an API Token that you can use to create or discover your existing
infrastructure.
