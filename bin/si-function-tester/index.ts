/**
 * SI Test Framework
 *
 * A testing framework for SI action functions and other function types.
 *
 * @example
 * ```typescript
 * import { defineTests, mockExec, expect } from "@si/test-framework";
 *
 * export default defineTests({
 *   "creates resource successfully": {
 *     input: {
 *       properties: { name: "my-instance" }
 *     },
 *     mocks: {
 *       exec: mockExec()
 *         .command("aws ec2 create-instance")
 *         .returns({ stdout: '{"instanceId": "i-123"}', exitCode: 0 })
 *     },
 *     expect: {
 *       status: "ok",
 *       payload: { instanceId: "i-123" }
 *     }
 *   }
 * });
 * ```
 *
 * @module
 */

export type {
  ActionExpectation,
  ActionResult,
  ExecResponse,
  Matcher,
  MockConfig,
  TestCase,
  TestResult,
  TestSuite,
} from "./types.ts";

export { expect, mockExec } from "./types.ts";
export { runTestSuite } from "./runner.ts";

/**
 * Define a test suite for an action function
 *
 * This is a convenience function that provides type checking
 * and a clear API for defining tests.
 */
import type { TestSuite } from "./types.ts";

export function defineTests(tests: TestSuite): TestSuite {
  return tests;
}
