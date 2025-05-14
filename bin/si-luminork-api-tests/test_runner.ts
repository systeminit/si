#!/usr/bin/env -S deno run --allow-net --allow-env --allow-read

/**
 * Custom test runner to avoid showing post-test output delimiters
 */
 
import { TestDefinition } from "https://deno.land/std@0.220.1/testing/types.ts";

// Original test function
const originalTest = Deno.test;

// Override Deno.test with our custom version
Deno.test = function customTest(
  nameOrFnOrOptions: string | Function | TestDefinition,
  optionalFn?: Function,
): void {
  // Handle different ways of calling Deno.test
  let testName = "";
  let testFn: Function;

  if (typeof nameOrFnOrOptions === "string") {
    testName = nameOrFnOrOptions;
    testFn = optionalFn as Function;
  } else if (typeof nameOrFnOrOptions === "function") {
    testFn = nameOrFnOrOptions;
  } else {
    // It's a TestDefinition object
    testName = nameOrFnOrOptions.name;
    testFn = nameOrFnOrOptions.fn;
  }

  // Create a wrapper function that captures and filters console output
  const wrappedFn = async function(...args: any[]) {
    // Save original console.log
    const originalConsoleLog = console.log;
    
    // Override console.log to filter out the delimiters
    console.log = function(...logArgs: any[]) {
      const logStr = String(logArgs[0] || "");
      
      // Skip logging these marker lines
      if (
        logStr.includes("------- post-test output -------") || 
        logStr.includes("----- post-test output end -----")
      ) {
        return;
      }
      
      // Call original console.log for everything else
      originalConsoleLog.apply(console, logArgs);
    };
    
    try {
      // Run the actual test function
      return await testFn.apply(this, args);
    } finally {
      // Restore original console.log
      console.log = originalConsoleLog;
    }
  };

  // Call the original Deno.test with our wrapped function
  if (typeof nameOrFnOrOptions === "string") {
    return originalTest(nameOrFnOrOptions, wrappedFn);
  } else if (typeof nameOrFnOrOptions === "function") {
    return originalTest(wrappedFn);
  } else {
    // It's a TestDefinition object
    return originalTest({
      ...nameOrFnOrOptions,
      fn: wrappedFn,
    });
  }
};

// Import and run the tests
import "./tests/system-status.test.ts";
import "./tests/change-sets.test.ts";
import "./tests/components.test.ts";
import "./tests/schemas.test.ts";