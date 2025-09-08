#!/usr/bin/env node

import { mcpTest } from "mcp-jest";
import path from "node:path";
import { fileURLToPath } from "node:url";
import dotenv from "dotenv";
import process from "node:process";

// Load environment variables
dotenv.config();

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// =============================================================================
// TEST CONFIGURATION - Edit this section to add/remove/configure tools
// =============================================================================

const TEST_CONFIG = {
  // Tools to test with their configurations
  tools: {
    "validate-credentials": {},
    "component-list": {},
    "change-set-list": {},
    "change-set-create": {
      args: { changeSetName: "Hello Nick!!" },
    },
    "schema-find": {
      args: { schemaNameOrId: "AWS::Neptune::DBCluster" },
    },
    // Uncomment and configure tools as needed:
    // "component-create": {
    //   args: {
    //     schemaVariantId: "your-schema-variant-id",
    //     componentName: "Test Component"
    //   }
    // },
    // "component-get": {
    //   args: { componentId: "your-component-id" }
    // },
    // "component-update": {
    //   args: {
    //     componentId: "your-component-id",
    //     properties: { /* your property updates */ }
    //   }
    // },
    // "schema-attributes-list": {
    //   args: { schemaVariantId: "your-schema-variant-id" }
    // },
  },

  // Test timeout (in milliseconds)
  timeout: 10000, // Increased to account for schema-find retries

  // Enable verbose output
  verbose: true,
};

// =============================================================================
// TEST EXECUTION - Don't modify below this line unless you know what you're doing
// =============================================================================

async function runTests() {
  console.log("Running SI MCP Server Tests...\n");

  // Check required environment variables
  if (!process.env.SI_API_TOKEN) {
    console.error(
      "❌ SI_API_TOKEN is required. Please set it in your .env file",
    );
    process.exit(1);
  }

  // Display test configuration
  console.log("📋 Test Configuration:");
  const toolNames = Object.keys(TEST_CONFIG.tools);
  console.log(`   Tools: ${toolNames.join(", ")}`);
  console.log(`   Timeout: ${TEST_CONFIG.timeout}ms\n`);

  try {
    // Build tools configuration with proper error detection
    const toolsConfig = {};
    Object.keys(TEST_CONFIG.tools).forEach((toolName) => {
      const toolConfig = TEST_CONFIG.tools[toolName];

      // Clean expectation function for error detection
      const expectFunction = function (result) {
        // Check for errors first
        if (
          result?.isError ||
          result?.structuredContent?.status === "failure"
        ) {
          const errorMsg =
            result.structuredContent?.errorMessage ||
            "Tool returned error response";
          throw new Error(errorMsg);
        }

        // Check for success
        if (result?.structuredContent?.status === "success") {
          return true;
        }

        // Default to success for connection/capability tests
        return true;
      };

      toolsConfig[toolName] = {
        args: toolConfig.args,
        expect: expectFunction,
      };
    });

    // Test with real Deno server using wrapper script
    const results = await mcpTest(
      {
        command: path.join(__dirname, "test-server.sh"),
        args: [],
      },
      {
        tools: toolsConfig,
        timeout: TEST_CONFIG.timeout,
      },
    );

    console.log(
      `\n🎯 Test Results: ${results.passed}/${results.total} tests passed`,
    );

    // Show individual test results summary
    if (results.results && Array.isArray(results.results)) {
      console.log("\n=== Test Summary ===");
      results.results.forEach((result) => {
        const status = result.status === "pass" ? "✅" : "❌";
        const duration = result.duration ? ` (${result.duration}ms)` : "";
        console.log(`${status} ${result.name}${duration}`);
      });
    }

    if (results.passed === results.total) {
      console.log("\n✅ All tests passed!");
      process.exit(0);
    } else {
      console.log("\n❌ Some tests failed");
      process.exit(1);
    }
  } catch (error) {
    console.error("Test execution failed:", error.message);
    console.error(error.stack);
    process.exit(1);
  }
}

runTests();
