[**TypeScript Function API**](../../../README.md) • **Docs**

***

[TypeScript Function API](../../../README.md) / [sandbox/exec](../README.md) / WatchArgs

# Interface: WatchArgs

## Properties

### cmd

> **cmd**: `string`

#### Defined in

[sandbox/exec.ts:7](https://github.com/systeminit/si/blob/main/bin/lang-js/src/sandbox/exec.ts#L7)

***

### args?

> `optional` **args**: readonly `string`[]

#### Defined in

[sandbox/exec.ts:8](https://github.com/systeminit/si/blob/main/bin/lang-js/src/sandbox/exec.ts#L8)

***

### execaOptions?

> `optional` **execaOptions**: `Options`\<`string`\>

#### Defined in

[sandbox/exec.ts:9](https://github.com/systeminit/si/blob/main/bin/lang-js/src/sandbox/exec.ts#L9)

***

### retryMs?

> `optional` **retryMs**: `number`

#### Defined in

[sandbox/exec.ts:10](https://github.com/systeminit/si/blob/main/bin/lang-js/src/sandbox/exec.ts#L10)

***

### maxRetryCount?

> `optional` **maxRetryCount**: `number`

#### Defined in

[sandbox/exec.ts:11](https://github.com/systeminit/si/blob/main/bin/lang-js/src/sandbox/exec.ts#L11)

***

### callback()

> **callback**: (`child`) => `Promise`\<`boolean`\>

#### Parameters

• **child**: `ExecaReturnValue`\<`string`\>

#### Returns

`Promise`\<`boolean`\>

#### Defined in

[sandbox/exec.ts:12](https://github.com/systeminit/si/blob/main/bin/lang-js/src/sandbox/exec.ts#L12)
