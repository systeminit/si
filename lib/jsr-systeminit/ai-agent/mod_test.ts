/**
 * Main test entry point for the AI Agent
 *
 * This file serves as a central entry point for all tests. Individual test files
 * for each module are located in the src/ directory with _test.ts suffix.
 *
 * Note: All tests require OPENAI_API_KEY environment variable to be set.
 */

// Import tests from specific modules to ensure they're included in test runs
import "./src/client_test.ts";
import "./src/editComponent_test.ts";
import "./src/extractFields_test.ts";

// Check if we can run the OpenAI-dependent tests
if (!Deno.env.has("OPENAI_API_KEY")) {
  console.log("You cannot run the test suite without OPENAI_API_KEY set");
}
