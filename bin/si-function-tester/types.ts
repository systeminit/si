/**
 * Test Framework Types
 *
 * Provides type definitions for writing tests for SI functions:
 * - Action functions (create, update, delete, etc.)
 * - Management functions (refresh, import, etc.)
 * - Codegen functions (generate code/payload)
 * - Qualification functions (validation)
 * - Attribute functions (compute values)
 * - Authentication functions (set credentials/env vars)
 */

/**
 * Mock configuration for external calls
 */
export interface MockConfig {
  /** Mock siExec calls */
  exec?: ExecMockBuilder;
  /** Mock requestStorage state */
  storage?: Record<string, unknown>;
}

/**
 * Matcher for expected values - can be exact value or a matcher function
 */
export type Matcher<T> = T | ((value: T) => boolean | void);

/**
 * Action result from function execution
 */
export interface ActionResult {
  status: "ok" | "warning" | "error";
  payload: unknown;
  resourceId?: string | null;
  message?: string;
}

/**
 * Management function result (refresh, import, etc.)
 */
export interface ManagementResult {
  status: "ok" | "error";
  message: string;
  ops?: {
    update?: {
      self?: {
        properties?: Record<string, unknown>;
      };
    };
    actions?: {
      self?: {
        add?: string[];
        remove?: string[];
      };
    };
  };
}

/**
 * Codegen function result (generate code/payload)
 */
export interface CodegenResult {
  format: string;
  code: string;
}

/**
 * Qualification function result (validation)
 */
export interface QualificationResult {
  result: "success" | "failure";
  message: string;
}

/**
 * Attribute function result (compute value)
 */
export type AttributeResult = string | number | boolean | object | null;

/**
 * Authentication function result (returns void, sets env vars)
 */
export type AuthenticationResult = void | undefined | null;

/**
 * All possible function result types
 */
export type FunctionResult =
  | ActionResult
  | ManagementResult
  | CodegenResult
  | QualificationResult
  | AttributeResult
  | AuthenticationResult;

/**
 * Expected output for action functions
 */
export interface ActionExpectation {
  /** Expected status */
  status?: Matcher<"ok" | "warning" | "error">;
  /** Expected payload structure/value */
  payload?: unknown;
  /** Expected resource ID */
  resourceId?: Matcher<string | null | undefined>;
  /** Expected message (required for warning/error) */
  message?: Matcher<string | undefined>;
  /** Custom validation function */
  validate?: (result: ActionResult) => boolean | void;
}

/**
 * Generic expectation for any function type
 * Use validate() for custom validation logic
 */
export interface GenericExpectation {
  /** Custom validation function */
  validate: (result: FunctionResult) => boolean | void;
}

/**
 * A single test case
 */
export interface TestCase {
  /** Input arguments to the function */
  input: {
    properties?: Record<string, unknown>;
    thisComponent?: Record<string, unknown>;
    component?: Record<string, unknown>;
    domain?: Record<string, unknown>;
    [key: string]: unknown;
  };
  /** Mock configurations */
  mocks?: MockConfig;
  /** Expected output - use ActionExpectation for actions, or GenericExpectation with validate() for other types */
  expect: ActionExpectation | GenericExpectation;
  /** Test timeout in milliseconds (default: 5000) */
  timeout?: number;
  /** Skip this test */
  skip?: boolean;
}

/**
 * Collection of test cases
 */
export interface TestSuite {
  [testName: string]: TestCase;
}

/**
 * Test result
 */
export interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
  duration: number;
  skipped?: boolean;
}

/**
 * Exec mock builder for fluent API
 */
export class ExecMockBuilder {
  private mocks: ExecMock[] = [];

  /**
   * Add a mock for a specific command
   */
  command(commandPattern: string | RegExp): ExecMockConfig {
    const mock: ExecMock = {
      pattern: commandPattern,
      response: { stdout: "", stderr: "", exitCode: 0 },
    };
    this.mocks.push(mock);

    return {
      returns: (response: ExecResponse) => {
        mock.response = response;
        return this;
      },
      throws: (error: Error) => {
        mock.error = error;
        return this;
      },
    };
  }

  /**
   * Get all configured mocks
   * @internal
   */
  getMocks(): ExecMock[] {
    return this.mocks;
  }
}

export interface ExecMock {
  pattern: string | RegExp;
  response?: ExecResponse;
  error?: Error;
}

export interface ExecResponse {
  stdout: string;
  stderr?: string;
  exitCode: number;
}

export interface ExecMockConfig {
  returns: (response: ExecResponse) => ExecMockBuilder;
  throws: (error: Error) => ExecMockBuilder;
}

/**
 * Create a new exec mock builder
 */
export function mockExec(): ExecMockBuilder {
  return new ExecMockBuilder();
}

/**
 * Helper matchers
 */
export const expect = {
  /** Match any string */
  string: () => (value: unknown) => typeof value === "string",

  /** Match string containing substring */
  stringContaining: (substring: string) => (value: unknown): boolean =>
    typeof value === "string" && value.includes(substring),

  /** Match string matching regex */
  stringMatching: (regex: RegExp) => (value: unknown): boolean =>
    typeof value === "string" && regex.test(value),

  /** Match any object */
  object: () => (value: unknown) => typeof value === "object" && value !== null,

  /** Match object containing specific keys */
  objectContaining:
    (keys: Record<string, unknown>) => (value: unknown): boolean => {
      if (typeof value !== "object" || value === null) return false;
      const obj = value as Record<string, unknown>;
      return Object.keys(keys).every((key) => key in obj);
    },

  /** Match any number */
  number: () => (value: unknown) => typeof value === "number",

  /** Match any boolean */
  boolean: () => (value: unknown) => typeof value === "boolean",

  /** Match any array */
  array: () => (value: unknown): boolean => Array.isArray(value),

  /** Match null */
  null: () => (value: unknown) => value === null,

  /** Match undefined */
  undefined: () => (value: unknown) => value === undefined,

  /** Match any value (always passes) */
  any: () => () => true,
};
