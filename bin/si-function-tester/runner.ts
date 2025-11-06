/**
 * Test Runner
 *
 * Executes test suites and validates results against expectations.
 */

// Note: These imports appear unused but are used in generated execution code
// deno-lint-ignore-file no-unused-vars verbatim-module-syntax

import { Buffer } from "node:buffer";
import os from "node:os";
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";

import * as _ from "https://deno.land/x/lodash_es@v0.0.2/mod.ts";
import { join } from "https://deno.land/std@0.224.0/path/mod.ts";

import type {
  ActionExpectation,
  ActionResult,
  FunctionResult,
  GenericExpectation,
  Matcher,
  TestCase,
  TestResult,
  TestSuite,
} from "./types.ts";

const DEFAULT_TIMEOUT = 5000;

/**
 * Run a test suite
 */
export async function runTestSuite(
  functionCode: string,
  testSuite: TestSuite,
  options?: { verbose?: boolean },
): Promise<TestResult[]> {
  const results: TestResult[] = [];

  for (const [testName, testCase] of Object.entries(testSuite)) {
    if (testCase.skip) {
      results.push({
        name: testName,
        passed: true,
        duration: 0,
        skipped: true,
      });
      continue;
    }

    const result = await runTestCase(
      testName,
      functionCode,
      testCase,
      options,
    );
    results.push(result);
  }

  return results;
}

/**
 * Run a single test case
 */
async function runTestCase(
  testName: string,
  functionCode: string,
  testCase: TestCase,
  _options?: { verbose?: boolean },
): Promise<TestResult> {
  const startTime = performance.now();

  try {
    // Execute the function with mocked dependencies
    const result = await executeWithMocks(functionCode, testCase);

    // Validate the result
    const validationError = validateResult(result, testCase.expect);

    const duration = performance.now() - startTime;

    if (validationError) {
      return {
        name: testName,
        passed: false,
        error: validationError,
        duration,
      };
    }

    return {
      name: testName,
      passed: true,
      duration,
    };
  } catch (err) {
    const duration = performance.now() - startTime;
    return {
      name: testName,
      passed: false,
      error: `Test execution failed: ${
        err instanceof Error ? err.message : String(err)
      }`,
      duration,
    };
  }
}

/**
 * Execute function with mocked dependencies
 */
async function executeWithMocks(
  functionCode: string,
  testCase: TestCase,
): Promise<FunctionResult> {
  const executionId = crypto.randomUUID();
  const timeout = testCase.timeout || DEFAULT_TIMEOUT;

  // Wrap the function code to match action function format
  const wrappedCode = wrapActionCode(functionCode);

  // Create the execution code with mocked sandbox
  const executionCode = createExecutionCodeWithMocks(
    wrappedCode,
    testCase,
    executionId,
  );

  // Execute in a temp directory
  const tempDir = await Deno.makeTempDir({
    prefix: `si-test-${executionId}-`,
  });

  try {
    const mainFile = join(tempDir, "main.ts");
    await Deno.writeTextFile(mainFile, executionCode);

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

    // Create timeout
    let timeoutId: number | undefined;
    const timeoutPromise = new Promise<never>((_, reject) => {
      timeoutId = setTimeout(() => {
        process.kill();
        reject(new Error(`Test timed out after ${timeout}ms`));
      }, timeout);
    });

    try {
      // Read all stdout and stderr
      const stdoutChunks: Uint8Array[] = [];
      const stderrChunks: Uint8Array[] = [];

      const result = await Promise.race([
        Promise.all([
          (async () => {
            const reader = process.stdout.getReader();
            while (true) {
              const { done, value } = await reader.read();
              if (done) break;
              stdoutChunks.push(value);
            }
          })(),
          (async () => {
            const reader = process.stderr.getReader();
            while (true) {
              const { done, value } = await reader.read();
              if (done) break;
              stderrChunks.push(value);
            }
          })(),
          process.status,
        ]),
        timeoutPromise,
      ]);

      const status = result[2] as Deno.CommandStatus;

      // Combine chunks
      const stdout = new TextDecoder().decode(
        new Uint8Array(stdoutChunks.flatMap((chunk) => Array.from(chunk))),
      );
      const stderr = new TextDecoder().decode(
        new Uint8Array(stderrChunks.flatMap((chunk) => Array.from(chunk))),
      );

      if (!status.success && !stdout.includes("__RESULT_MARKER__")) {
        throw new Error(`Execution failed: ${stderr}`);
      }

      // Extract result from markers
      const resultMatch = stdout.match(/__RESULT_MARKER__(.+)/);
      if (!resultMatch) {
        throw new Error(
          `No result marker found in output.\nStdout: ${stdout}\nStderr: ${stderr}`,
        );
      }

      const rawResult = JSON.parse(resultMatch[1]);

      // Return raw result without filtering - supports all function types:
      // - Actions: { status, payload, resourceId, message }
      // - Management: { status, message, ops }
      // - Codegen: { format, code }
      // - Qualifications: { result, message }
      // - Attributes: primitives or objects
      return rawResult;
    } finally {
      if (timeoutId !== undefined) {
        clearTimeout(timeoutId);
      }
    }
  } finally {
    // Cleanup temp directory
    try {
      await Deno.remove(tempDir, { recursive: true });
    } catch {
      // Ignore cleanup errors
    }
  }
}

/**
 * Create execution code with mocked sandbox
 */
function createExecutionCodeWithMocks(
  code: string,
  testCase: TestCase,
  executionId: string,
): string {
  const input = JSON.stringify(testCase.input);

  // Serialize mock configurations
  const mocksCode = generateMocksCode(testCase.mocks, executionId);

  return `
import { Buffer } from "node:buffer";
import os from "node:os";
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";

import toml from "npm:toml";
import jsonpatch from "npm:fast-json-patch";
import * as _ from "https://deno.land/x/lodash_es@v0.0.2/mod.ts";
import * as yaml from "npm:js-yaml";
import Joi from "npm:joi";

${mocksCode}

// Create sandbox
const requestStorage = createMockedStorage();
const siExec = createMockedExec();
const YAML = { stringify: yaml.dump, parse: yaml.load };

// Make sandbox available globally
Object.assign(globalThis, {
  _,
  Buffer,
  requestStorage,
  zlib,
  siExec,
  YAML,
  os,
  fs,
  path,
  Joi,
  toml,
  jsonpatch,
});

${code}

try {
  const arg = ${input};
  const result = await siMain(arg);

  // Handle void/undefined returns (e.g., from authentication functions)
  const resultToSerialize = result === undefined ? null : result;
  console.log("__RESULT_MARKER__" + JSON.stringify(resultToSerialize));
} catch (error) {
  console.error("__ERROR__" + error.message);
  console.log("__RESULT_MARKER__" + JSON.stringify({
    status: "error",
    payload: null,
    message: error.message
  }));
}
`;
}

/**
 * Generate code for mocks
 */
function generateMocksCode(
  mocks: TestCase["mocks"],
  _executionId: string,
): string {
  const execMocks = mocks?.exec?.getMocks() || [];
  const storageMocks = mocks?.storage || {};

  return `
// Mock configurations
const EXEC_MOCKS = ${
    JSON.stringify(
      execMocks.map((m) => ({
        pattern: m.pattern instanceof RegExp ? m.pattern.source : m.pattern,
        isRegex: m.pattern instanceof RegExp,
        response: m.response,
        error: m.error ? { message: m.error.message } : undefined,
      })),
    )
  };

const STORAGE_INITIAL = ${JSON.stringify(storageMocks)};

function createMockedExec() {
  // Track which mocks have been used for sequential matching
  const mockUsageCount = new Array(EXEC_MOCKS.length).fill(0);

  async function waitUntilEnd(cmd, args = [], _options) {
    const fullCommand = cmd + " " + args.join(" ");
    console.log(\`[MOCK] Running CLI command: "\${fullCommand}"\`);

    // Find all matching mocks and use the first unused one
    for (let i = 0; i < EXEC_MOCKS.length; i++) {
      const mock = EXEC_MOCKS[i];
      const matches = mock.isRegex
        ? new RegExp(mock.pattern).test(fullCommand)
        : fullCommand.includes(mock.pattern);

      if (matches && mockUsageCount[i] === 0) {
        // Mark this mock as used
        mockUsageCount[i] = 1;

        if (mock.error) {
          console.log(\`[MOCK] Throwing error: \${mock.error.message}\`);
          throw new Error(mock.error.message);
        }

        if (mock.response) {
          console.log(\`[MOCK] Returning response (exitCode: \${mock.response.exitCode})\`);
          return {
            command: fullCommand,
            exitCode: mock.response.exitCode,
            stdout: mock.response.stdout || "",
            stderr: mock.response.stderr || "",
            failed: mock.response.exitCode !== 0,
          };
        }
      }
    }

    console.warn(\`[MOCK] No mock found for command: "\${fullCommand}"\`);
    return {
      command: fullCommand,
      exitCode: 0,
      stdout: "",
      stderr: "",
      failed: false,
    };
  }

  async function watch(_options, _deadlineCount) {
    console.warn("[MOCK] siExec.watch() not fully supported in tests");
    return {
      result: { command: "watch", exitCode: 0, stdout: "", stderr: "", failed: false }
    };
  }

  return { waitUntilEnd, watch };
}

function createMockedStorage() {
  const env = new Map();
  const data = new Map(Object.entries(STORAGE_INITIAL));

  return {
    setEnv: (key, value) => env.set(key, value),
    getEnv: (key) => env.get(key),
    deleteEnv: (key) => env.delete(key),
    setData: (key, value) => data.set(key, value),
    getData: (key) => data.get(key),
    setItem: (key, value) => data.set(key, value),
    getItem: (key) => data.get(key),
    deleteItem: (key) => data.delete(key),
    _inspect: () => ({
      env: Object.fromEntries(env),
      data: Object.fromEntries(data),
    }),
  };
}
`;
}

/**
 * Wrap action code to match the format expected by the executor
 */
function wrapActionCode(code: string): string {
  // Check if code already has an export
  const hasExport = /export\s+(default\s+)?function/.test(code);
  const hasMain = /function\s+main\s*\(/.test(code);

  if (hasExport && hasMain) {
    // Code already has proper structure
    return `
${code}

async function siMain(arg) {
  let payload = null;
  let resourceId = null;
  try {
    resourceId = arg?.properties?.si?.resourceId;
    payload = arg?.properties?.resource?.payload ?? null;

    const returnValue = await main(arg);
    return returnValue;
  } catch (err) {
    return {
      status: "error",
      payload,
      resourceId,
      message: err.message,
    }
  }
}
export { siMain };
`;
  }

  // If code doesn't have proper structure, wrap it
  return `
${code}

async function siMain(arg) {
  let payload = null;
  let resourceId = null;
  try {
    resourceId = arg?.properties?.si?.resourceId;
    payload = arg?.properties?.resource?.payload ?? null;

    const returnValue = await main(arg);
    return returnValue;
  } catch (err) {
    return {
      status: "error",
      payload,
      resourceId,
      message: err.message,
    }
  }
}
export { siMain };
`;
}

/**
 * Validate a result against expectations
 */
function validateResult(
  result: FunctionResult,
  expectation: ActionExpectation | GenericExpectation,
): string | null {
  // If this is a GenericExpectation (only has validate), use custom validation
  if ("validate" in expectation && Object.keys(expectation).length === 1) {
    const genericExpectation = expectation as GenericExpectation;
    try {
      const customResult = genericExpectation.validate(result);
      if (customResult === false) {
        return "Custom validation failed";
      }
    } catch (err) {
      return `Custom validation threw error: ${
        err instanceof Error ? err.message : String(err)
      }`;
    }
    return null;
  }

  // Otherwise, treat as ActionExpectation
  const actionExpectation = expectation as ActionExpectation;
  const actionResult = result as Record<string, unknown>;

  // Check status
  if (actionExpectation.status !== undefined) {
    if (
      !matchesValue(
        actionResult.status as "ok" | "warning" | "error",
        actionExpectation.status,
      )
    ) {
      return `Expected status to be ${
        formatExpected(actionExpectation.status)
      }, but got "${actionResult.status}"`;
    }
  }

  // Check message
  if (actionExpectation.message !== undefined) {
    if (
      !matchesValue(
        actionResult.message as string | undefined,
        actionExpectation.message,
      )
    ) {
      return `Expected message to be ${
        formatExpected(actionExpectation.message)
      }, but got ${JSON.stringify(actionResult.message)}`;
    }
  }

  // Check resourceId
  if (actionExpectation.resourceId !== undefined) {
    if (
      !matchesValue(
        actionResult.resourceId as string | null | undefined,
        actionExpectation.resourceId,
      )
    ) {
      return `Expected resourceId to be ${
        formatExpected(actionExpectation.resourceId)
      }, but got ${JSON.stringify(actionResult.resourceId)}`;
    }
  }

  // Check payload
  if (actionExpectation.payload !== undefined) {
    if (!_.isEqual(actionResult.payload, actionExpectation.payload)) {
      return `Expected payload to be ${
        JSON.stringify(actionExpectation.payload)
      }, but got ${JSON.stringify(actionResult.payload)}`;
    }
  }

  // Run custom validation
  if (actionExpectation.validate) {
    try {
      const customResult = actionExpectation.validate(
        actionResult as unknown as ActionResult,
      );
      if (customResult === false) {
        return "Custom validation failed";
      }
    } catch (err) {
      return `Custom validation threw error: ${
        err instanceof Error ? err.message : String(err)
      }`;
    }
  }

  return null;
}

/**
 * Check if a value matches a matcher (exact value or matcher function)
 */
function matchesValue<T>(value: T, matcher: Matcher<T>): boolean {
  if (typeof matcher === "function") {
    try {
      const matcherFn = matcher as (value: T) => boolean | void;
      const result = matcherFn(value);
      return result !== false;
    } catch {
      return false;
    }
  }

  return _.isEqual(value, matcher);
}

/**
 * Format expected value for error messages
 */
function formatExpected(matcher: unknown): string {
  if (typeof matcher === "function") {
    return "[matcher function]";
  }
  return JSON.stringify(matcher);
}
