import { VM, VMScript } from "vm2";
import { compileCode, createVm } from "../src/vm";
import { createSandbox } from "../src/sandbox";

import { FunctionKind } from "../src/function";

describe("createVm", () => {
  test("creates a new vm for execution", () => {
    const sandbox = createSandbox(FunctionKind.ResolverFunction, "poop");
    const vm = createVm(sandbox);
    expect(vm).toBeInstanceOf(VM);
  });
});

describe("compileCode", () => {
  test("compiles code for execution", () => {
    const code = compileCode("'foo'");
    expect(code).toBeInstanceOf(VMScript);
  });
});

describe("runCode", () => {
  test("runs code on a vm, returning the result", () => {
    const sandbox = createSandbox(FunctionKind.ResolverFunction, "poop");
    const vm = createVm(sandbox);
    const code = compileCode("'foo'");
    const result = vm.run(code);
    expect(result).toBe("foo");
  });
});
