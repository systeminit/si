[**TypeScript Function API**](../../README.md) • **Docs**

***

[TypeScript Function API](../../README.md) / [asset\_builder](../README.md) / AssetBuilder

# Class: AssetBuilder

Represents a builder for creating System Initiative Asset Schemas.

## Example

```ts
const asset = new AssetBuilder();

const myProp = new PropBuilder().setName("myProp").setKind("string").build();
asset.addProp(myProp);

return asset.build();
```

## Implements

- [`IAssetBuilder`](../interfaces/IAssetBuilder.md)

## Constructors

### new AssetBuilder()

> **new AssetBuilder**(): [`AssetBuilder`](AssetBuilder.md)

#### Returns

[`AssetBuilder`](AssetBuilder.md)

#### Defined in

[asset\_builder.ts:1056](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1056)

## Properties

### asset

> **asset**: [`Asset`](../interfaces/Asset.md)

#### Defined in

[asset\_builder.ts:1054](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1054)

## Methods

### addProp()

> **addProp**(`prop`): [`AssetBuilder`](AssetBuilder.md)

Adds a prop to the asset.

#### Parameters

• **prop**: [`PropDefinition`](../interfaces/PropDefinition.md)

The prop definition to add

#### Returns

[`AssetBuilder`](AssetBuilder.md)

This AssetBuilder instance for method chaining

#### Implementation of

[`IAssetBuilder`](../interfaces/IAssetBuilder.md).[`addProp`](../interfaces/IAssetBuilder.md#addprop)

#### Defined in

[asset\_builder.ts:1066](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1066)

***

### addSecretProp()

> **addSecretProp**(`prop`): [`AssetBuilder`](AssetBuilder.md)

Adds a secret prop to the asset.

#### Parameters

• **prop**: [`SecretPropDefinition`](../interfaces/SecretPropDefinition.md)

The secret prop definition to add

#### Returns

[`AssetBuilder`](AssetBuilder.md)

This AssetBuilder instance for method chaining

#### Implementation of

[`IAssetBuilder`](../interfaces/IAssetBuilder.md).[`addSecretProp`](../interfaces/IAssetBuilder.md#addsecretprop)

#### Defined in

[asset\_builder.ts:1080](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1080)

***

### defineSecret()

> **defineSecret**(`definition`): `this`

Adds a secret to the asset.

#### Parameters

• **definition**: [`SecretDefinition`](../interfaces/SecretDefinition.md)

The secret definition to add

#### Returns

`this`

This AssetBuilder instance for method chaining

#### Implementation of

[`IAssetBuilder`](../interfaces/IAssetBuilder.md).[`defineSecret`](../interfaces/IAssetBuilder.md#definesecret)

#### Defined in

[asset\_builder.ts:1118](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1118)

***

### addResourceProp()

> **addResourceProp**(`prop`): [`AssetBuilder`](AssetBuilder.md)

Adds a resource prop to the asset.

#### Parameters

• **prop**: [`PropDefinition`](../interfaces/PropDefinition.md)

The prop definition to add

#### Returns

[`AssetBuilder`](AssetBuilder.md)

This AssetBuilder instance for method chaining

#### Implementation of

[`IAssetBuilder`](../interfaces/IAssetBuilder.md).[`addResourceProp`](../interfaces/IAssetBuilder.md#addresourceprop)

#### Defined in

[asset\_builder.ts:1158](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1158)

***

### addDocLink()

> **addDocLink**(`key`, `value`): [`AssetBuilder`](AssetBuilder.md)

Adds a doc link to the asset.

#### Parameters

• **key**: `string`

the name of the doc link

• **value**: `string`

the value for the doc link

#### Returns

[`AssetBuilder`](AssetBuilder.md)

This AssetBuilder instance for method chaining

#### Implementation of

[`IAssetBuilder`](../interfaces/IAssetBuilder.md).[`addDocLink`](../interfaces/IAssetBuilder.md#adddoclink)

#### Defined in

[asset\_builder.ts:1209](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1209)

***

### build()

> **build**(): [`Asset`](../interfaces/Asset.md)

#### Returns

[`Asset`](../interfaces/Asset.md)

#### Implementation of

[`IAssetBuilder`](../interfaces/IAssetBuilder.md).[`build`](../interfaces/IAssetBuilder.md#build)

#### Defined in

[asset\_builder.ts:1217](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1217)
