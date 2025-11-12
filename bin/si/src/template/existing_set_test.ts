import { assertEquals } from "@std/assert";
import { Context } from "../context.ts";
import { TemplateContext } from "./context.ts";
import { queryExistingSet } from "./existing_set.ts";

// Clear SI_API_TOKEN to ensure clean test environment
Deno.env.delete("SI_API_TOKEN");

// Initialize context for tests
await Context.init({ verbose: 0, noColor: true });

Deno.test("queryExistingSet() - returns empty array when API not configured", async () => {
  const tctx = new TemplateContext("test-template", {
    key: "test-key",
  });

  // Don't configure API (apiConfig will be undefined)
  const result = await queryExistingSet(tctx, "changeset-id");

  assertEquals(result, []);
});

// Note: Additional tests for queryExistingSet() require a live API or mocking framework.
// The following scenarios should be tested manually or with integration tests:
// - Returns empty array when no components found
// - Successfully parses components with template tags
// - Skips components missing templateWorkingSetId tag
// - Filters attributes correctly using filterComponentAttributes()
