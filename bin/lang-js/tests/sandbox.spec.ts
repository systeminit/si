import { FunctionKind } from "../src/function";
import { createSandbox } from "../src/sandbox";

describe("createSandbox", () => {
  test("creates a new sandbox environment for execution", () => {
    const sandbox = createSandbox(FunctionKind.ResourceSync, "poop");
    expect(sandbox).toHaveProperty("console");
    expect(sandbox).toHaveProperty("_");
  });
});
