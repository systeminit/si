import { FunctionKind } from "./function.ts";
import { rawStorage, toJSON } from "./sandbox/requestStorage.ts";
import { Debug } from "./debug.ts";
import { join } from "https://deno.land/std/path/mod.ts";
import { makeConsole } from "./sandbox/console.ts";
import * as _ from "https://deno.land/x/lodash_es@v0.0.2/mod.ts";

const debug = Debug("langJs:execute");

const tempDirCache = new Map<string, string>();
const sandboxBundleCache = new Map<string, string>();
const textDecoder = new TextDecoder();

class TimeoutError extends Error {
  constructor(seconds: number) {
    super(`function timed out after ${seconds} seconds`);
    this.name = "TimeoutError";
  }
}

const executionCodeTemplate = (
  func_kind: FunctionKind,
  execution_id: string,
  code: string,
  handler: string,
  withArg: string,
  storedState: string,
) => `
const sandbox = await import("./sandbox.bundle.js");
const sandboxContext = sandbox.createSandbox("${func_kind}", "${execution_id}");
Object.assign(globalThis, sandboxContext);

const storage = requestStorage.rawStorage();
const storedState = JSON.parse('${storedState}');
if (storedState.env) {
  Object.assign(storage.env, storedState.env);
}
if (storedState.data) {
  Object.assign(storage.data, storedState.data);
}

${code}

try {
  const result = await ${handler}(${withArg});

  const finalState = {
    env: { ...storage.env },
    data: { ...storage.data }
  };

  console.log("__STATE_MARKER__" + JSON.stringify(finalState));
  console.log("__RESULT_MARKER__" + JSON.stringify(result));
} catch (error) {
  console.error(error.message)
}`;

export async function runCode(
  code: string,
  handler: string,
  func_kind: FunctionKind,
  execution_id: string,
  timeout: number,
  with_arg?: Record<string, unknown>,
): Promise<Record<string, unknown>> {
  const console = makeConsole("");

  let tempDir = tempDirCache.get(execution_id);

  if (!tempDir) {
    tempDir = await Deno.makeTempDir({
      prefix: `lang-js-execution-${execution_id}-`,
    });
    tempDirCache.set(execution_id, tempDir);
  }

  // Generate and cache sandbox bundle dynamically
  let bundlePath = sandboxBundleCache.get(execution_id);
  if (!bundlePath) {
    // Import build function and generate bundle dynamically
    const { buildSandbox } = await import("./build.ts");
    bundlePath = join(tempDir, "sandbox.bundle.js");
    await buildSandbox(bundlePath);
    sandboxBundleCache.set(execution_id, bundlePath);
  }

  const mainFile = join(tempDir, "main.ts");

  const executionCode = executionCodeTemplate(
    func_kind,
    execution_id,
    code,
    handler,
    JSON.stringify(with_arg),
    toJSON(),
  );

  debug({ executionCode });

  await Deno.writeTextFile(mainFile, executionCode);

  // Ensure existing env vars are available in the run
  for (const [key, value] of Object.entries(rawStorage().env || {})) {
    Deno.env.set(key, value);
  }

  const command = new Deno.Command("deno", {
    args: [
      "run",
      "--quiet",
      "--allow-all",
      "--unstable-node-globals",
      mainFile,
    ],
    stdout: "piped",
    stderr: "piped",
    cwd: tempDir,
    env: {
      ...Deno.env.toObject(),
      "NO_COLOR": "1",
    },
  });

  const process = command.spawn();

  const timeoutId = setTimeout(() => {
    process.kill();
    throw new TimeoutError(timeout);
  }, timeout * 1000);

  // Handle streams
  const [stdout, stderr] = await Promise.all([
    handleStream(process.stdout.getReader(), console, "stdout"),
    handleStream(process.stderr.getReader(), console, "stderr"),
    process.status,
  ]);

  clearTimeout(timeoutId);

  if (stderr.trim()) {
    throw new Error(stderr.trim());
  }

  return processExecutionOutput(stdout);
}

async function handleStream(
  reader: ReadableStreamDefaultReader<Uint8Array>,
  console: ReturnType<typeof makeConsole>,
  type: "stdout" | "stderr",
): Promise<string> {
  let buffer = "";

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const text = textDecoder.decode(value);
      buffer += text;

      const lines = text.split("\n");
      for (const line of lines) {
        if (line.trim()) {
          // don't log the marker lines
          if (
            type === "stdout" &&
            (line.includes("__STATE_MARKER__") ||
              line.includes("__RESULT_MARKER__"))
          ) {
            continue;
          }

          type === "stdout" ? console.log(line) : console.error(line);
        }
      }
    }
  } finally {
    reader.releaseLock();
  }

  return buffer;
}

function processExecutionOutput(stdoutBuffer: string): Record<string, unknown> {
  const resultMarkerIndex = stdoutBuffer.indexOf("__RESULT_MARKER__");
  const stateMarkerIndex = stdoutBuffer.indexOf("__STATE_MARKER__");

  if (resultMarkerIndex === -1) {
    throw new Error("No output received from function run");
  }

  // Process state if it exists
  if (stateMarkerIndex !== -1) {
    const stateJson = stdoutBuffer.slice(
      stateMarkerIndex + "__STATE_MARKER__".length,
      resultMarkerIndex,
    );
    const state = JSON.parse(stateJson);

    if (state) {
      Object.assign(rawStorage().env, state.env);
      Object.assign(rawStorage().data, state.data);
    }
  }

  return JSON.parse(
    stdoutBuffer.slice(resultMarkerIndex + "__RESULT_MARKER__".length),
  );
}
