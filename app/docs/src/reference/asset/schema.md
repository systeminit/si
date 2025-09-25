# Asset Schema Reference

Asset Schema are defined as TypeScript functions that use the [AssetBuilder
API](/reference/typescript/asset_builder/README). This document augments the
API documentation by laying out practical examples of each option, and briefly
explaining what they do.

## Schema Basics

Asset Schema define:

* Component properties
* Resource properties
* Secrets the asset requires
* Secrets the asset defines

## The Builder Pattern

The Builder Pattern allows you to create an object, configure it, and
eventually call `build()` on it to finalize and export the data. For example:

```typescript
const asset = new AssetBuilder();

const keyName = new PropBuilder()
    .setName("KeyName")
    .setKind("string")
    .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
    .build();
asset.addProp(keyName);

return asset.build();
```

There are 3 `Builders` in this snippet - the `AssetBuilder`, `PropBuilder`, and
`PropWidgetDefinitionBuilder`. The pattern is always the same:

1. Create the Builder.
2. Configure the object.
3. Call `build()`.

## The asset object

The [AssetBuilder
API](/reference/typescript/asset_builder/classes/AssetBuilder) is used to
define the schema for an asset, such as a component. Schema definitions always
begin by instantiating a new `AssetBuilder`, and end by returning the value of
`asset.build()`.

```typescript
const asset = new AssetBuilder();

return asset.build();
```

## Component properties

Properties are added to an asset with the [PropBuilder
API](/reference/typescript/asset_builder/classes/PropBuilder) and the
`addProp()` function of the `AssetBuilder`.

Properties map to the underlying data structure of an asset, and specify things
like field validations and influence the attribute panel's UI.

There are 6 kinds of properties, corresponding roughly to the standard
JavaScript types:

* array
* boolean
* integer
* map
* object
* string

:::info
The difference between a `map` and an `object` is that maps take arbitrary key/value
pairs, while `objects` have defined properties.
:::

All properties have:

* A Kind, which defines its fundamental data type, specified by `setKind()`.
* A Name, specified by `setName()`.
* An optional Validation Format, specified by `setValidationFormat()` which
uses [Joi to specify valid values](https://joi.dev/api/?v=17.13.3).
* An optional Default Value, specified by `setDefaultValue()`.
* An optional boolean field that hides the property from the attributes UI,
specified with `setHidden()`.
* An optional Widget configuration, that determines how the field is presented
in the attributes panel, specified with `setWidget()`.

### Boolean properties

A boolean property named `IsPublic` with a default value of `false`.

```typescript
const isPublicProp = new PropBuilder()
    .setName("IsPublic")
    .setKind("boolean")
    .setDefaultValue(false)
    .build();
```

### String properties

This example specifies a property named `KeyName`, whose value is a string:

```typescript
const keyNameProp = new PropBuilder()
  .setKind("string")
  .setName("KeyName")
  .build();
asset.addProp(keyNameProp);
```

A string with options, that displays as a select box, with a default value:

```typescript
const keyType = new PropBuilder()
    .setName("KeyType")
    .setKind("string")
    .setWidget(new PropWidgetDefinitionBuilder()
        .setKind("select")
        .addOption("rsa", "rsa")
        .addOption("ed25519", "ed25519")
        .build())
    .setDefaultValue("rsa")
    .build();
asset.addProp(keyType);
```

A string with a complex regular expression validation:

```typescript
const cidrBlockProp = new PropBuilder()
    .setKind("string")
    .setName("CidrBlock")
    .setValidationFormat(
        Joi
           .string()
           .regex(/^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])(\/)(1[6-9]|2[0-8])$/)
           .messages({
               'string.pattern.base': 'Must be a valid IPv4 CIDR with CIDR Blocks between /16 and /28'
           })
    )
    .build();
asset.addProp(cidrBlockProp);
```

### Integer properties

A Throughput integer property:

```typescript
const throughputProp = new PropBuilder()
    .setName("Throughput")
    .setKind("integer")
    .build()
asset.addProp(throughputProp);
```

### Map properties

A map named `ResourceTags`:

```typescript
const resourceTags = new PropBuilder()
    .setKind("map")
    .setName("ResourceTags")
    .setEntry(new PropBuilder()
        .setKind("string")
        .setName("tag")
    )
    .build();
asset.addProp(resourceTags);
```

Maps use the `setEntry` API to define the property kind of their value. Their
keys are always `string`. The name of the Entry is displayed in the attribute
panel, but is not present in the resulting data structure.

So this map would produce a data structure like:

```typescript
{
  "van halen": "great band",
  "beyonce": "also great"
}
```

Any property kind is a valid map entry.

A more complex map of objects:

```typescript
const resourceTags = new PropBuilder()
    .setKind("map")
    .setName("ResourceTags")
    .setEntry(new PropBuilder()
        .setName("Francis")
        .setKind("object")
        .addChild(new PropBuilder()
            .setName("Bacon")
            .setKind("boolean")
            .build()
        )
    )
    .build();
asset.addProp(resourceTags);
```

Would produce:

```typescript
{
  "van halen": { "bacon": false },
  "beyonce": { "bacon": true },
}
```

### Object properties

An `AdvancedConfiguration` object, with 3 properties: `InstanceProfileArn`, `EbsOptimized`, and `UserData`:

```typescript
const advancedConfigurationProp = new PropBuilder()
    .setName("AdvancedConfiguration")
    .setKind("object")
    .addChild(new PropBuilder()
        .setKind("string")
        .setName("InstanceProfileArn")
        .build())
    .addChild(new PropBuilder()
        .setKind("boolean")
        .setName("EbsOptimized")
        .build())
    .addChild(new PropBuilder()
        .setKind("string")
        .setName("UserData")
        .build())
    .build()
asset.addProp(advancedConfigurationProp);
```

Objects use the `addChild` interface to specify their properties. Any property
kind can be a child property of an object.

This would produce an object that looks like this:

```typescript
{
  InstanceProfileArn: "arn:...",
  EbsOptimized: false,
  UserData: "...",
}
```

### Array properties

A `SecurityGroups` array of strings:

```typescript
const securityGroupsProp = new PropBuilder()
     .setKind("array")
     .setName("SecurityGroups")
     .setEntry(new PropBuilder()
         .setKind("string")
         .setName("SecurityGroup")
         .build())
     .build();
asset.addProp(securityGroupsProp);
```

Array's use the `setEntry` API to define the kind of their members. Any kind of
property is valid as an array entry.

An array of objects:

```typescript
const tagSpecificationsProp = new PropBuilder()
    .setName("TagSpecifications")
    .setKind("array")
    .setEntry(
        new PropBuilder()
        .setName("TagSpecificationsChild")
        .setKind("object")
        .addChild(
            new PropBuilder()
            .setName("Key")
            .setKind("string")
            .build(),
        )
        .addChild(
            new PropBuilder()
            .setName("Value")
            .setKind("string")
            .build(),
        )
        .build(),
    )
    .build();
asset.addProp(tagSpecificationsProp);
```

### Widgets

Widgets define how properties are displayed. Each kind of property has a
default widget, but it can be useful to alter the display on occasion (like the
example above with KeyPair options.) Widgets are set with the
[PropWidgetDefinitionBuilder](/reference/typescript/asset_builder/classes/PropWidgetDefinitionBuilder),
and used through the `setWidget()` method on a `PropBuilder`.

Available widget types are:

* array
* checkbox
* color
* comboBox
* header
* map
* select
* text
* textArea
* codeEditor
* password

The `select` widget and the `comboBox` widget both accept a list of options, set with
the `setOption()` method on the builder.

## Resource Properties

Resource Properties are used to extract information from Resources, and store
them as hidden properties. They are generally used as sources for subscriptions
from other components.

They are defined the same way as component properties, but with the
`setHidden(true)` option set on the `PropBuilder`, and are added to the asset
with `addResourceProp()` rather than `addProp()`.

For example:

```typescript
const instanceIdProp = new PropBuilder()
    .setName("InstanceId")
    .setKind("string")
    .setHidden(true)
    .build();
asset.addResourceProp(instanceIdProp);
```

Would extract the `InstanceId` from the resource information if it exists, and
populate this property.

## Secrets the asset requires

When an asset requires a secret, it is specified with the [SecretPropBuilder
API](/reference/typescript/asset_builder/classes/SecretPropBuilder). It creates
a property that allows a secret of the given kind to be set.

They consist of:

* The name of the secret prop
* The secret kind, which corresponds to the name of its secret definition

Here is an example of an AWS Credential:

```typescript
const credentialProp = new SecretPropBuilder()
    .setName("credential")
    .setSecretKind("AWS Credential")
    .build();
asset.addSecretProp(credentialProp)
```

This would allow the asset to accept any `AWS Credential`.

## Secrets the asset defines

Assets that define secrets should *only* define secrets. They use the
[SecretDefinitionBuilder](/reference/typescript/asset_builder/classes/SecretDefinitionBuilder)
to define themselves, and are added to the asset with the `defineSecret()`
method. They consist of:

* A name for the credential
* Props that define the shape of the secret itself

For example, the Docker Hub Credential:

```typescript
function main() {
const credential = new SecretDefinitionBuilder()
    .setName("Docker Hub Credential")
    .addProp(
        new PropBuilder()
        .setName("Username")
        .setKind("string")
        .setWidget(
            new PropWidgetDefinitionBuilder()
            .setKind("password")
            .build()
        ).build())
    .addProp(
        new PropBuilder()
        .setName("Password")
        .setKind("string")
        .setWidget(
            new PropWidgetDefinitionBuilder()
            .setKind("password")
            .build()
        ).build())
    .build();
return new AssetBuilder()
    .defineSecret(credential)
    .build()
}
```

Would allow users to add a Docker Hub Credential secret type, with two values,
Username and Password.

## Tips for Schema Creation

1. Resist the temptation to abstract the resource you are modeling. Assets in System Initiative work best when they are as close to 1:1 models of the upstream.
2. Frequently the right set of properties for a component mirror what they need when they are created.

## Using the si-generator to generate schema for AWS Services

The [System Initiative source code](https://github.com/systeminit/si)
repository contains a program that will automatically generate schema for AWS
services. Check out the repository, and navigate to the `bin/si-generator` directory.

Ensure you have the [aws cli](https://aws.amazon.com/cli/) installed.

Start by finding the `create` API for the asset you wish to model. For example, if you were modeling the AWS EC2 Key Pair, the command would be `aws ec2 create-key-pair`.

Then run the generator:

```shell
$ deno run ./main.ts asset ec2 create-key-pair
```

The resulting schema definition will correspond to the CLI skeleton of the AWS CLI:

```typescript
function main() {
  const asset = new AssetBuilder();

  const keyNameProp = new PropBuilder()
    .setName("KeyName")
    .setKind("string")
    .build();
  asset.addProp(keyNameProp);

  const dryRunProp = new PropBuilder()
    .setName("DryRun")
    .setKind("boolean")
    .build();
  asset.addProp(dryRunProp);

  const keyTypeProp = new PropBuilder()
    .setName("KeyType")
    .setKind("string")
    .build();
  asset.addProp(keyTypeProp);

  const tagSpecificationsProp = new PropBuilder()
    .setName("TagSpecifications")
    .setKind("array")
    .setEntry(
      new PropBuilder()
        .setName("TagSpecificationsChild")
        .setKind("object")
        .addChild(
          new PropBuilder()
            .setName("ResourceType")
            .setKind("string")
            .build(),
        )
        .addChild(
          new PropBuilder()
            .setName("Tags")
            .setKind("array")
            .setEntry(
              new PropBuilder()
                .setName("TagsChild")
                .setKind("object")
                .addChild(
                  new PropBuilder()
                    .setName("Key")
                    .setKind("string")
                    .build(),
                )
                .addChild(
                  new PropBuilder()
                    .setName("Value")
                    .setKind("string")
                    .build(),
                )
                .build(),
            )
            .build(),
        )
        .build(),
    )
    .build();
  asset.addProp(tagSpecificationsProp);

  const keyFormatProp = new PropBuilder()
    .setName("KeyFormat")
    .setKind("string")
    .build();
  asset.addProp(keyFormatProp);

  const credentialProp = new SecretPropBuilder()
    .setName("credential")
    .setSecretKind("AWS Credential")
    .build();
  asset.addSecretProp(credentialProp);

  // Add any props needed for information that isn't
  // strictly part of the object domain here.
  const extraProp = new PropBuilder()
    .setKind("object")
    .setName("extra")
    .addChild(
      new PropBuilder()
        .setKind("string")
        .setName("Region")
        .build(),
    )
    .build();

  asset.addProp(extraProp);

  return asset.build();
}
```
