[**TypeScript Function API**](../../README.md) • **Docs**

***

[TypeScript Function API](../../README.md) / [asset\_builder](../README.md) / SecretPropBuilder

# Class: SecretPropBuilder

Creates a prop [and a socket] in an asset with which to connect a secret

## Example

```ts
const secretPropName = new SecretPropBuilder()
  .setName("credential")
  .setSecretKind("DigitalOcean Credential")
 .build();
```

## Implements

- [`ISecretPropBuilder`](../interfaces/ISecretPropBuilder.md)

## Constructors

### new SecretPropBuilder()

> **new SecretPropBuilder**(): [`SecretPropBuilder`](SecretPropBuilder.md)

#### Returns

[`SecretPropBuilder`](SecretPropBuilder.md)

#### Defined in

[asset\_builder.ts:822](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L822)

## Properties

### prop

> **prop**: [`SecretPropDefinition`](../interfaces/SecretPropDefinition.md)

#### Defined in

[asset\_builder.ts:820](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L820)

## Methods

### setName()

> **setName**(`name`): `this`

The secret prop name. This will appear in the model UI and can be any value

#### Parameters

• **name**: `string`

the name of the secret prop

#### Returns

`this`

this

#### Example

```ts
.setName("token")
```

#### Implementation of

[`ISecretPropBuilder`](../interfaces/ISecretPropBuilder.md).[`setName`](../interfaces/ISecretPropBuilder.md#setname)

#### Defined in

[asset\_builder.ts:842](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L842)

***

### setSecretKind()

> **setSecretKind**(`kind`): `this`

The type of the secret - relates to the Secret Definition Name

#### Parameters

• **kind**: `string`

{string}

#### Returns

`this`

this

#### Example

```ts
.setSecretKind("DigitalOcean Credential")
```

#### Implementation of

[`ISecretPropBuilder`](../interfaces/ISecretPropBuilder.md).[`setSecretKind`](../interfaces/ISecretPropBuilder.md#setsecretkind)

#### Defined in

[asset\_builder.ts:857](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L857)

***

### setConnectionAnnotation()

> **setConnectionAnnotation**(`annotation`): `this`

#### Parameters

• **annotation**: `string`

#### Returns

`this`

#### Implementation of

[`ISecretPropBuilder`](../interfaces/ISecretPropBuilder.md).[`setConnectionAnnotation`](../interfaces/ISecretPropBuilder.md#setconnectionannotation)

#### Defined in

[asset\_builder.ts:862](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L862)

***

### setDocLinkRef()

> **setDocLinkRef**(`ref`): `this`

#### Parameters

• **ref**: `string`

#### Returns

`this`

#### Implementation of

[`ISecretPropBuilder`](../interfaces/ISecretPropBuilder.md).[`setDocLinkRef`](../interfaces/ISecretPropBuilder.md#setdoclinkref)

#### Defined in

[asset\_builder.ts:867](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L867)

***

### setDocLink()

> **setDocLink**(`link`): `this`

#### Parameters

• **link**: `string`

#### Returns

`this`

#### Implementation of

[`ISecretPropBuilder`](../interfaces/ISecretPropBuilder.md).[`setDocLink`](../interfaces/ISecretPropBuilder.md#setdoclink)

#### Defined in

[asset\_builder.ts:872](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L872)

***

### skipInputSocket()

> **skipInputSocket**(): `this`

Whether the prop should disable the auto-creation of an input socket

#### Returns

`this`

this

#### Example

```ts
.skipInputSocket()
```

#### Implementation of

[`ISecretPropBuilder`](../interfaces/ISecretPropBuilder.md).[`skipInputSocket`](../interfaces/ISecretPropBuilder.md#skipinputsocket)

#### Defined in

[asset\_builder.ts:885](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L885)

***

### build()

> **build**(): [`SecretPropDefinition`](../interfaces/SecretPropDefinition.md)

#### Returns

[`SecretPropDefinition`](../interfaces/SecretPropDefinition.md)

#### Implementation of

[`ISecretPropBuilder`](../interfaces/ISecretPropBuilder.md).[`build`](../interfaces/ISecretPropBuilder.md#build)

#### Defined in

[asset\_builder.ts:890](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L890)
