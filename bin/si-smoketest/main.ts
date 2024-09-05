// main.ts
import assert from "node:assert";
import { SdfApiClient } from "./sdf_api_client.ts";
import { createDefaultTestReportEntry, printTestReport, TestReportEntry } from "./test_execution_lib.ts";
import { parseArgs } from "./binary_exeuction_lib.ts"; 

if (import.meta.main) {
  const { workspaceId, userId, password, testsToRun } = parseArgs(Deno.args);
  
  // Init the SDF Module
  const sdfApiClient = await SdfApiClient.init(workspaceId, userId, password);

  // Dynamically load test files from the ./tests directory
  const testFiles: { [key: string]: string } = {};
  for await (const dirEntry of Deno.readDir("./tests")) {
    if (dirEntry.isFile && dirEntry.name.endsWith(".ts")) {
      const testName = dirEntry.name.replace(".ts", "");
      testFiles[testName] = `./tests/${dirEntry.name}`;
    }
  }

  // If no tests are specified, run all tests by default
  const tests = testsToRun.length > 0 ? testsToRun : Object.keys(testFiles);
  const testReport: TestReportEntry[] = [];

  for (const testName of tests) {
    const testPath = testFiles[testName];
    const testEntry = createDefaultTestReportEntry(testName);

    try {
      if (testPath) {
        try {
          const { default: testFunc } = await import(testPath);
          let testStart = new Date();
          await testFunc(sdfApiClient);
          testEntry.test_result = "success";
          testEntry.finish_time = new Date().toISOString();
          testEntry.test_duration = `${new Date().getTime() - testStart.getTime()}ms`;
        } catch (importError) {
          testEntry.message = `Failed to load test file "${testPath}": ${importError.message}`;
        }
      } else {
        testEntry.message = `test "${testName}" not found.`;
      }
    } catch (error) {
      testEntry.message = error.message;
    } finally {
      testEntry.finish_time = new Date().toISOString();
      testEntry.test_duration = `${new Date().getTime() - new Date(testEntry.start_time).getTime()}ms`;
      testReport.push(testEntry);
    }
  }

  // Generate and print the report
  printTestReport(testReport);

  // Assert that all tests passed
  assert(testReport.every(entry => entry.test_result === "success"), "Not all tests passed. Please check the logs for details.");

  console.log("~~ ALL TESTS PASSED ~~");
}
