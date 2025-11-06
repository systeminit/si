/**
 * Mock Utilities
 *
 * Provides mock implementations for sandbox APIs that can be
 * injected during test execution.
 */

import type { Options, Result } from "npm:execa";
import type { ExecMock, ExecMockBuilder } from "./types.ts";

/**
 * Create a mocked siExec that matches patterns and returns configured responses
 */
export function createMockedExec(
  mockBuilder?: ExecMockBuilder,
  _executionId?: string,
) {
  const mocks = mockBuilder?.getMocks() || [];

  function waitUntilEnd(
    execaFile: string,
    execaArgs?: readonly string[],
    _execaOptions?: Options,
  ): Promise<Result> {
    const fullCommand = `${execaFile} ${execaArgs?.join(" ") || ""}`.trim();

    console.log(`[MOCK] Running CLI command: "${fullCommand}"`);

    // Find matching mock
    for (const mock of mocks) {
      if (matchesPattern(fullCommand, mock.pattern)) {
        if (mock.error) {
          console.log(`[MOCK] Throwing error: ${mock.error.message}`);
          throw mock.error;
        }

        if (mock.response) {
          console.log(
            `[MOCK] Returning response (exitCode: ${mock.response.exitCode})`,
          );
          return Promise.resolve(createMockResult(fullCommand, mock));
        }
      }
    }

    // No mock found - log warning and return empty success
    console.warn(
      `[MOCK] No mock found for command: "${fullCommand}". Returning empty success response.`,
    );
    return Promise.resolve(createMockResult(fullCommand, {
      pattern: fullCommand,
      response: { stdout: "", stderr: "", exitCode: 0 },
    }));
  }

  function watch(_options: unknown, _deadlineCount?: number): Promise<{
    result: Result;
    failed?: "deadlineExceeded" | "commandFailed";
  }> {
    console.warn("[MOCK] siExec.watch() is not fully supported in tests yet");
    return Promise.resolve({
      result: createMockResult("watch", {
        pattern: "watch",
        response: { stdout: "", stderr: "", exitCode: 0 },
      }),
    });
  }

  return { waitUntilEnd, watch };
}

/**
 * Check if a command matches a pattern (string or regex)
 */
function matchesPattern(
  command: string,
  pattern: string | RegExp,
): boolean {
  if (typeof pattern === "string") {
    return command.includes(pattern);
  }
  return pattern.test(command);
}

/**
 * Create a mock Result object that matches execa's Result type
 */
function createMockResult(command: string, mock: ExecMock): Result {
  const response = mock.response || { stdout: "", stderr: "", exitCode: 0 };

  return {
    command,
    escapedCommand: command,
    exitCode: response.exitCode,
    stdout: response.stdout,
    stderr: response.stderr || "",
    all: response.stdout + (response.stderr || ""),
    failed: response.exitCode !== 0,
    timedOut: false,
    isCanceled: false,
    killed: false,
    // Additional properties to match execa's Result type
    // deno-lint-ignore no-explicit-any
  } as any as Result;
}

/**
 * Create a mocked requestStorage with pre-populated values
 */
export function createMockedRequestStorage(
  initialData?: Record<string, unknown>,
) {
  const envStorage = new Map<string, unknown>();
  const dataStorage = new Map<string, unknown>();

  // Pre-populate if initial data provided
  if (initialData) {
    Object.entries(initialData).forEach(([key, value]) => {
      dataStorage.set(key, value);
    });
  }

  return {
    setEnv: (key: string, value: unknown) => {
      envStorage.set(key, value);
    },
    getEnv: (key: string) => {
      return envStorage.get(key);
    },
    deleteEnv: (key: string) => {
      envStorage.delete(key);
    },
    setData: (key: string, value: unknown) => {
      dataStorage.set(key, value);
    },
    getData: (key: string) => {
      return dataStorage.get(key);
    },
    // Expose for test inspection
    _inspect: () => ({
      env: Object.fromEntries(envStorage),
      data: Object.fromEntries(dataStorage),
    }),
  };
}
