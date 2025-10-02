import {
  assert,
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.224.0/assert/mod.ts";
import { executeFunction, FunctionKind, functionKinds } from "../src/function.ts";
import { AnyFunction, RequestCtx } from "../src/request.ts";
import { FuncBackendResponseType } from "../src/function_kinds/resolver_function.ts";

let lastLog = "";
console.log = (msg: string) => {
  console.dir(msg);
  lastLog = msg;
};

const ctx: RequestCtx = {
  executionId: "",
};

Deno.test("function edge cases - functionKinds returns all kinds", () => {
  const kinds = functionKinds();

  assert(kinds.includes(FunctionKind.ActionRun));
  assert(kinds.includes(FunctionKind.Before));
  assert(kinds.includes(FunctionKind.Management));
  assert(kinds.includes(FunctionKind.ResolverFunction));
  assert(kinds.includes(FunctionKind.Validation));
  assert(kinds.includes(FunctionKind.SchemaVariantDefinition));

  assert(kinds.length >= 6);
});

Deno.test("function edge cases - empty execution ID", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa("function main() { return { status: 'ok' }; }"),
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    executionId: "", // Empty execution ID
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.executionId, "");
  assertEquals(result.status, "success");
});

Deno.test("function edge cases - very long execution ID", async () => {
  lastLog = "";

  const longId = "x".repeat(1000);
  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa("function main() { return { status: 'ok' }; }"),
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    executionId: longId,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.executionId, longId);
});

Deno.test("function edge cases - special characters in handler name", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa("function $handler_123() { return { status: 'ok' }; }"),
    handler: "$handler_123",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
});

Deno.test("function edge cases - handler with reserved keywords", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa("function validHandler() { return { status: 'ok' }; }"),
    handler: "eval", // Reserved keyword that doesn't exist
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "failure");
});

Deno.test("function edge cases - empty handler name", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa("function main() { return { status: 'ok' }; }"),
    handler: "",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "failure");
});

Deno.test("function edge cases - base64 with invalid padding", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: "invalid-base64!!!",
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "failure");
});

Deno.test("function edge cases - empty base64", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: "",
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
  assertEquals(result.health, "error");
  assertEquals(result.message, "main is not defined");
});

Deno.test("function edge cases - multiple before functions", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        const val1 = requestStorage.getEnv("test1");
        const val2 = requestStorage.getEnv("test2");
        return { status: "ok", val1, val2 };
      }
    `),
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
    before: [
      {
        arg: {},
        codeBase64: btoa(`
          function main() {
            requestStorage.setEnv("test1", "value1");
          }
        `),
        handler: "main",
      },
      {
        arg: {},
        codeBase64: btoa(`
          function main() {
            requestStorage.setEnv("test2", "value2");
          }
        `),
        handler: "main",
      },
    ],
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
});

Deno.test("function edge cases - before function throws error", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa("function main() { return { status: 'ok' }; }"),
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
    before: [
      {
        arg: {},
        codeBase64: btoa(`
          function main() {
            throw new Error("Before function failed");
          }
        `),
        handler: "main",
      },
    ],
  }, 30);

  // Before function errors are logged but don't fail the main function
  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
  assertEquals(result.health, "ok");
});

Deno.test("function edge cases - resolver function with complex value", async () => {
  lastLog = "";

  const complexValue = {
    nested: {
      array: [1, 2, 3],
      object: { key: "value" },
    },
    nullValue: null,
    boolValue: true,
  };

  const funcObj: AnyFunction = {
    value: complexValue,
    component: {
      data: {
        name: "test-component",
        properties: complexValue,
      },
      parents: [],
    },
    responseType: FuncBackendResponseType.Object,
    codeBase64: btoa(`
      function main(input) {
        return {
          receivedNested: input.nested !== undefined,
          arrayLength: input.nested?.array?.length,
        };
      }
    `),
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ResolverFunction,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
  assertEquals(result.data.receivedNested, true);
  assertEquals(result.data.arrayLength, 3);
});

Deno.test("function edge cases - validation with edge case values", async () => {
  lastLog = "";

  // Test with negative number
  const funcObj: AnyFunction = {
    value: -999,
    validationFormat: JSON.stringify({
      type: "number",
      rules: [{ name: "min", args: { limit: -1000 } }],
    }),
    codeBase64: "",
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.Validation,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
});

Deno.test("function edge cases - management function", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    thisComponent: {
      properties: {},
      sources: {},
      geometry: { default: { x: 0, y: 0, width: 100, height: 100 } },
    },
    components: {},
    currentView: "test-view",
    variantSocketMap: {},
    codeBase64: btoa(`
      function main() {
        return { status: "ok", message: "Management succeeded" };
      }
    `),
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.Management,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
  assertEquals(result.health, "ok");
  assertEquals(result.message, "Management succeeded");
});

Deno.test("function edge cases - very short timeout", {
  sanitizeOps: false,
  sanitizeResources: false,
}, async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      async function main() {
        await new Promise(resolve => setTimeout(resolve, 5000));
        return { status: "ok" };
      }
    `),
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
  }, 0.1); // 100ms timeout

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "failure");
  assertEquals(result.error?.kind?.UserCodeException, "TimeoutError");
  assertEquals(result.error?.message, "function timed out after 0.1 seconds");
});

Deno.test("function edge cases - function returns array", {
  sanitizeOps: false,
  sanitizeResources: false,
}, async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        return { status: "ok", payload: [1, 2, 3, 4, 5] };
      }
    `),
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
  assertEquals(result.health, "ok");
  assertEquals(result.payload, [1, 2, 3, 4, 5]);
});

Deno.test("function edge cases - function with console output", {
  sanitizeOps: false,
  sanitizeResources: false,
}, async () => {
  lastLog = "";
  const allLogs: string[] = [];

  const originalLog = console.log;
  console.log = (msg: string) => {
    originalLog(msg);
    allLogs.push(msg);
    lastLog = msg;
  };

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        console.log("Debug message");
        return { status: "ok" };
      }
    `),
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
  }, 30);

  console.log = originalLog;

  // Should have output logs and result
  assert(allLogs.length > 1);
  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
});
