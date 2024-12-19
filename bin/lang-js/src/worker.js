import { createSandbox } from "./sandbox.ts";
import { rawStorage } from "./sandbox/requestStorage.ts";

self.onmessage = async (event) => {
  const { bundledCode, func_kind, execution_id, with_arg, env } = event.data || {};

  const sandbox = createSandbox(func_kind, execution_id);
  const keys = Object.keys(sandbox);
  const values = Object.values(sandbox);

  try {
    if (env) {
      Object.assign(rawStorage().env, env);
      for (const [key, value] of Object.entries(rawStorage().env || {})) {
        Deno.env.set(key, value);
      }
    }

    const func = new Function(
      ...keys,
      "with_arg",
      `
      return (async () => {
        ${bundledCode}
        return await run(with_arg);
      })()
    `,
    );

    const result = await func(...values, with_arg);
    self.postMessage({
      result,
      env: rawStorage().env,
    });
  } catch (err) {
    self.postMessage({
      error: err.message,
      stack: err.stack,
    });
  }
};
