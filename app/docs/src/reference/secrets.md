# Secrets

Secrets in System Initiative represent any confidential data, such as login information, API tokens, passwords, etc.

They are end-to-end encrypted, both within the application, on the wire, and at rest.

## Encryption Strategy

System Initiative is a multi-tenant system, starting with an 'installation' of System Initiative. For most people, this means our SaaS platform - but it can also mean a local development instance, a BYOC instance, or a custom installation. Each installation of System Initiative generates an [Ed25519 Public Key Pair](https://en.wikipedia.org/wiki/EdDSA) for the installation. This is the *cluster key*.

Each [workspace](./workspaces.md) generates an [Ed25519 Public Key Pair](https://en.wikipedia.org/wiki/EdDSA) specific to it. The private side of the key is encrypted with the *cluster key*, which ensures that is encrypted at rest (even if the database is compromised and dumped in plain text). It cannot be retrieved without the corresponding *cluster key*. This is the *workspace key*.

When a secret is put into System Initiative, it is first encrypted with the *public workspace key* for the workspace it is destined for. This ensures that *your secret* is also encrypted at rest, and cannot be retrieved without the corresponding *cluster key* and *workspace key*.

When one of your secrets is needed, we decrypt it and inject it into the micro-vm assigned to your function.

```mermaid
flowchart TD
    Secret[Your Secret Value]
    WorkspacePubKey[Workspace Public Key]
    WorkspacePrivKey[Workspace Private Key]
    ClusterKey[Cluster Key]

    Secret -->|Encrypted with| WorkspacePubKey
    WorkspacePubKey --> EncryptedSecret[(Encrypted Secret)]

    WorkspacePrivKey -->|Encrypted with| ClusterKey

    EncryptedSecret -.->|Requires for decryption| WorkspacePrivKey

    subgraph AWS RDS Encryption
        WorkspacePubKey
        EncryptedSecret
        WorkspacePrivKey
    end
```

## Redaction

Should the secret appear in any logs or output, it will be automatically *redacted*. System Initiative will not leak your secrets.

:::warning
No programmable system is perfectly secure against secret ex-filtration. You must still trust the underlying function code that your secret is being used by.

All "built in" schemas provided by System Initiative have been vetted for ex-filtration risk.
:::

## Creating a Secret

<DocTabs tabs="CLI,Web Application,Public API">
<TabPanel value="CLI">
To create a Secret with the CLI:

```shellscript [Create a Secret]
$ si secret create "AWS Credential" --name "aws-credential-dev" --use-local-profile
✨ info    si          Creating change set: Create AWS Credential - 1765498265932
✨ info    si          Discovering credentials from local environment...
✨ info    si          ✓ Found 3 credential(s) in local environment
✨ info    si
✨ info    si          Creating AWS Credential component "aws-credential-dev"...
✨ info    si          ✓ Component created with ID: 01KC7YBC03EJH2TAM5K4XTX1GE
✨ info    si          Creating secret data...
✨ info    si          ✓ Secret created with ID: 01KC7YBEWJAC0K3Q7K1QQ22R17
✨ info    si          Attaching secret to component...
✨ info    si          ✓ Secret attached to component
✨ info    si
✨ info    si          ✓ Credential created successfully!
✨ info    si            Component ID: 01KC7YBC03EJH2TAM5K4XTX1GE
✨ info    si            Secret ID: 01KC7YBEWJAC0K3Q7K1QQ22R17
✨ info    si            Secret Name: aws-credential-dev
✨ info    si            Change Set ID: 01KC7YBATRTKY603YKW5WAZXQ0
```

By using `--use-local-profile` the CLI will read the credentials from your environment. You can also pass `--interactive` and it will prompt the user for the parameters. 

</TabPanel>
<TabPanel value="Web Application">

Creating a secret is identical to creating any other component.
Press the `C` hotkey or click the 'Add Component' button, from the Grid or Map views. Search for the name of your secret, press enter, and add the secret information to the components attributes.


![Creating a Secret](./secrets/creating-a-secret.png)

The information inside the *secret* section will be encrypted when the 'Add Secret' button is pressed.

</TabPanel>
<TabPanel value="Public API">

:::code-group
```typescript [TypeScript]
const response = await secretsApi.createSecret({
  workspaceId,
  changeSetId,
  createSecretV1Request: {
    name: 'my-aws-credentials',
    definitionName: 'AWS Credential',
    description: 'My AWS access credentials',
    rawData: {
      'accessKeyId': 'AKIAIOSFODNN7EXAMPLE',
      'secretAccessKey': 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'
    }
  }
});
```

```python [Python]
request = CreateSecretV1Request(
    name="my-aws-credentials",
    definition_name="AWS Credential",
    description="My AWS access credentials",
    raw_data={
        "accessKeyId": "AKIAIOSFODNN7EXAMPLE",
        "secretAccessKey": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    },
)

response = secrets_api.create_secret(
    workspace_id=workspace_id,
    change_set_id=change_set_id,
    create_secret_v1_request=request,
)
```
:::

Both examples use the [Public Create Secret API](./public-api#create-a-secret). See the [Public API](./public-api) documentation and the SDKs for more details on configuring the SDK to communicate with System Initiative.

:::warning
Unlike adding secrets through the Web Application, embedding secrets in your own programs using the Public API can expose leak those secrets. We recommend injecting the values through environment variables or other means, rather than hard-coding them in your API calls.
:::

</TabPanel>
</DocTabs>

:::info
Most things in System Initiative can be done from the Web Application, AI Agent, or the Public API. Creating secrets can only be done through the Web Application and the Public API, as we believe it is fundamentally insecure to feed secrets to an LLM.
:::

## Subscribe to a Secret

To use a secret, you subscribe to the secret value from the component that needs it.

<DocTabs tabs="AI Agent,Web Application,Public API">
<TabPanel value="AI Agent">

```prompt [Subscribe to Secret]
> Create a AWS VPC using the My AWS Credential secret
● VPC created successfully in the "create-vpc" change set with:
  - CIDR Block: 10.0.0.0/16
  - Region: us-west-1
  - Credential: My AWS Credential

  You can now apply this change set to create the VPC in AWS, or
  make additional changes first.
```

Reference the name of the specific credential you want to subscribe to in the prompt.

</TabPanel>
<TabPanel value="Web Application">

A component that requires a secret will have an entry in its *secrets* attributes for every kind of secret it needs. Select that entry, and you will be presented with a list of secrets that match its kind. Select the secret you want to use to subscribe to it.

![Subscribe to a Secret](./secrets/subscribe-to-secret.png)

</TabPanel>
<TabPanel value="Public API">

:::code-group
```typescript [TypeScript]
const componentResponse = await componentsApi.createComponent({
  workspaceId,
  changeSetId,
  createComponentV1Request: {
    name: 'test-ec2-instance',
    schemaName: 'AWS::EC2::Instance',
    attributes: {
      "/secrets/AWS Credential": {
        component: secretId,
        path: "/secrets/AWS Credential",
      },
    }
  }
});
```

```python [Python]
component_request = CreateComponentV1Request(
    name="test-ec2-instance",
    schema_name="AWS::EC2::Instance",
    attributes={
        "/secrets/AWS Credential": {
            "component": secret_id,
            "path": "/secrets/AWS Credential",
        },
    },
)

component_response = components_api.create_component(
    workspace_id=workspace_id,
    change_set_id=change_set_id,
    create_component_v1_request=component_request,
)
```
:::

</TabPanel>
</DocTabs>

## Updating a Secret


<DocTabs tabs="CLI,Web Application,Public API">
<TabPanel value="CLI">
To update a Secret with the CLI:

```shellscript [Create a Secret]
$ si secret update "aws-credential-dev" --use-local-profile
✨ info    si          Creating change set: Update secret - 1765498777287
✨ info    si          Looking for component: aws-credential-dev
✨ info    si          Found component: aws-credential-dev (01KC7YRCH5XDZ66J848MFY4PKA)
✨ info    si          Found secretId in component: 01KC7YRD7W8GAMXTC6SZPNTK1B
✨ info    si          Found secret: aws-credential-dev (01KC7YRD7W8GAMXTC6SZPNTK1B)
✨ info    si          Discovering credentials from local environment...
✨ info    si          ✓ Found 3 credential(s) in local environment
✨ info    si
✨ info    si          Updating secret "aws-credential-dev"...
✨ info    si          ✓ Secret updated: 01KC7YRD7W8GAMXTC6SZPNTK1B
✨ info    si
✨ info    si          ✓ Secret updated successfully!
✨ info    si            Secret ID: 01KC7YRD7W8GAMXTC6SZPNTK1B
✨ info    si            Secret Name: aws-credential-dev
✨ info    si            Change Set ID: 01KC7YTW94K3BKRD91X7PRJ6PN
```

By using `--use-local-profile` the CLI will read the credentials from your environment. You can also pass `--interactive` and it will prompt the user for the parameters. You can change the name of the secret by passing `--name` and / or the description by passing `--description`.

</TabPanel>
<TabPanel value="Web Application">
To update a secret, replace the secret data with the new information.

First navigate to the component, and click on the secret you want to update:

![Update Secret by selecting it](./secrets/update-secret-first-click.png)

Then update the secret information:

![Creating a Secret](./secrets/creating-a-secret.png)

:::tip
Note that existing secret data is not present in the form! Once a secret has been encrypted and stored, it cannot be retrieved by anything other than a function execution.
:::

</TabPanel>
<TabPanel value="Public API">

To update a secret with the public API, use the [Update Secret API](./public-api.md#update-a-secret).

:::code-group
```typescript [TypeScript]
const updateResponse = await secretsApi.updateSecret({
  workspaceId,
  changeSetId,
  secretId,
  updateSecretV1Request: {
    id: secretId,
    name: 'updated-aws-credentials',
    description: 'Updated credentials with new keys',
    rawData: {
      'accessKeyId': 'AKIAIOSFODNN7NEWKEY',
      'secretAccessKey': 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYNEWKEY'
    }
  }
});
```

```python [Python]
update_request = UpdateSecretV1Request(
    id=secret_id,
    name="updated-aws-credentials",
    description="Updated credentials with new keys",
    raw_data={
        "accessKeyId": "AKIAIOSFODNN7NEWKEY",
        "secretAccessKey": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYNEWKEY",
    },
)

update_response = secrets_api.update_secret(
    workspace_id=workspace_id,
    change_set_id=change_set_id,
    secret_id=secret_id,
    update_secret_v1_request=update_request,
)
```
:::

:::warning
Unlike updating secrets through the Web Application, embedding secrets in your own programs using the Public API can expose leak those secrets. We recommend injecting the values through environment variables or other means, rather than hard-coding them in your API calls.
:::

</TabPanel>
</DocTabs>

## Delete a Secret

You can delete (or erase) secrets like like [components](./secrets/creating-a-secret.png).

:::tip
When a secret is deleted, it cannot be recovered, even from within a change set.
:::

<DocTabs tabs="AI Agent,Web Application,Public API">
<TabPanel value="AI Agent">

Reference the name of the specific secret you want to delete or erase in the prompt.

```prompt [Delete Secret]
> Delete the My AWS Credential secret
● The "My AWS Credential" secret has been deleted
```

</TabPanel>
<TabPanel value="Web Application">

To delete a secret, either delete the component from the Grid screen by right clicking on the component and selecting 'Delete':

![Delete from Grid](./secrets/grid-delete.png)

Or the map, by right clicking the component and selecting 'Delete':

![Delete from Map](./secrets/map-delete.png)

Or from the Component screen:

![Delete from Component](./secrets/component-delete.png)

</TabPanel>
<TabPanel value="Public API">

To update a secret with the public API, use the [Delete Secret API](./public-api.md#delete-a-secret).

:::code-group
```typescript [TypeScript]
const deleteResponse = await secretsApi.deleteSecret({
  workspaceId,
  changeSetId,
  secretId
});
```

```python [Python]
delete_response = secrets_api.delete_secret(
    workspace_id=workspace_id,
    change_set_id=change_set_id,
    secret_id=secret_id,
)
```
:::

:::warning
Unlike updating secrets through the Web Application, embedding secrets in your own programs using the Public API can expose leak those secrets. We recommend injecting the values through environment variables or other means, rather than hard-coding them in your API calls.
:::

</TabPanel>
</DocTabs>

