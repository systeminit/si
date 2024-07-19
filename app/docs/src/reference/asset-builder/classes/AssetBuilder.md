[**lang-js**](../README.md) • **Docs**

***

[lang-js](../README.md) / AssetBuilder

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

[asset\_builder.ts:1061](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1061)

## Properties

### asset

> **asset**: [`Asset`](../interfaces/Asset.md)

#### Defined in

[asset\_builder.ts:1059](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1059)

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

[asset\_builder.ts:1071](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1071)

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

[asset\_builder.ts:1085](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1085)

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

[asset\_builder.ts:1123](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1123)

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

[asset\_builder.ts:1164](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1164)

***

### addInputSocket()

> **addInputSocket**(`socket`): [`AssetBuilder`](AssetBuilder.md)

Adds an input socket to the asset.

#### Parameters

• **socket**: [`SocketDefinition`](../interfaces/SocketDefinition.md)

The socket definition to add

#### Returns

[`AssetBuilder`](AssetBuilder.md)

This AssetBuilder instance for method chaining

#### Implementation of

[`IAssetBuilder`](../interfaces/IAssetBuilder.md).[`addInputSocket`](../interfaces/IAssetBuilder.md#addinputsocket)

#### Defined in

[asset\_builder.ts:1178](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1178)

***

### addOutputSocket()

> **addOutputSocket**(`socket`): [`AssetBuilder`](AssetBuilder.md)

Adds an output socket to the asset.

#### Parameters

• **socket**: [`SocketDefinition`](../interfaces/SocketDefinition.md)

The socket definition to add

#### Returns

[`AssetBuilder`](AssetBuilder.md)

This AssetBuilder instance for method chaining

#### Implementation of

[`IAssetBuilder`](../interfaces/IAssetBuilder.md).[`addOutputSocket`](../interfaces/IAssetBuilder.md#addoutputsocket)

#### Defined in

[asset\_builder.ts:1192](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1192)

***

### addSiPropValueFrom()

> **addSiPropValueFrom**(`siPropValueFrom`): `this`

#### Parameters

• **siPropValueFrom**: [`SiPropValueFromDefinition`](../interfaces/SiPropValueFromDefinition.md)

#### Returns

`this`

#### Implementation of

[`IAssetBuilder`](../interfaces/IAssetBuilder.md).[`addSiPropValueFrom`](../interfaces/IAssetBuilder.md#addsipropvaluefrom)

#### Defined in

[asset\_builder.ts:1200](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1200)

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

[asset\_builder.ts:1215](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1215)

***

### build()

> **build**(): [`Asset`](../interfaces/Asset.md)

#### Returns

[`Asset`](../interfaces/Asset.md)

#### Implementation of

[`IAssetBuilder`](../interfaces/IAssetBuilder.md).[`build`](../interfaces/IAssetBuilder.md#build)

#### Defined in

[asset\_builder.ts:1223](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1223)
