import { RemoteFunctionRequest } from "./remote_function";
import { VM, VMScript } from "vm2";
import Debug from "debug";
import lodash from "lodash";

export function createVm(kind: RemoteFunctionRequest["kind"]): VM {
  const sandbox: Record<string, any> = {};
  const timeout = 2000;
  const fixAsync = true;
  if (kind == "resolver") {
    sandbox["debug"] = Debug("cyclone:resolver:ts");
    sandbox["_"] = lodash;
  }
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

export function runCode(vm: VM, code: VMScript): unknown {
  return vm.run(code);
}
