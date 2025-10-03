import { assertObjectMatch } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { FunctionKind } from "../src/function.ts";
import { createSandbox } from "../src/sandbox.ts";

Deno.test("createSandbox", () => {
  const sandbox = createSandbox(FunctionKind.ResolverFunction, "poop");

  assertObjectMatch(sandbox, {
    _: sandbox._,
  });
});
