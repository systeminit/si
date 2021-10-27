import { compileCode, createVm, runCode } from "../src/vm";
import { createSandbox } from "../src/sandbox";
import { VM, VMScript } from "vm2";

describe("createVm", () => {
  test("creates a new vm for execution", () => {
    const sandbox = createSandbox("resolver");
    const vm = createVm("resolver", sandbox);
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
    const sandbox = createSandbox("resolver");
    const vm = createVm("resolver", sandbox);
    const code = compileCode("'foo'");
    const result = runCode(vm, code);
    expect(result).toBe("foo");
  });
});
