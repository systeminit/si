// worker.js
import { createSandbox } from "./sandbox.ts";
import { Debug } from "./debug.ts";

const debug = Debug("langjs:worker");

self.onmessage = async (event) => {
  const { bundledCode, func_kind, execution_id, with_arg } = event.data || {};

  const sandbox = createSandbox(func_kind, execution_id);
  const keys = Object.keys(sandbox);
  const values = Object.values(sandbox);

  debug({ "bundledCode": bundledCode });
  try {
    const func = new Function(
      ...keys,
      "with_arg",
      `
      return (async () => {
        ${bundledCode}
        return await main(with_arg);
      })()
    `,
    );

    const result = await func(...values, with_arg);
    self.postMessage(result);
  } catch (err) {
    self.postMessage({
      error: err.message,
      stack: err.stack,
    });
  }
};
