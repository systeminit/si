import { createSandbox } from "./sandbox.ts";
import { rawStorage } from "./sandbox/requestStorage.ts";

class TimeoutError extends Error {
  constructor(seconds) {
    super(`function timed out after ${seconds} seconds`);
    this.name = "TimeoutError";
  }
}

self.onmessage = async (event) => {
  const { bundledCode, func_kind, execution_id, with_arg, storage, timeout } =
    event.data ||
    {};

  const sandbox = createSandbox(func_kind, execution_id);
  const keys = Object.keys(sandbox);
  const values = Object.values(sandbox);

  if (storage) {
    Object.assign(rawStorage(), storage);
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
        try {
          return await run(with_arg);
        } catch (e) {
          return {
            err: {
              name: e.name,
              message: e.message
            }
          };
        }
      })()
    `,
  );

  const timeoutId = setTimeout(() => {
    throw new TimeoutError(timeout);
  }, timeout * 1000);

  try {
    const result = await func(...values, with_arg);
    clearTimeout(timeoutId);
    self.postMessage({
      result,
      storage: rawStorage(),
    });
  } catch (e) {
    clearTimeout(timeoutId);
    throw e;
  }
};
