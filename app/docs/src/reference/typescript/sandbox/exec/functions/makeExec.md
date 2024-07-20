[**TypeScript Function API**](../../../README.md) • **Docs**

***

[TypeScript Function API](../../../README.md) / [sandbox/exec](../README.md) / makeExec

# Function: makeExec()

> **makeExec**(`executionId`): `object`

## Parameters

• **executionId**: `string`

## Returns

`object`

### waitUntilEnd()

> **waitUntilEnd**: (`execaFile`, `execaArgs`?) => `Promise`\<[`SiExecResult`](../type-aliases/SiExecResult.md)\>

Runs a command and waits until it finishes executing.

#### Parameters

• **execaFile**: `string`

• **execaArgs?**: readonly `string`[]

#### Returns

`Promise`\<[`SiExecResult`](../type-aliases/SiExecResult.md)\>

#### Example

```ts
const child = siExec.waitUntilEnd("aws", [
  "ec2",
  "describe-hosts"
]);
```

### watch()

> **watch**: (`options`, `deadlineCount`?) => `Promise`\<[`WatchResult`](../interfaces/WatchResult.md)\>

#### Parameters

• **options**: [`WatchArgs`](../interfaces/WatchArgs.md)

• **deadlineCount?**: `number`

#### Returns

`Promise`\<[`WatchResult`](../interfaces/WatchResult.md)\>

## Defined in

[sandbox/exec.ts:30](https://github.com/systeminit/si/blob/main/bin/lang-js/src/sandbox/exec.ts#L30)
