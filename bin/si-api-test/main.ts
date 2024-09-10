import { SdfApiClient } from "./sdf_api_client.ts";
import {
  createDefaultTestReportEntry,
  printTestReport,
  TestFunction,
  TestReportEntry,
  testsFailed,
} from "./test_execution_lib.ts";
import {
  checkEnvironmentVariables,
  parseArgs,
} from "./binary_execution_lib.ts";
import { sleep } from "./test_helpers.ts";

const testReport: TestReportEntry[] = [];

if (import.meta.main) {
  // Parse args and check environment variables
  const { workspaceId, userId, password, testsToRun, testProfile, token } =
    parseArgs(
      Deno.args,
    );
  checkEnvironmentVariables(Deno.env.toObject());

  // Init the SDF Module
  const sdfApiClient = await SdfApiClient.init({
    workspaceId,
    userEmailOrId: userId,
    password,
    token,
  });

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

  // Load test funcs after filter
  const testFuncs = {} as Record<string, TestFunction>;
  for (const testName of tests) {
    const testPath = testFiles[testName];
    const { default: testFunc } = await import(testPath);

    testFuncs[testName] = testFunc;
  }

  console.log("Running tests:");
  console.dir(tests);

  // Create a list of promises for all test executions
  const testPromises: Promise<void>[] = [];

  // Execute tests based on the profile
  const startTime = Date.now();
  let testExecutionSequence = 1;
  let elapsed = 0;

  const intervalId = setInterval(() => {
    const jobTotal = testPromises.length;
    const jobsFinished = testReport.length;
    const elapsedTime = Math.floor((Date.now() - startTime) / 1000);
    console.log(
      `Finished ${jobsFinished} out of ${jobTotal}, ran for ${elapsedTime}s`,
    );
  }, 1000);

  do {
    for (const testName of tests) {
      // Execute tests asynchronously and increment sequence, show progress bar
      const testPromise = executeTest(
        testName,
        testFuncs[testName],
        sdfApiClient,
        testExecutionSequence++,
      );
      testPromises.push(testPromise);
    }

    elapsed = Date.now() - startTime;

    const jitter = testProfile?.useJitter ? (Math.random() * 1000) : 0;
    const sleepAmount = testProfile?.rate ? testProfile.rate + jitter : 0;

    await sleep(sleepAmount);
  } while (testProfile && elapsed < (testProfile.maxDuration * 1000));
  console.log("Finished enqueuing jobs");

  await Promise.all(testPromises);
  clearInterval(intervalId);
  console.log("~~ FINAL REPORT GENERATED ~~");
  printTestReport(testReport);
  const exitCode = testsFailed(testReport) ? 1 : 0;
  Deno.exit(exitCode);
}

// Define the test execution function
async function executeTest(
  testName: string,
  testFn: TestFunction,
  sdfApiClient: SdfApiClient,
  sequence: number,
) {
  const testEntry = createDefaultTestReportEntry(testName);

  // Display progress bar immediately when the test is triggered (only if showProgressBar is true)
  try {
    const testStart = new Date();
    await testFn(sdfApiClient);
    testEntry.test_result = "success";
    testEntry.finish_time = new Date().toISOString();
    testEntry.test_duration = `${new Date().getTime() - testStart.getTime()}ms`;
  } catch (error) {
    testEntry.message = `Error in test "${testName}": ${error.message}`;
    testEntry.test_result = "failure";
  } finally {
    testEntry.finish_time = new Date().toISOString();
    testEntry.test_duration = `${
      new Date().getTime() - new Date(testEntry.start_time).getTime()
    }ms`;
    testEntry.test_execution_sequence = sequence;
    testReport.push(testEntry);
  }
}
