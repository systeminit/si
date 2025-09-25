[**TypeScript Function API**](../../README.md) • **Docs**

***

[TypeScript Function API](../../README.md) / [asset\_builder](../README.md) / IAssetBuilder

# Interface: IAssetBuilder

## Methods

### addProp()

> **addProp**(`prop`): `this`

#### Parameters

• **prop**: [`PropDefinition`](PropDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1023](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1023)

***

### addSecretProp()

> **addSecretProp**(`prop`): `this`

#### Parameters

• **prop**: [`SecretPropDefinition`](SecretPropDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1025](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1025)

***

### defineSecret()

> **defineSecret**(`definition`): `this`

#### Parameters

• **definition**: [`SecretDefinition`](SecretDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1027](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1027)

***

### addResourceProp()

> **addResourceProp**(`prop`): `this`

#### Parameters

• **prop**: [`PropDefinition`](PropDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1029](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1029)

***

### addSiPropValueFrom()

> **addSiPropValueFrom**(`siPropValueFrom`): `this`

#### Parameters

• **siPropValueFrom**: [`SiPropValueFromDefinition`](SiPropValueFromDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1035](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1035)

***

### addDocLink()

> **addDocLink**(`key`, `value`): `this`

#### Parameters

• **key**: `string`

• **value**: `string`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:1037](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1037)

***

### build()

> **build**(): [`Asset`](Asset.md)

#### Returns

[`Asset`](Asset.md)

#### Defined in

[asset\_builder.ts:1039](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L1039)
