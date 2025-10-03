import {
  assertEquals,
  assertRejects,
} from "https://deno.land/std@0.224.0/assert/mod.ts";
import { executeFunction, FunctionKind } from "../src/function.ts";
import { AnyFunction, RequestCtx } from "../src/request.ts";

let lastLog = "";
console.log = (msg: string) => {
  console.dir(msg);
  lastLog = msg;
};

const ctx: RequestCtx = {
  executionId: "",
};

Deno.test("execution edge cases - malformed handler name", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa("function validFunction() { return { status: 'ok' }; }"),
    handler: "nonExistentHandler",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
  assertEquals(result.health, "error");
  assertEquals(result.message, "nonExistentHandler is not defined");
});

Deno.test("execution edge cases - syntax error in code", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa("function main() { this is invalid javascript }"),
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

Deno.test("execution edge cases - runtime error in function", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        throw new Error("Intentional runtime error");
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
  assertEquals(result.health, "error");
  assertEquals(result.message, "Intentional runtime error");
});

Deno.test("execution edge cases - function returns null", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        return null;
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
  assertEquals(result.status, "failure");
});

Deno.test("execution edge cases - function returns undefined", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        return undefined;
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
  assertEquals(result.status, "failure");
});

Deno.test("execution edge cases - function returns non-serializable object", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        const circular = {};
        circular.self = circular;
        return { status: "ok", data: circular };
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
  assertEquals(result.status, "failure");
});

Deno.test("execution edge cases - async function with rejection", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      async function main() {
        await Promise.reject(new Error("Async rejection"));
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
  assertEquals(result.health, "error");
  assertEquals(result.message, "Async rejection");
});

Deno.test("execution edge cases - function with infinite recursion", {
  sanitizeOps: false,
  sanitizeResources: false,
}, async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        return main();
      }
    `),
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.ActionRun,
    ...funcObj,
    ...ctx,
  }, 2); // Short timeout

  const result = JSON.parse(lastLog);
  // Stack overflow is caught and returned as error health
  assertEquals(result.status, "success");
  assertEquals(result.health, "error");
  assertEquals(result.message, "Maximum call stack size exceeded");
});

Deno.test("execution edge cases - empty code", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(""),
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

Deno.test("execution edge cases - very large output", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        // Generate large but valid output
        const largeArray = new Array(1000).fill({ data: "x".repeat(100) });
        return { status: "ok", payload: largeArray };
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
  assertEquals(result.payload.length, 1000);
});

Deno.test("execution edge cases - special characters in output", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        return {
          status: "ok",
          payload: {
            message: "Special chars: \\n\\t\\r\\"\\'\`",
            unicode: String.fromCodePoint(0x4E16, 0x754C) + " " + String.fromCodePoint(0x1F30D)
          }
        };
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
  assertEquals(result.payload.message, "Special chars: \n\t\r\"'`");
  assertEquals(result.payload.unicode, "世界 🌍");
});

Deno.test("execution edge cases - validation with invalid format type", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: 123,
    validationFormat: JSON.stringify({ type: "invalidType" }),
    codeBase64: "",
    handler: "main",
  };

  await executeFunction({
    kind: FunctionKind.Validation,
    ...funcObj,
    ...ctx,
  }, 30);

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "failure");
});

Deno.test("execution edge cases - function modifies globalThis", async () => {
  lastLog = "";

  const funcObj: AnyFunction = {
    value: {},
    codeBase64: btoa(`
      function main() {
        globalThis.malicious = "hacked";
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

  const result = JSON.parse(lastLog);
  assertEquals(result.status, "success");
  // Verify isolation - globalThis should not be affected in parent
  assertEquals((globalThis as any).malicious, undefined);
});
