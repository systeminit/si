# Secrets

Adopting the use of secrets in applications and infrastructure can be daunting.
Secret usage and integration is couched in cautionary yellow tape or unexpected
new application or infrastructure requirements.

Are my secrets secure? Are there safe and secure defaults? Will I accidentally
leak sensitive data in logs?

Each service may have a different way of accepting these secrets, either as a

- Environment variable
- Bearer token in an API call
- Password/connection string for a database
- SSH Key for connectivity

System Initiative secrets are secure by default. They are encrypted in the
browser before being transmitted over the wire and encrypted at rest. All
generated logs will automatically redact the secret so there are no leaks.
Integrating them into your workflow or with 3rd party systems is as simple as
writing a function.

To illustrate this in action, here's Fletcher, a Principal Engineer at System
Initiative, giving a video walkthrough of the functionality.

### Youtube Video

# It's a secret

Fletcher's video provided an overview of how a secret is composed and how it can
be consumed by an asset in System Initiative. Here's a breakdown

A new secret is defined like any other asset. The key difference is using the
`SecretDefinitionBuilder` as opposed to the normal `PropBuilder` and to use the
`defineSecret` method in the `AssetBuilder`.

Take special care to note that secrets in System Initiative can have as many
fields as needed. The name of the property is how you distinguish between the
values of the secret when they're consumed.

```typescript
function main() {
  const secretDefinition = new SecretDefinitionBuilder()
    .setName("GitHub Token")
    .addProp(
      new PropBuilder()
        .setName("token")
        .setKind("string")
        .setWidget(
          new PropWidgetDefinitionBuilder().setKind("password").build(),
        )
        .build(),
    )
    .build();

  return new AssetBuilder().defineSecret(secretDefinition).build();
}
```

Next, an Authentication Function is attached to the newly created asset by
creating a new function.

Authentication functions are always executed before other functions in the call
order and, as a result, are used to create the context in which other functions
are executed in. What's important to note here is that if you wanted to connect
to other secret management systems from System Initiative, this is where it
would be done.

In this example, the `gh` binary GitHub provides expects an environment variable
`GH_TOKEN` to be set to execute authenticated actions against the GitHub
service.

```typescript
async function main(secret: Input): Promise<Output> {
  requestStorage.setEnv("GH_TOKEN", secret.token);
}
```

Remember before when you took note of the name of the property being `token`?
The value is provided as a property of the input of the function.

Now that a secret has been defined, it's time to use it by creating an asset
that can accept the secret.

```typescript
function main() {
  const fullNameProp = new PropBuilder()
    .setName("FullName")
    .setKind("string")
    .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
    .setValueFrom(
      new ValueFromBuilder()
        .setKind("prop")
        .setPropPath(["root", "si", "name"])
        .build(),
    )
    .build();

  const tokenSecretProp = new SecretPropBuilder()
    .setName("token")
    .setSecretKind("GitHub Token")
    .build();

  return new AssetBuilder()
    .addProp(fullNameProp)
    .addSecretProp(tokenSecretProp)
    .build();
}
```

In this example, a new asset is defined, this time using the
`SecretPropBuilder`. Taking care to use the same name `token` so it all works
together correctly.

At this point, there is the definition of the secret, an asset that expects the
secret, and what's left is to use the secret.

Here's a qualification function that uses the `gh` cli tool to see if a
repository exists.

```typescript
async function main(component: Input): Promise<Output> {
  if (!component.domain?.FullName) {
    return {
      result: "failure",
      message: "no full name available",
    };
  }

  const child = await siExec.waitUntilEnd("gh", [
    "repo",
    "view",
    component.domain.FullName,
    "--json",
    "id",
  ]);
  return {
    result: child.exitCode === 0 ? "success" : "failure",
    message: child.exitCode === 0 ? child.stdout : child.stderr,
  };
}
```

Take special note that no specific secret logic needs to take place here. That
is because the code is executed in the context of the Authentication Function
that was defined as part of the definition of the secret.
