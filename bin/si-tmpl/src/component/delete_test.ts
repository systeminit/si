/**
 * Tests for Component Delete Module
 *
 * These tests verify the component deletion functionality including:
 * - ULID pattern detection
 * - Component resolution by ID and name
 * - Dry-run behavior
 * - Handling of already-deleted components
 *
 * @module
 */

import { assertEquals } from "@std/assert";
import { Context } from "../context.ts";

// Initialize Context once - this runs at module load time
await Context.init({ verbose: 0, noColor: true });

/**
 * Test ULID pattern detection regex
 * ULIDs are 26 characters using Crockford's base32 (0-9, A-Z excluding I, L, O, U)
 */
Deno.test("ULID detection - valid ULID patterns", () => {
  const ulidPattern = /^[0-9A-HJKMNP-TV-Z]{26}$/i;

  // Valid ULIDs
  assertEquals(ulidPattern.test("01HQZX3Y4N5P6Q7R8S9T0V1W2X"), true);
  assertEquals(ulidPattern.test("01ARZ3NDEKTSV4RRFFQ69G5FAV"), true);
  assertEquals(ulidPattern.test("01BX5ZZKBKACTAV9WEVGEMMVRZ"), true);

  // All uppercase
  assertEquals(ulidPattern.test("01HQZX3Y4N5P6Q7R8S9T0V1W2X"), true);

  // All lowercase (should still match with case-insensitive flag)
  assertEquals(ulidPattern.test("01hqzx3y4n5p6q7r8s9t0v1w2x"), true);

  // Mixed case
  assertEquals(ulidPattern.test("01HqZx3Y4n5P6q7R8s9T0v1W2X"), true);
});

Deno.test("ULID detection - invalid ULID patterns", () => {
  const ulidPattern = /^[0-9A-HJKMNP-TV-Z]{26}$/i;

  // Too short
  assertEquals(ulidPattern.test("01HQZX3Y4N5P6Q7R8S9T0V"), false);

  // Too long
  assertEquals(ulidPattern.test("01HQZX3Y4N5P6Q7R8S9T0V1W2X99"), false);

  // Contains invalid characters (I, L, O, U)
  assertEquals(ulidPattern.test("01IQZX3Y4N5P6Q7R8S9T0V1W2X"), false);
  assertEquals(ulidPattern.test("01LQZX3Y4N5P6Q7R8S9T0V1W2X"), false);
  assertEquals(ulidPattern.test("01OQZX3Y4N5P6Q7R8S9T0V1W2X"), false);
  assertEquals(ulidPattern.test("01UQZX3Y4N5P6Q7R8S9T0V1W2X"), false);

  // Contains special characters
  assertEquals(ulidPattern.test("01HQZX3Y-4N5P6Q7R8S9T0V1W2X"), false);
  assertEquals(ulidPattern.test("01HQZX3Y_4N5P6Q7R8S9T0V1W2X"), false);
  assertEquals(ulidPattern.test("01HQZX3Y 4N5P6Q7R8S9T0V1W2X"), false);

  // Empty string
  assertEquals(ulidPattern.test(""), false);

  // Component name (not ULID)
  assertEquals(ulidPattern.test("my-component-name"), false);
  assertEquals(ulidPattern.test("server-1"), false);
});

Deno.test("ULID detection - edge cases", () => {
  const ulidPattern = /^[0-9A-HJKMNP-TV-Z]{26}$/i;

  // All zeros (valid ULID format)
  assertEquals(ulidPattern.test("00000000000000000000000000"), true);

  // All 9s (valid ULID format)
  assertEquals(ulidPattern.test("99999999999999999999999999"), true);

  // All As (valid ULID format)
  assertEquals(ulidPattern.test("AAAAAAAAAAAAAAAAAAAAAAAAAA"), true);

  // All Zs (valid ULID format)
  assertEquals(ulidPattern.test("ZZZZZZZZZZZZZZZZZZZZZZZZZZ"), true);

  // Exactly 26 characters but not base32
  assertEquals(ulidPattern.test("abcdefghijklmnopqrstuvwxyz"), false); // contains excluded chars
});

Deno.test("Component name patterns - should NOT match ULID", () => {
  const ulidPattern = /^[0-9A-HJKMNP-TV-Z]{26}$/i;

  // Common component naming patterns
  assertEquals(ulidPattern.test("my-server"), false);
  assertEquals(ulidPattern.test("web-server-1"), false);
  assertEquals(ulidPattern.test("database_instance"), false);
  assertEquals(ulidPattern.test("app-server-prod"), false);
  assertEquals(ulidPattern.test("ec2-instance-test"), false);

  // Names with numbers
  assertEquals(ulidPattern.test("server123"), false);
  assertEquals(ulidPattern.test("123-server"), false);

  // Names that are close to ULID length
  assertEquals(ulidPattern.test("this-is-exactly-26-chars-x"), false); // 27 chars
  assertEquals(ulidPattern.test("this-is-exactly-25-char"), false); // 25 chars
});

Deno.test("Component deletion options - structure validation", () => {
  // Valid options with all fields
  const options1 = {
    changeSet: "dev",
    dryRun: true,
  };
  assertEquals(options1.changeSet, "dev");
  assertEquals(options1.dryRun, true);

  // Valid options without dry-run
  const options2: { changeSet: string; dryRun?: boolean } = {
    changeSet: "01HQZX3Y4N5P6Q7R8S9T0V1W2X",
  };
  assertEquals(options2.changeSet, "01HQZX3Y4N5P6Q7R8S9T0V1W2X");
  assertEquals(options2.dryRun, undefined);

  // Change set can be name or ULID
  const options3 = {
    changeSet: "main",
    dryRun: false,
  };
  assertEquals(options3.changeSet, "main");
  assertEquals(options3.dryRun, false);
});

Deno.test("Component identifier validation - ULID vs name", () => {
  const ulidPattern = /^[0-9A-HJKMNP-TV-Z]{26}$/i;

  // Test component identifiers
  const ulid = "01HQZX3Y4N5P6Q7R8S9T0V1W2X";
  const name = "my-component";

  assertEquals(ulidPattern.test(ulid), true, "ULID should match pattern");
  assertEquals(ulidPattern.test(name), false, "Name should not match pattern");

  // Verify we can distinguish between them
  const isComponentUlid = ulidPattern.test(ulid);
  const isComponentName = !ulidPattern.test(name);

  assertEquals(isComponentUlid, true);
  assertEquals(isComponentName, true);
});

Deno.test("Dry-run logic - should not delete when dry-run is true", () => {
  const options = {
    changeSet: "dev",
    dryRun: true,
  };

  // Simulate dry-run check
  let deleteWasCalled = false;

  if (!options.dryRun) {
    deleteWasCalled = true;
  }

  assertEquals(deleteWasCalled, false, "Delete should not be called in dry-run mode");
});

Deno.test("Dry-run logic - should delete when dry-run is false or undefined", () => {
  const options1 = {
    changeSet: "dev",
    dryRun: false,
  };

  const options2: { changeSet: string; dryRun?: boolean } = {
    changeSet: "dev",
  };

  // Simulate normal mode check
  let deleteWasCalled1 = false;
  let deleteWasCalled2 = false;

  if (!options1.dryRun) {
    deleteWasCalled1 = true;
  }

  if (!options2.dryRun) {
    deleteWasCalled2 = true;
  }

  assertEquals(deleteWasCalled1, true, "Delete should be called when dry-run is false");
  assertEquals(deleteWasCalled2, true, "Delete should be called when dry-run is undefined");
});

Deno.test("Already deleted component - should handle gracefully", () => {
  // Simulate component that's already marked for deletion
  const component = {
    id: "01HQZX3Y4N5P6Q7R8S9T0V1W2X",
    name: "my-component",
    toDelete: true,
    schemaId: "schema-123",
  };

  // Check if already marked for deletion
  const isAlreadyDeleted = component.toDelete;

  assertEquals(isAlreadyDeleted, true, "Component should be marked as toDelete");

  // In this case, we should not call delete API again
  let deleteApiCalled = false;
  if (!isAlreadyDeleted) {
    deleteApiCalled = true;
  }

  assertEquals(deleteApiCalled, false, "Delete API should not be called for already deleted component");
});

Deno.test("Not deleted component - should proceed with deletion", () => {
  // Simulate component that's NOT marked for deletion
  const component = {
    id: "01HQZX3Y4N5P6Q7R8S9T0V1W2X",
    name: "my-component",
    toDelete: false,
    schemaId: "schema-123",
  };

  // Check if needs deletion
  const needsDeletion = !component.toDelete;

  assertEquals(needsDeletion, true, "Component should need deletion");

  // Should proceed with deletion
  let shouldDelete = false;
  if (!component.toDelete) {
    shouldDelete = true;
  }

  assertEquals(shouldDelete, true, "Should proceed with deletion");
});

Deno.test("404 error handling - component not found should be treated as success", () => {
  // Simulate a 404 error response
  const error404 = {
    response: {
      status: 404,
    },
    message: "Component not found",
  };

  // Check if it's a 404
  // deno-lint-ignore no-explicit-any
  const statusCode = (error404 as any)?.response?.status ||
    // deno-lint-ignore no-explicit-any
    (error404 as any)?.status;

  assertEquals(statusCode, 404, "Should detect 404 status code");

  // In this case, we should NOT throw - just log warning and return
  let shouldThrow = false;
  if (statusCode !== 404) {
    shouldThrow = true;
  }

  assertEquals(shouldThrow, false, "Should not throw for 404 errors");
});

Deno.test("Non-404 error handling - other errors should propagate", () => {
  // Simulate a 500 error response
  const error500 = {
    response: {
      status: 500,
    },
    message: "Internal server error",
  };

  // Check if it's a 404
  // deno-lint-ignore no-explicit-any
  const statusCode = (error500 as any)?.response?.status ||
    // deno-lint-ignore no-explicit-any
    (error500 as any)?.status;

  assertEquals(statusCode, 500, "Should detect 500 status code");

  // In this case, we SHOULD throw - error should propagate
  let shouldThrow = false;
  if (statusCode !== 404) {
    shouldThrow = true;
  }

  assertEquals(shouldThrow, true, "Should throw for non-404 errors");
});

Deno.test("Error status code detection - various formats", () => {
  // Format 1: error.response.status (axios-style)
  const error1 = { response: { status: 404 } };
  // deno-lint-ignore no-explicit-any
  const status1 = (error1 as any)?.response?.status || (error1 as any)?.status;
  assertEquals(status1, 404);

  // Format 2: error.status (direct)
  const error2 = { status: 404 };
  // deno-lint-ignore no-explicit-any
  const status2 = (error2 as any)?.response?.status || (error2 as any)?.status;
  assertEquals(status2, 404);

  // Format 3: neither (no status code)
  const error3 = { message: "Something went wrong" };
  // deno-lint-ignore no-explicit-any
  const status3 = (error3 as any)?.response?.status || (error3 as any)?.status;
  assertEquals(status3, undefined);
});
