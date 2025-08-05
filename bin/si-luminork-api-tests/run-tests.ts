#!/usr/bin/env -S deno run --allow-net --allow-env --allow-read

/**
 * Command-line script to run tests with parameters
 *
 * Usage:
 *   deno run -A run-tests.ts --help
 *   deno run -A run-tests.ts --api-url=http://localhost:5380 --auth-token=YOUR_TOKEN --workspace-id=YOUR_WORKSPACE_ID
 */

import { parse } from "https://deno.land/std@0.220.1/flags/mod.ts";

// Function to print usage and exit
function printUsage() {
  console.log(`
Luminork API Test Runner

Usage:
  deno run -A run-tests.ts [options]

Options:
  --help                      Show this help message
  --api-url=<url>             API URL (fallback option, environment vars take precedence)
  --auth-token=<token>        Auth token (fallback option, environment vars take precedence)
  --workspace-id=<id>         Workspace ID (fallback option, environment vars take precedence)
  --timeout=<ms>              Request timeout in milliseconds (fallback option)
  --tests=<glob>              Specific test files to run (default: "./tests/**/*.test.ts")

Environment variables (take precedence over CLI params):
  LUMINORK_API_URL or API_URL
  LUMINORK_AUTH_TOKEN or AUTH_TOKEN
  LUMINORK_WORKSPACE_ID or WORKSPACE_ID
  LUMINORK_TIMEOUT

Example:
  deno run -A run-tests.ts --api-url=http://localhost:5380 --auth-token=my-token --workspace-id=my-workspace
  deno run -A run-tests.ts --tests="./tests/components.test.ts"
`);
  Deno.exit(0);
}

// Parse command line arguments
const args = parse(Deno.args, {
  string: ['api-url', 'auth-token', 'workspace-id', 'timeout', 'tests'],
  boolean: ['help'],
  alias: {
    'h': 'help',
    'a': 'api-url',
    't': 'auth-token',
    'w': 'workspace-id',
    'T': 'timeout'
  }
});

// Show help if requested
if (args.help) {
  printUsage();
}

// Set CLI parameters as variables that will be picked up by tests
if (args['api-url']) {
  console.log(`Setting API URL from CLI parameter: ${args['api-url']}`);
  Deno.env.set('CLI_API_URL', args['api-url']);
}

if (args['auth-token']) {
  console.log(`Setting Auth Token from CLI parameter (value hidden)`);
  Deno.env.set('CLI_AUTH_TOKEN', args['auth-token']);
}

if (args['workspace-id']) {
  console.log(`Setting Workspace ID from CLI parameter: ${args['workspace-id']}`);
  Deno.env.set('CLI_WORKSPACE_ID', args['workspace-id']);
}

if (args['timeout']) {
  console.log(`Setting timeout from CLI parameter: ${args['timeout']}ms`);
  Deno.env.set('CLI_TIMEOUT', args['timeout']);
}

// Determine test files to run
const testFiles = args.tests || "./tests/**/*.test.ts";
console.log(`Running test files matching: ${testFiles}`);

// Run the tests
const testProcess = Deno.run({
  cmd: [
    "deno",
    "test",
    "--allow-env",
    "--allow-net",
    "--allow-read",
    testFiles
  ],
  stdout: "inherit",
  stderr: "inherit"
});

// Wait for the tests to complete
const status = await testProcess.status();

// Use exit code "53" for test failures
const exitCode = status.success ? 0 : 53;
Deno.exit(exitCode);
