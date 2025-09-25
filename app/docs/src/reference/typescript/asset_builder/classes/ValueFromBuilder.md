[**TypeScript Function API**](../../README.md) • **Docs**

***

[TypeScript Function API](../../README.md) / [asset\_builder](../README.md) / ValueFromBuilder

# Class: ValueFromBuilder

Gets a value from a prop

## Example

```ts
const value = new ValueFromBuilder()
 .setKind("prop")
 .setPropPath(["root", "si", "name"])
 .build()
```

## Implements

- [`IValueFromBuilder`](../interfaces/IValueFromBuilder.md)

## Constructors

### new ValueFromBuilder()

> **new ValueFromBuilder**(): [`ValueFromBuilder`](ValueFromBuilder.md)

#### Returns

[`ValueFromBuilder`](ValueFromBuilder.md)

#### Defined in

[asset\_builder.ts:34](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L34)

## Properties

### valueFrom

> **valueFrom**: [`ValueFrom`](../interfaces/ValueFrom.md)

#### Defined in

[asset\_builder.ts:32](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L32)

## Methods

### setKind()

> **setKind**(`kind`): `this`

The type of the builder

#### Parameters

• **kind**: [`ValueFromKind`](../type-aliases/ValueFromKind.md)

{string} [prop]

#### Returns

`this`

this

#### Example

```ts
.setKind("prop")
```

#### Implementation of

[`IValueFromBuilder`](../interfaces/IValueFromBuilder.md).[`setKind`](../interfaces/IValueFromBuilder.md#setkind)

#### Defined in

[asset\_builder.ts:48](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L48)

***

### setPropPath()

> **setPropPath**(`path`): `this`

Specify the prop path if using a prop

#### Parameters

• **path**: `string`[]

a list of strings that represent the path to the prop

#### Returns

`this`

this

#### Example

```ts
.setPropPath(["root", "si", "name"])
```

#### Implementation of

[`IValueFromBuilder`](../interfaces/IValueFromBuilder.md).[`setPropPath`](../interfaces/IValueFromBuilder.md#setproppath)

#### Defined in

[asset\_builder.ts:84](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L84)

***

### build()

> **build**(): [`ValueFrom`](../interfaces/ValueFrom.md)

Build the object

#### Returns

[`ValueFrom`](../interfaces/ValueFrom.md)

#### Example

```ts
.build()
```

#### Implementation of

[`IValueFromBuilder`](../interfaces/IValueFromBuilder.md).[`build`](../interfaces/IValueFromBuilder.md#build)

#### Defined in

[asset\_builder.ts:99](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L99)
