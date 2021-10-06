import { compileCode, createVm, runCode } from "../src/vm";
import { VM, VMScript } from "vm2";

describe("createVm", () => {
  test("creates a new vm for execution", () => {
    const vm = createVm("resolver");
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
    const vm = createVm("resolver");
    const code = compileCode("'foo'");
    const result = runCode(vm, code);
    expect(result).toBe("foo");
  });

  describe("sandbox", function () {
    test("has debug", () => {
      const vm = createVm("resolver");
      const code = compileCode("debug('poop canoe'); 'foo'");
      const result = runCode(vm, code);
      expect(result).toBe("foo");
    });
  });
});
