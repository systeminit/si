[**TypeScript Function API**](../../README.md) • **Docs**

***

[TypeScript Function API](../../README.md) / [asset\_builder](../README.md) / PropBuilder

# Class: PropBuilder

Creates a prop to attach values to an asset

## Example

```ts
const propName = new PropBuilder()
  .setName("name")
  .setKind("string")
  .setDocumentation("This is the documentation for the prop")
  .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
 .build();
```

## Implements

- [`IPropBuilder`](../interfaces/IPropBuilder.md)

## Constructors

### new PropBuilder()

> **new PropBuilder**(): [`PropBuilder`](PropBuilder.md)

#### Returns

[`PropBuilder`](PropBuilder.md)

## Properties

### prop

> **prop**: [`PropDefinition`](../interfaces/PropDefinition.md)

#### Defined in

[asset\_builder.ts:547](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L547)

## Methods

### addChild()

> **addChild**(`child`): `this`

Adds a child to an object type prop

#### Parameters

• **child**: [`PropDefinition`](../interfaces/PropDefinition.md)

#### Returns

`this`

this

#### Example

```ts
.addChild(new PropBuilder()
    .setKind("string")
    .setName("sweetChildProp")
    .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
    .build())
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`addChild`](../interfaces/IPropBuilder.md#addchild)

#### Defined in

[asset\_builder.ts:563](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L563)

***

### setEntry()

> **setEntry**(`entry`): `this`

Adds an entry to array or map type props

#### Parameters

• **entry**: [`PropDefinition`](../interfaces/PropDefinition.md)

#### Returns

`this`

this

#### Example

```ts
.setEntry(new PropBuilder()
    .setKind("string")
    .setName("iamanentryprop")
    .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
    .build())
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setEntry`](../interfaces/IPropBuilder.md#setentry)

#### Defined in

[asset\_builder.ts:590](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L590)

***

### addMapKeyFunc()

> **addMapKeyFunc**(`func`): `this`

Add a button for putting entries into maps

#### Parameters

• **func**: [`MapKeyFunc`](../interfaces/MapKeyFunc.md)

#### Returns

`this`

this

#### Example

```ts
.addMapKeyFunc(new MapKeyFuncBuilder()
   .setKey("Name")
   .build()
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`addMapKeyFunc`](../interfaces/IPropBuilder.md#addmapkeyfunc)

#### Defined in

[asset\_builder.ts:613](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L613)

***

### setValidationFormat()

> **setValidationFormat**(`format`): `this`

Add joi validation schema to this prop

#### Parameters

• **format**: `Schema`\<`any`\>

{Joi.Schema} - A joi schema object

#### Returns

`this`

this

#### Example

```ts
.setValidationFormat(Joi.string().required())
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setValidationFormat`](../interfaces/IPropBuilder.md#setvalidationformat)

#### Defined in

[asset\_builder.ts:630](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L630)

***

### build()

> **build**(): [`PropDefinition`](../interfaces/PropDefinition.md)

Build the object

#### Returns

[`PropDefinition`](../interfaces/PropDefinition.md)

#### Example

```ts
.build()
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`build`](../interfaces/IPropBuilder.md#build)

#### Defined in

[asset\_builder.ts:647](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L647)

***

### setDefaultValue()

> **setDefaultValue**(`value`): `this`

Set a value to be automatically populated in the prop

#### Parameters

• **value**: `any`

#### Returns

`this`

this

#### Example

```ts
.setDefaultValue("cats")
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setDefaultValue`](../interfaces/IPropBuilder.md#setdefaultvalue)

#### Defined in

[asset\_builder.ts:662](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L662)

***

### setDocLink()

> **setDocLink**(`link`): `this`

Set a link to external documentation that will appear beneath the prop

#### Parameters

• **link**: `string`

#### Returns

`this`

this

#### Example

```ts
.setDocLink("https://www.systeminit.com/")
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setDocLink`](../interfaces/IPropBuilder.md#setdoclink)

#### Defined in

[asset\_builder.ts:677](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L677)

***

### setDocumentation()

> **setDocumentation**(`docs`): `this`

Sets inline documentation for the prop

#### Parameters

• **docs**: `string`

#### Returns

`this`

this

#### Example

```ts
.setDocumentation("This is documentation for the prop")
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setDocumentation`](../interfaces/IPropBuilder.md#setdocumentation)

#### Defined in

[asset\_builder.ts:692](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L692)

***

### setDocLinkRef()

> **setDocLinkRef**(`ref`): `this`

#### Parameters

• **ref**: `string`

#### Returns

`this`

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setDocLinkRef`](../interfaces/IPropBuilder.md#setdoclinkref)

#### Defined in

[asset\_builder.ts:697](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L697)

***

### setHidden()

> **setHidden**(`hidden`): `this`

Whether the prop should be displayed in th UI or not

#### Parameters

• **hidden**: `boolean`

#### Returns

`this`

this

#### Example

```ts
.setHidden(true)
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setHidden`](../interfaces/IPropBuilder.md#sethidden)

#### Defined in

[asset\_builder.ts:712](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L712)

***

### setKind()

> **setKind**(`kind`): `this`

The type of the prop

#### Parameters

• **kind**: [`PropDefinitionKind`](../type-aliases/PropDefinitionKind.md)

{PropDefinitionKind} [array | boolean | integer | map | object | string]

#### Returns

`this`

this

#### Example

```ts
.setKind("text")
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setKind`](../interfaces/IPropBuilder.md#setkind)

#### Defined in

[asset\_builder.ts:727](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L727)

***

### setName()

> **setName**(`name`): `this`

The prop name. This will appear in the model UI

#### Parameters

• **name**: `string`

the name of the prop

#### Returns

`this`

this

#### Example

```ts
.setName("Region")
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setName`](../interfaces/IPropBuilder.md#setname)

#### Defined in

[asset\_builder.ts:742](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L742)

***

### setValueFrom()

> **setValueFrom**(`valueFrom`): `this`

DEPRECATED: this method no longer does anything. It will be ignored
when executing the asset function. Please use the asset editing
interface to perform equivalent functionality.

If the prop is new, you will need to regenerate the asset first!

In the past, this was used to set the value of this prop using a
ValueFromBuilder.

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

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setValueFrom`](../interfaces/IPropBuilder.md#setvaluefrom)

#### Defined in

[asset\_builder.ts:761](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L761)

***

### setWidget()

> **setWidget**(`widget`): `this`

The type of widget for the prop, determing how it is displayed in the UI

#### Parameters

• **widget**: [`PropWidgetDefinition`](../interfaces/PropWidgetDefinition.md)

#### Returns

`this`

this

#### Example

```ts
setWidget(new PropWidgetDefinitionBuilder()
.setKind("text")
.build())
```

#### Implementation of

[`IPropBuilder`](../interfaces/IPropBuilder.md).[`setWidget`](../interfaces/IPropBuilder.md#setwidget)

#### Defined in

[asset\_builder.ts:778](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L778)
