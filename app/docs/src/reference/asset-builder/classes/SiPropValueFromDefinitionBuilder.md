[**lang-js**](../README.md) • **Docs**

***

[lang-js](../README.md) / SiPropValueFromDefinitionBuilder

# Class: SiPropValueFromDefinitionBuilder

## Implements

- [`ISiPropValueFromDefinitionBuilder`](../interfaces/ISiPropValueFromDefinitionBuilder.md)

## Constructors

### new SiPropValueFromDefinitionBuilder()

> **new SiPropValueFromDefinitionBuilder**(): [`SiPropValueFromDefinitionBuilder`](SiPropValueFromDefinitionBuilder.md)

#### Returns

[`SiPropValueFromDefinitionBuilder`](SiPropValueFromDefinitionBuilder.md)

#### Defined in

[asset\_builder.ts:459](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L459)

## Properties

### definition

> **definition**: [`SiPropValueFromDefinition`](../interfaces/SiPropValueFromDefinition.md)

#### Defined in

[asset\_builder.ts:457](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L457)

## Methods

### build()

> **build**(): [`SiPropValueFromDefinition`](../interfaces/SiPropValueFromDefinition.md)

Build the object

#### Returns

[`SiPropValueFromDefinition`](../interfaces/SiPropValueFromDefinition.md)

#### Example

```ts
.build()
```

#### Implementation of

[`ISiPropValueFromDefinitionBuilder`](../interfaces/ISiPropValueFromDefinitionBuilder.md).[`build`](../interfaces/ISiPropValueFromDefinitionBuilder.md#build)

#### Defined in

[asset\_builder.ts:469](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L469)

***

### setKind()

> **setKind**(`kind`): `this`

#### Parameters

• **kind**: [`SiPropValueFromDefinitionKind`](../type-aliases/SiPropValueFromDefinitionKind.md)

#### Returns

`this`

#### Implementation of

[`ISiPropValueFromDefinitionBuilder`](../interfaces/ISiPropValueFromDefinitionBuilder.md).[`setKind`](../interfaces/ISiPropValueFromDefinitionBuilder.md#setkind)

#### Defined in

[asset\_builder.ts:473](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L473)

***

### setValueFrom()

> **setValueFrom**(`valueFrom`): `this`

#### Parameters

• **valueFrom**: [`ValueFrom`](../interfaces/ValueFrom.md)

#### Returns

`this`

#### Implementation of

[`ISiPropValueFromDefinitionBuilder`](../interfaces/ISiPropValueFromDefinitionBuilder.md).[`setValueFrom`](../interfaces/ISiPropValueFromDefinitionBuilder.md#setvaluefrom)

#### Defined in

[asset\_builder.ts:478](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L478)
