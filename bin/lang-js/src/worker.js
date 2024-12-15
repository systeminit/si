// worker.js
import { createSandbox } from "./sandbox.ts";
import { Debug } from "./debug.ts";

const debug = Debug("langjs:worker");

self.onmessage = async (event) => {
  const { bundledCode, func_kind, execution_id, with_arg } = event.data || {};

  const sandbox = createSandbox(func_kind, execution_id);
  const keys = Object.keys(sandbox);
  const values = Object.values(sandbox);

  debug({"bundledCode": bundledCode})
  try {
    // Create the function with import support
    const func = new Function(
      ...keys,
      "with_arg",
      `
      return (async () => {
        // Try using a bare specifier that Deno might recognize
        // const importDynamic = (specifier) => {
        //   try {
        //     return import(specifier);
        //   } catch (e) {
        //     console.error('Import failed:', e);
        //     throw e;
        //   }
        // };
        // const import_func = importDynamic;
        ${bundledCode}
        return await main(with_arg);
      })()
    `,
    );
    debug(func.toString())

    const result = await func(...values, with_arg);
    debug(result);

    self.postMessage(result);
  } catch (err) {
    self.postMessage({
      error: err.message,
      stack: err.stack,
    });
  }
};
