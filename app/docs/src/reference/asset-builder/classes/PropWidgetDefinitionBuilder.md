[**lang-js**](../README.md) • **Docs**

***

[lang-js](../README.md) / PropWidgetDefinitionBuilder

# Class: PropWidgetDefinitionBuilder

Create a widget for interacting with a prop that is displayed in the modelling view.

## Example

```ts
const validation = new PropWidgetDefinitionBuilder()
 .setKind("text")
 .build()
```

## Implements

- [`IPropWidgetDefinitionBuilder`](../interfaces/IPropWidgetDefinitionBuilder.md)

## Constructors

### new PropWidgetDefinitionBuilder()

> **new PropWidgetDefinitionBuilder**(): [`PropWidgetDefinitionBuilder`](PropWidgetDefinitionBuilder.md)

#### Returns

[`PropWidgetDefinitionBuilder`](PropWidgetDefinitionBuilder.md)

#### Defined in

[asset\_builder.ts:309](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L309)

## Properties

### propWidget

> **propWidget**: [`PropWidgetDefinition`](../interfaces/PropWidgetDefinition.md)

#### Defined in

[asset\_builder.ts:307](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L307)

## Methods

### setKind()

> **setKind**(`kind`): `this`

The type of widget

#### Parameters

• **kind**: [`PropWidgetDefinitionKind`](../type-aliases/PropWidgetDefinitionKind.md)

{PropWidgetDefinitionKind} [array | checkbox | color | comboBox | header | map | select | text | textArea | codeEditor | password]

#### Returns

`this`

this

#### Example

```ts
.setKind("color")
```

#### Implementation of

[`IPropWidgetDefinitionBuilder`](../interfaces/IPropWidgetDefinitionBuilder.md).[`setKind`](../interfaces/IPropWidgetDefinitionBuilder.md#setkind)

#### Defined in

[asset\_builder.ts:323](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L323)

***

### addOption()

> **addOption**(`key`, `value`): `this`

Add an option when using a comboBox

#### Parameters

• **key**: `string`

the value displayed in the comboBox

• **value**: `string`

the value the prop is set to

#### Returns

`this`

this

#### Example

```ts
.setOption("us-east-2 - US East (Ohio)", "us-east-2")
```

#### Implementation of

[`IPropWidgetDefinitionBuilder`](../interfaces/IPropWidgetDefinitionBuilder.md).[`addOption`](../interfaces/IPropWidgetDefinitionBuilder.md#addoption)

#### Defined in

[asset\_builder.ts:340](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L340)

***

### build()

> **build**(): [`PropWidgetDefinition`](../interfaces/PropWidgetDefinition.md)

Build the object

#### Returns

[`PropWidgetDefinition`](../interfaces/PropWidgetDefinition.md)

#### Example

```ts
.build()
```

#### Implementation of

[`IPropWidgetDefinitionBuilder`](../interfaces/IPropWidgetDefinitionBuilder.md).[`build`](../interfaces/IPropWidgetDefinitionBuilder.md#build)

#### Defined in

[asset\_builder.ts:358](https://github.com/systeminit/si/blob/main/bin/lang-js/src/asset_builder.ts#L358)
