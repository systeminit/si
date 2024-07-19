[**lang-js**](../README.md) • **Docs**

***

[lang-js](../README.md) / IAssetBuilder

# Interface: IAssetBuilder

## Methods

### addProp()

> **addProp**(`prop`): `this`

#### Parameters

• **prop**: [`PropDefinition`](PropDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1028](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1028)

***

### addSecretProp()

> **addSecretProp**(`prop`): `this`

#### Parameters

• **prop**: [`SecretPropDefinition`](SecretPropDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1030](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1030)

***

### defineSecret()

> **defineSecret**(`definition`): `this`

#### Parameters

• **definition**: [`SecretDefinition`](SecretDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1032](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1032)

***

### addResourceProp()

> **addResourceProp**(`prop`): `this`

#### Parameters

• **prop**: [`PropDefinition`](PropDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1034](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1034)

***

### addInputSocket()

> **addInputSocket**(`socket`): `this`

#### Parameters

• **socket**: [`SocketDefinition`](SocketDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1036](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1036)

***

### addOutputSocket()

> **addOutputSocket**(`socket`): `this`

#### Parameters

• **socket**: [`SocketDefinition`](SocketDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1038](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1038)

***

### addSiPropValueFrom()

> **addSiPropValueFrom**(`siPropValueFrom`): `this`

#### Parameters

• **siPropValueFrom**: [`SiPropValueFromDefinition`](SiPropValueFromDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1040](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1040)

***

### addDocLink()

> **addDocLink**(`key`, `value`): `this`

#### Parameters

• **key**: `string`

• **value**: `string`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1042](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1042)

***

### build()

> **build**(): [`Asset`](Asset.md)

#### Returns

[`Asset`](Asset.md)

#### Defined in

[asset\_builder.ts:1044](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1044)
