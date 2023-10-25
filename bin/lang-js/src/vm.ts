import { NodeVM, VM, VMScript } from "vm2";

import { Sandbox } from "./sandbox";

// This is needed to execute functions asynchronously (you have to export a default function for a module)
export function createNodeVm(sandbox: Sandbox): NodeVM {
  return new NodeVM({
    sandbox,
    eval: false,
    wasm: false,
  });
}

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
