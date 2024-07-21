[**TypeScript Function API**](../../README.md) • **Docs**

***

[TypeScript Function API](../../README.md) / [asset\_builder](../README.md) / SocketDefinitionBuilder

# Class: SocketDefinitionBuilder

Defines an input or output socket for passing values between components

## Example

```ts
const regionSocket = new SocketDefinitionBuilder()
 .setName("Region")
 .setArity("one")
 .build();
```

## Implements

- [`ISocketDefinitionBuilder`](../interfaces/ISocketDefinitionBuilder.md)

## Constructors

### new SocketDefinitionBuilder()

> **new SocketDefinitionBuilder**(): [`SocketDefinitionBuilder`](SocketDefinitionBuilder.md)

#### Returns

[`SocketDefinitionBuilder`](SocketDefinitionBuilder.md)

#### Defined in

[asset\_builder.ts:141](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L141)

## Properties

### socket

> **socket**: [`SocketDefinition`](../interfaces/SocketDefinition.md)

#### Defined in

[asset\_builder.ts:138](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L138)

***

### connectionAnnotations

> **connectionAnnotations**: `string`[] = `[]`

#### Defined in

[asset\_builder.ts:139](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L139)

## Methods

### build()

> **build**(): [`SocketDefinition`](../interfaces/SocketDefinition.md)

Build the object

#### Returns

[`SocketDefinition`](../interfaces/SocketDefinition.md)

#### Example

```ts
.build()
```

#### Implementation of

[`ISocketDefinitionBuilder`](../interfaces/ISocketDefinitionBuilder.md).[`build`](../interfaces/ISocketDefinitionBuilder.md#build)

#### Defined in

[asset\_builder.ts:151](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L151)

***

### setArity()

> **setArity**(`arity`): `this`

Specify the number of connections the socket can support

#### Parameters

• **arity**: [`SocketDefinitionArityType`](../type-aliases/SocketDefinitionArityType.md)

[one | many]

#### Returns

`this`

this

#### Example

```ts
.setArity("one")
```

#### Implementation of

[`ISocketDefinitionBuilder`](../interfaces/ISocketDefinitionBuilder.md).[`setArity`](../interfaces/ISocketDefinitionBuilder.md#setarity)

#### Defined in

[asset\_builder.ts:175](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L175)

***

### setConnectionAnnotation()

> **setConnectionAnnotation**(`annotation`): `this`

Add a field to the connection annotations array for the socket.
The input should be sequence of word chars (\w regex matcher), optionally
followed by any `<identifier>`, which makes it a supertype of `identifier`.
This can be repeated recursively as many times as necessary (see example).
At socket connecting time an *input* socket can receive a connection of any
*output* socket that has a compatible connection annotation.

e.g. An input socket with the `Port<string>` connection
annotation can receive a
connection from an output socket with the `Docker<Port<string>>` annotation,
but not one with just `string`.

The socket's name is always one of the connection annotations.

#### Parameters

• **annotation**: `string`

#### Returns

`this`

this

#### Example

```ts
.setConnectionAnnotation("EC2<IAM<string>>")
```

#### Implementation of

[`ISocketDefinitionBuilder`](../interfaces/ISocketDefinitionBuilder.md).[`setConnectionAnnotation`](../interfaces/ISocketDefinitionBuilder.md#setconnectionannotation)

#### Defined in

[asset\_builder.ts:202](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L202)

***

### setName()

> **setName**(`name`): `this`

The name of the socket. Note that this will be used to connect sockets
and to reference the socket within the asset.

#### Parameters

• **name**: `string`

#### Returns

`this`

this

#### Example

```ts
.setName("Subnet ID")
```

#### Implementation of

[`ISocketDefinitionBuilder`](../interfaces/ISocketDefinitionBuilder.md).[`setName`](../interfaces/ISocketDefinitionBuilder.md#setname)

#### Defined in

[asset\_builder.ts:221](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L221)

***

### setUiHidden()

> **setUiHidden**(`hidden`): `this`

Should this socket show in the UI. Note that the socket can still be connected when the component is placed in a frame.

#### Parameters

• **hidden**: `boolean`

#### Returns

`this`

this

#### Example

```ts
.setName("Subnet ID")
```

#### Implementation of

[`ISocketDefinitionBuilder`](../interfaces/ISocketDefinitionBuilder.md).[`setUiHidden`](../interfaces/ISocketDefinitionBuilder.md#setuihidden)

#### Defined in

[asset\_builder.ts:236](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L236)

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
   .setKind("inputSocket")
   .setSocketName("Region")
   .build())
```

#### Implementation of

[`ISocketDefinitionBuilder`](../interfaces/ISocketDefinitionBuilder.md).[`setValueFrom`](../interfaces/ISocketDefinitionBuilder.md#setvaluefrom)

#### Defined in

[asset\_builder.ts:255](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L255)
