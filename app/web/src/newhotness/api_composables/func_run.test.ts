import { describe, expect, test } from "vitest";
import { funcRunStatus, FuncRun, FuncKind, FuncBackendKind, FuncBackendResponseType } from "./func_run";
import { ManagementState } from "./management_func_job_state";

/**
 * Creates a minimal FuncRun object for testing.
 * Only includes fields required by funcRunStatus logic.
 */
function createFuncRun(overrides: Partial<FuncRun> = {}): FuncRun {
  return {
    id: "test-func-run-id",
    state: "Success",
    backendKind: FuncBackendKind.JsAttribute,
    backendResponseType: FuncBackendResponseType.Qualification,
    functionName: "test-function",
    functionKind: FuncKind.Qualification,
    functionArgsCasAddress: "test-args-hash",
    functionCodeCasAddress: "test-code-hash",
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    functionArgs: {},
    functionCodeBase64: "",
    resultValue: null,
    unprocessedResultValue: null,
    ...overrides,
  };
}

describe("funcRunStatus", () => {
  describe("null/undefined handling", () => {
    test("returns null when funcRun is undefined", () => {
      const result = funcRunStatus(undefined);
      expect(result).toBeNull();
    });
  });

  describe("management state handling", () => {
    test("returns Running when managementState is executing", () => {
      const funcRun = createFuncRun();
      const result = funcRunStatus(funcRun, "executing");
      expect(result).toBe("Running");
    });

    test("returns Running when managementState is operating", () => {
      const funcRun = createFuncRun();
      const result = funcRunStatus(funcRun, "operating");
      expect(result).toBe("Running");
    });

    test("returns Running when managementState is pending", () => {
      const funcRun = createFuncRun();
      const result = funcRunStatus(funcRun, "pending");
      expect(result).toBe("Running");
    });
  });

  describe("qualification function result handling", () => {
    test("returns Failure when qualification result is failure", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Qualification,
        state: "Success",
        unprocessedResultValue: { result: "failure", message: "Validation failed" },
      });

      const result = funcRunStatus(funcRun);
      expect(result).toBe("Failure");
    });

    test("returns Warning when qualification result is warning", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Qualification,
        state: "Success",
        unprocessedResultValue: { result: "warning", message: "cfn-lint found 4 warnings" },
      });

      const result = funcRunStatus(funcRun);
      expect(result).toBe("Warning");
    });

    test("returns Success when qualification result is success", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Qualification,
        state: "Success",
        unprocessedResultValue: { result: "success", message: "All checks passed" },
      });

      const result = funcRunStatus(funcRun);
      expect(result).toBe("Success");
    });

    test("returns Success when qualification has no unprocessedResultValue", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Qualification,
        state: "Success",
        unprocessedResultValue: null,
      });

      const result = funcRunStatus(funcRun);
      expect(result).toBe("Success");
    });

    test("does not check result when qualification state is not Success", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Qualification,
        state: "Running",
        unprocessedResultValue: { result: "failure" },
      });

      const result = funcRunStatus(funcRun);
      expect(result).toBe("Running");
    });
  });

  describe("management function handling", () => {
    test("returns Failure when management state is failure", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Management,
        state: "Success",
      });

      const result = funcRunStatus(funcRun, "failure");
      expect(result).toBe("Failure");
    });

    test("returns Failure when management actionResultState is Failure", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Management,
        state: "Success",
        actionResultState: "Failure",
      });

      const result = funcRunStatus(funcRun);
      expect(result).toBe("Failure");
    });

    test("returns Failure when management resultValue health is error", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Management,
        state: "Success",
        resultValue: { health: "error" },
      });

      const result = funcRunStatus(funcRun);
      expect(result).toBe("Failure");
    });

    test("returns Success when management state is success", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Management,
        state: "Success",
      });

      const result = funcRunStatus(funcRun, "success");
      expect(result).toBe("Success");
    });
  });

  describe("action result state handling", () => {
    test("returns ActionFailure when actionResultState is Failure", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Action,
        state: "Success",
        actionResultState: "Failure",
      });

      const result = funcRunStatus(funcRun);
      expect(result).toBe("ActionFailure");
    });
  });

  describe("default state handling", () => {
    test("returns funcRun.state when no special conditions apply", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Attribute,
        state: "Running",
      });

      const result = funcRunStatus(funcRun);
      expect(result).toBe("Running");
    });

    test("returns Failure state directly when funcRun failed", () => {
      const funcRun = createFuncRun({
        functionKind: FuncKind.Attribute,
        state: "Failure",
      });

      const result = funcRunStatus(funcRun);
      expect(result).toBe("Failure");
    });
  });
});
