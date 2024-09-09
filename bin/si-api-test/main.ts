import { SdfApiClient } from "./sdf_api_client.ts";
import { createDefaultTestReportEntry, printTestReport, TestReportEntry, testsFailed } from "./test_execution_lib.ts";
import { checkEnvironmentVariables, parseArgs } from "./binary_execution_lib.ts";

if (import.meta.main) {
  // Parse args and check environment variables
  const { workspaceId, userId, password, testsToRun, testProfile } = parseArgs(Deno.args);
  checkEnvironmentVariables(Deno.env.toObject());

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
  let completedTests = 0;
  let triggeredTests = 0;
  let testExecutionSequence = 1;
  const totalTests = (testProfile?.Requests ?? 1) * tests.length;
  const startTime = Date.now();
  const totalDurationMs = Number(testProfile?.Duration) * 1000;

  // Function to display the progress bars
  function displayProgressBars(triggered: number, completed: number, total: number) {
    const barLength = 15; // Length of each progress bar

    // Calculate progress for triggered tests
    const triggeredProgress = Math.min(triggered / total, 1);
    const triggeredBars = Math.round(triggeredProgress * barLength);
    const triggeredProgressBar = `[${"=".repeat(triggeredBars)}${" ".repeat(barLength - triggeredBars)}]`;
    const triggeredPercent = (triggeredProgress * 100).toFixed(2);

    // Calculate progress for completed tests
    const completedProgress = Math.min(completed / total, 1);
    const completedBars = Math.round(completedProgress * barLength);
    const completedProgressBar = `[${"=".repeat(completedBars)}${" ".repeat(barLength - completedBars)}]`;
    const completedPercent = (completedProgress * 100).toFixed(2);

    // Calculate elapsed and remaining time
    const elapsed = Date.now() - startTime;
    const remaining = Math.max(totalDurationMs - elapsed, 0);
    const remainingSeconds = Math.ceil(remaining / 1000);

    // Log progress bars with total and remaining time
    console.log(`Triggered: ${triggeredProgressBar} ${triggeredPercent}% (${triggered}/${total}) | Completed: ${completedProgressBar} ${completedPercent}% (${completed}/${total}) | Remaining Time: ${remainingSeconds}s`);
  }

  // Define the test execution function
  const executeTest = async (testName: string, sdfApiClient: SdfApiClient, sequence: number, showProgressBar: boolean) => {
    const testEntry = createDefaultTestReportEntry(testName);
    const testPath = testFiles[testName];

    // Display progress bar immediately when the test is triggered (only if showProgressBar is true)
    if (showProgressBar) {
      triggeredTests++;
      displayProgressBars(triggeredTests, completedTests, totalTests);
    }

    if (!testPath) {
      testEntry.message = `Test "${testName}" not found.`;
      testReport.push(testEntry);
      completedTests++;
      if (showProgressBar) {
        displayProgressBars(triggeredTests, completedTests, totalTests);
      }
      return;
    }

    try {
      const { default: testFunc } = await import(testPath);
      const testStart = new Date();
      await testFunc(sdfApiClient);
      testEntry.test_result = "success";
      testEntry.finish_time = new Date().toISOString();
      testEntry.test_duration = `${new Date().getTime() - testStart.getTime()}ms`;
    } catch (error) {
      testEntry.message = `Error in test "${testName}": ${error.message}`;
      testEntry.test_result = "failure";
    } finally {
      testEntry.finish_time = new Date().toISOString();
      testEntry.test_duration = `${new Date().getTime() - new Date(testEntry.start_time).getTime()}ms`;
      testEntry.test_execution_sequence = sequence;
      testReport.push(testEntry);

      // Update the completed tests count and display progress bars (only if showProgressBar is true)
      completedTests++;
      if (showProgressBar) {
        displayProgressBars(triggeredTests, completedTests, totalTests);
      }
    }
  };

  if (testProfile?.Duration && testProfile?.Requests) {
    const interval = Math.floor(Number(testProfile.Duration) * 1000 / testProfile.Requests);
    let elapsed = 0;

    // Create a list of promises for all test executions
    const testPromises: Promise<void>[] = [];

    // Execute tests based on the profile
    const intervalId = setInterval(async () => {
      if (elapsed >= Number(testProfile.Duration) * 1000) {
        clearInterval(intervalId);
        // Wait for all test executions to complete
        await Promise.all(testPromises);
        console.log("~~ FINAL REPORT GENERATED ~~");
        printTestReport(testReport);
        return;
      }

      for (const testName of tests) {
        // Execute tests asynchronously and increment sequence, show progress bar
        const testPromise = executeTest(testName, sdfApiClient, testExecutionSequence++, true);
        testPromises.push(testPromise);
      }

      elapsed += interval;
    }, interval);
  } else {
    // Fallback to one-shot execution if no profile is set, do not show progress bar
    const oneShotPromises: Promise<void>[] = tests.map(testName => executeTest(testName, sdfApiClient, testExecutionSequence++, false));
    await Promise.all(oneShotPromises);
    console.log("~~ FINAL REPORT GENERATED ~~");
    printTestReport(testReport);
    let exitCode = testsFailed(testReport) ? 1 : 0;
    Deno.exit(exitCode);
  }
}
