---
outline:
  level: [1, 2, 3, 4]
---

# Azure Support in System Initiative

Updated November 17, 2025.

System Initiative supports Azure resources through the
[Azure Resource Manager (ARM) REST API Specification](https://github.com/Azure/azure-rest-api-specs).
We automatically generate coverage using the latest stable API versions. If a
stable version is not available, we use the latest preview release to maintain
functionality and access to new features.

We support authentication through:

- [Service Principal with Client Secret](https://learn.microsoft.com/en-us/azure/active-directory/develop/howto-create-service-principal-portal)

If this does not cover your use cases, please
[contact us](https://calendly.com/d/cw8r-6rq-b3n/share-your-use-case-with-system-initiative)
so we can extend support as needed.

## Connecting System Initiative to your Azure Account

In order to use Azure from within System Initiative, you need to use a
`Microsoft Credential` component. Creating that component will then prompt you
for a `ClientId`, `ClientSecret` and a `TenantId` that you can use to create or
discover your existing infrastructure. We suggest the use of
`Microsoft.Resources/subscription` component in the workspace as a
subscriptionID is a key part of a resource ID in Azure.
