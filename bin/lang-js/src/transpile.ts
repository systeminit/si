import * as _ from "npm:lodash-es";
import { transpile } from "jsr:@deno/emit";
import { Debug } from "./debug.ts";
import { lock } from "npm:proper-lockfile";

const debug = Debug("langJs:transpile");

const LOCK_FILE = "/tmp/lang-js-transpile";

async function ensureLockfile() {
  try {
    await Deno.writeTextFile(LOCK_FILE, "");
  } catch (err) {
    if (!(err instanceof Deno.errors.AlreadyExists)) {
      throw err;
    }
  }
}

export function bundleCode(code: string): Promise<string> {
  return (async () => {
    let release;

    await ensureLockfile();

    try {
      release = await lock("/tmp/lang-js-transpile", {
        stale: 30000,
        updateInterval: 100,
        retries: {
          retries: 60,
          minTimeout: 100,
          maxTimeout: 1000,
        },
      });

      debug({ "code before bundle": code });
      const tempDir = await Deno.makeTempDir();
      const tempFile = `${tempDir}/script.ts`;

      await Deno.writeTextFile(tempFile, code);
      const fileUrl = new URL(tempFile, import.meta.url);

      try {
        const result = await transpile(fileUrl);
        const bundled = result.get(fileUrl.href) as string;
        if (!bundled) {
          throw new Error("Transpilation resulted in empty output");
        }
        debug({ "code after bundle": bundled });
        return bundled;
      } finally {
        await Deno.remove(tempDir, { recursive: true });
      }
    } finally {
      if (release) {
        await release();
      }
    }
  })();
}
