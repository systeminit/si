import { join } from "https://deno.land/std@0.224.0/path/mod.ts";
import { executeFunction, FunctionKind } from "../src/function.ts";
import { AnyFunction, RequestCtx } from "../src/request.ts";

async function base64FromFile(path: string) {
  const buffer = await Deno.readFile(join(Deno.cwd(), path));
  return btoa(new TextDecoder().decode(buffer));
}

let lastLog = "";
console.log = (msg: string) => {
  console.dir(msg);
  lastLog = msg;
};

// Test timeout functionality - this kills a process so we disable sanitization
Deno.test({
  name: "executeFunction - Will Timeout",
  sanitizeOps: false,
  sanitizeResources: false,
  async fn() {
    const FUNCS_FOLDER = "./bin/lang-js/tests/functions/";
    const codeBase64 = await base64FromFile(FUNCS_FOLDER + "willTimeout.ts");

    const ctx: RequestCtx = {
      executionId: "",
    };

    const funcObj: AnyFunction = {
      value: {},
      codeBase64,
      handler: "main",
    };

    await executeFunction({
      kind: FunctionKind.ActionRun,
      ...funcObj,
      ...ctx,
    }, 1); // 1 second timeout

    const result = JSON.parse(lastLog);

    // Verify the timeout resulted in a failure with the correct error message
    if (result.status !== "failure") {
      throw new Error(`Expected status "failure" but got "${result.status}"`);
    }
    if (!result.error?.message?.includes("function timed out after 1 seconds")) {
      throw new Error(`Expected timeout error message but got: ${result.error?.message}`);
    }
  },
});
