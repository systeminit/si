#!/usr/bin/env -S deno run --allow-read --allow-net

/**
 * SI Test Framework Client
 *
 * Submits tests to the running test server and displays results.
 * Much faster than spinning up containers.
 */

import { parse } from "https://deno.land/std@0.224.0/flags/mod.ts";

const RESET = "\x1b[0m";
const GREEN = "\x1b[32m";
const RED = "\x1b[31m";
const YELLOW = "\x1b[33m";
const BOLD = "\x1b[1m";
const GRAY = "\x1b[90m";

interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
  duration: number;
  skipped?: boolean;
}

interface TestResponse {
  success: boolean;
  results?: TestResult[];
  error?: string;
  duration: number;
}

async function main() {
  const args = parse(Deno.args, {
    string: ["server"],
    boolean: ["help", "json"],
    alias: { h: "help", s: "server", j: "json" },
    default: {
      server: "http://localhost:8081",
    },
  });

  if (args.help || args._.length < 2) {
    printHelp();
    Deno.exit(0);
  }

  const functionFile = args._[0] as string;
  const testFile = args._[1] as string;
  const serverUrl = args.server;

  try {
    // Read files
    const functionCode = await Deno.readTextFile(functionFile);
    const testCode = await Deno.readTextFile(testFile);

    // Submit to server
    if (!args.json) {
      console.log(`${BOLD}Submitting tests to ${serverUrl}...${RESET}\n`);
    }

    const response = await fetch(`${serverUrl}/test`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        functionCode,
        testCode,
      }),
    });

    if (!response.ok) {
      throw new Error(
        `Server returned ${response.status}: ${response.statusText}`,
      );
    }

    const result: TestResponse = await response.json();

    // Display results
    if (args.json) {
      console.log(JSON.stringify(result, null, 2));
    } else {
      displayResults(result);
    }

    // Exit with appropriate code
    Deno.exit(result.success ? 0 : 1);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`${RED}${BOLD}Error:${RESET} ${message}`);
    Deno.exit(1);
  }
}

function displayResults(response: TestResponse) {
  if (response.error) {
    console.error(`${RED}${BOLD}Error:${RESET} ${response.error}`);
    return;
  }

  if (!response.results) {
    console.error(`${RED}${BOLD}Error:${RESET} No results returned`);
    return;
  }

  let passed = 0;
  let failed = 0;
  let skipped = 0;

  for (const result of response.results) {
    if (result.skipped) {
      skipped++;
      console.log(
        `  ${YELLOW}○${RESET} ${GRAY}${result.name} (skipped)${RESET}`,
      );
    } else if (result.passed) {
      passed++;
      console.log(
        `  ${GREEN}✓${RESET} ${result.name} ${GRAY}(${
          result.duration.toFixed(0)
        }ms)${RESET}`,
      );
    } else {
      failed++;
      console.log(`  ${RED}✗${RESET} ${result.name}`);
      if (result.error) {
        console.log(`    ${RED}${result.error}${RESET}`);
      }
    }
  }

  // Print summary
  console.log();
  console.log(
    `${BOLD}Test Results:${RESET} ${GREEN}${passed} passed${RESET}, ${RED}${failed} failed${RESET}${
      skipped > 0 ? `, ${YELLOW}${skipped} skipped${RESET}` : ""
    }`,
  );
  console.log(
    `${GRAY}Total duration: ${response.duration.toFixed(0)}ms${RESET}`,
  );

  if (failed > 0) {
    console.log(`\n${RED}${BOLD}Tests failed!${RESET}`);
  } else {
    console.log(`\n${GREEN}${BOLD}All tests passed!${RESET}`);
  }
}

function printHelp() {
  console.log(`
${BOLD}SI Test Framework Client${RESET}

Submit tests to a running test server.

${BOLD}USAGE:${RESET}
  client.ts [OPTIONS] <function-file> <test-file>

${BOLD}ARGUMENTS:${RESET}
  <function-file>    Path to the function file
  <test-file>        Path to the test file

${BOLD}OPTIONS:${RESET}
  -h, --help              Show this help message
  -s, --server <URL>      Test server URL (default: http://localhost:8081)
  -j, --json              Output results as JSON

${BOLD}EXAMPLES:${RESET}
  # Submit tests to local server
  ./client.ts function.ts function.test.ts

  # Submit to remote server
  ./client.ts -s http://test-server:8081 function.ts test.ts

  # Get JSON output
  ./client.ts --json function.ts test.ts

${BOLD}SERVER SETUP:${RESET}
  # Start the test server
  docker run -d -p 8081:8081 --name si-function-tester si-function-tester

  # Check server health
  curl http://localhost:8081/health
  `);
}

if (import.meta.main) {
  main();
}
