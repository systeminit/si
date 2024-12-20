import { VM, VMScript } from "vm2";

import { Sandbox } from "./sandbox.ts";

export function createVm(sandbox: Sandbox): VM {
  const timeout = 10000;
  const fixAsync = true;
  return new VM({
    timeout,
    sandbox,
    eval: false,
    wasm: false,
    fixAsync,
  });
}

export function compileCode(source: string): VMScript {
  const code = new VMScript(source);
  code.compile();
  return code;
}
