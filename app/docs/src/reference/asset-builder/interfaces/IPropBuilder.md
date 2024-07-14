[**lang-js**](../README.md) • **Docs**

***

[lang-js](../README.md) / IPropBuilder

# Interface: IPropBuilder

## Methods

### setName()

> **setName**(`name`): `this`

#### Parameters

• **name**: `string`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:510](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L510)

***

### setKind()

> **setKind**(`kind`): `this`

#### Parameters

• **kind**: [`PropDefinitionKind`](../type-aliases/PropDefinitionKind.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:512](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L512)

***

### setDocLinkRef()

> **setDocLinkRef**(`ref`): `this`

#### Parameters

• **ref**: `string`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:514](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L514)

***

### setDocumentation()

> **setDocumentation**(`ref`): `this`

#### Parameters

• **ref**: `string`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:516](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L516)

***

### setDocLink()

> **setDocLink**(`link`): `this`

#### Parameters

• **link**: `string`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:518](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L518)

***

### addChild()

> **addChild**(`child`): `this`

#### Parameters

• **child**: [`PropDefinition`](PropDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:520](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L520)

***

### setEntry()

> **setEntry**(`entry`): `this`

#### Parameters

• **entry**: [`PropDefinition`](PropDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:522](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L522)

***

### setWidget()

> **setWidget**(`widget`): `this`

#### Parameters

• **widget**: [`PropWidgetDefinition`](PropWidgetDefinition.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:524](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L524)

***

### setValueFrom()

> **setValueFrom**(`valueFrom`): `this`

#### Parameters

• **valueFrom**: [`ValueFrom`](ValueFrom.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:526](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L526)

***

### setHidden()

> **setHidden**(`hidden`): `this`

#### Parameters

• **hidden**: `boolean`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:528](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L528)

***

### setDefaultValue()

> **setDefaultValue**(`value`): `this`

#### Parameters

• **value**: `any`

#### Returns

`this`

#### Defined in

[asset\_builder.ts:531](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L531)

***

### setValidationFormat()

> **setValidationFormat**(`format`): `this`

#### Parameters

• **format**: `Schema`\<`any`\>

#### Returns

`this`

#### Defined in

[asset\_builder.ts:533](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L533)

***

### addMapKeyFunc()

> **addMapKeyFunc**(`func`): `this`

#### Parameters

• **func**: [`MapKeyFunc`](MapKeyFunc.md)

#### Returns

`this`

#### Defined in

[asset\_builder.ts:535](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L535)

***

### build()

> **build**(): [`PropDefinition`](PropDefinition.md)

#### Returns

[`PropDefinition`](PropDefinition.md)

#### Defined in

[asset\_builder.ts:537](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L537)
