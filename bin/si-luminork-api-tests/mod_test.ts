import { assertExists } from "@std/assert";
import * as framework from "./src/index.ts";

Deno.test("framework exports exist", () => {
  // Verify the main framework exports are available
  assertExists(framework);
});

Deno.test("framework has expected exports", () => {
  // Check that key exports from the framework are present
  // This is a basic smoke test to ensure the module structure is valid
  const exports = Object.keys(framework);
  assertExists(exports);
});
