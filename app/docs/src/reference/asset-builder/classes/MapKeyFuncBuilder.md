[**lang-js**](../README.md) • **Docs**

***

[lang-js](../README.md) / MapKeyFuncBuilder

# Class: MapKeyFuncBuilder

Used to add a value to a map

## Example

```ts
const mapButton = new MapKeyFuncBuilder()
   .setKey("Name")
   .build()
```

## Implements

- [`IMapKeyFuncBuilder`](../interfaces/IMapKeyFuncBuilder.md)

## Constructors

### new MapKeyFuncBuilder()

> **new MapKeyFuncBuilder**(): [`MapKeyFuncBuilder`](MapKeyFuncBuilder.md)

#### Returns

[`MapKeyFuncBuilder`](MapKeyFuncBuilder.md)

#### Defined in

[asset\_builder.ts:387](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L387)

## Properties

### mapKeyFunc

> **mapKeyFunc**: [`MapKeyFunc`](../interfaces/MapKeyFunc.md)

#### Defined in

[asset\_builder.ts:385](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L385)

## Methods

### build()

> **build**(): [`MapKeyFunc`](../interfaces/MapKeyFunc.md)

Build the object

#### Returns

[`MapKeyFunc`](../interfaces/MapKeyFunc.md)

#### Example

```ts
.build()
```

#### Implementation of

[`IMapKeyFuncBuilder`](../interfaces/IMapKeyFuncBuilder.md).[`build`](../interfaces/IMapKeyFuncBuilder.md#build)

#### Defined in

[asset\_builder.ts:397](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L397)

***

### setKey()

> **setKey**(`key`): `this`

Set the value of the key for the map entry

#### Parameters

• **key**: `string`

the name of the key

#### Returns

`this`

this

#### Example

```ts
.setKey("Name")
```

#### Implementation of

[`IMapKeyFuncBuilder`](../interfaces/IMapKeyFuncBuilder.md).[`setKey`](../interfaces/IMapKeyFuncBuilder.md#setkey)

#### Defined in

[asset\_builder.ts:411](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L411)

***

### setValueFrom()

> **setValueFrom**(`valueFrom`): `this`

DEPRECATED: Set the value of this socket using a ValueFromBuilder.
The recommended way to do this is to attach an attribute function.

#### Parameters

• **valueFrom**: [`ValueFrom`](../interfaces/ValueFrom.md)

#### Returns

`this`

this

#### Example

```ts
.setValueFrom(new ValueFromBuilder()
   .setKind("prop")
   .setPropPath(["root", "si", "name"])
   .build())
```

#### Implementation of

[`IMapKeyFuncBuilder`](../interfaces/IMapKeyFuncBuilder.md).[`setValueFrom`](../interfaces/IMapKeyFuncBuilder.md#setvaluefrom)

#### Defined in

[asset\_builder.ts:430](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L430)
