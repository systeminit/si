[**TypeScript Function API**](../../README.md) • **Docs**

***

[TypeScript Function API](../../README.md) / [asset\_builder](../README.md) / IPropBuilder

# Interface: IPropBuilder

## Methods

### setName()

> **setName**(`name`): `this`

#### Parameters

• **name**: `string`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:505](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L505)

***

### setKind()

> **setKind**(`kind`): `this`

#### Parameters

• **kind**: [`PropDefinitionKind`](../type-aliases/PropDefinitionKind.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:507](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L507)

***

### setDocLinkRef()

> **setDocLinkRef**(`ref`): `this`

#### Parameters

• **ref**: `string`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:509](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L509)

***

### setDocumentation()

> **setDocumentation**(`ref`): `this`

#### Parameters

• **ref**: `string`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:511](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L511)

***

### setDocLink()

> **setDocLink**(`link`): `this`

#### Parameters

• **link**: `string`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:513](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L513)

***

### addChild()

> **addChild**(`child`): `this`

#### Parameters

• **child**: [`PropDefinition`](PropDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:515](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L515)

***

### setEntry()

> **setEntry**(`entry`): `this`

#### Parameters

• **entry**: [`PropDefinition`](PropDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:517](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L517)

***

### setWidget()

> **setWidget**(`widget`): `this`

#### Parameters

• **widget**: [`PropWidgetDefinition`](PropWidgetDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:519](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L519)

***

### setValueFrom()

> **setValueFrom**(`valueFrom`): `this`

#### Parameters

• **valueFrom**: [`ValueFrom`](ValueFrom.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:521](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L521)

***

### setHidden()

> **setHidden**(`hidden`): `this`

#### Parameters

• **hidden**: `boolean`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:523](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L523)

***

### setDefaultValue()

> **setDefaultValue**(`value`): `this`

#### Parameters

• **value**: `any`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:526](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L526)

***

### setValidationFormat()

> **setValidationFormat**(`format`): `this`

#### Parameters

• **format**: `Schema`\<`any`\>

#### Returns

`this`

#### Defined in

[asset\_builder.ts:528](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L528)

***

### addMapKeyFunc()

> **addMapKeyFunc**(`func`): `this`

#### Parameters

• **func**: [`MapKeyFunc`](MapKeyFunc.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:530](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L530)

***

### build()

> **build**(): [`PropDefinition`](PropDefinition.md)

#### Returns

[`PropDefinition`](PropDefinition.md)

#### Defined in

[asset\_builder.ts:532](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L532)
