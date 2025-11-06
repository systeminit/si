/**
 * SI Test Framework HTTP Server
 *
 * A long-running service that accepts test submissions via HTTP
 * and returns results. Much faster than spinning up containers.
 */

import { serve } from "https://deno.land/std@0.224.0/http/server.ts";
import { join } from "https://deno.land/std@0.224.0/path/mod.ts";
import { runTestSuite } from "./runner.ts";
import type { TestSuite } from "./types.ts";

const PORT = 8081;

interface TestRequest {
  functionCode: string;
  testCode: string;
  timeout?: number;
}

interface TestResponse {
  success: boolean;
  results?: unknown[];
  error?: string;
  duration: number;
}

async function handleTestRequest(req: Request): Promise<Response> {
  if (req.method !== "POST") {
    return new Response("Method not allowed", { status: 405 });
  }

  const startTime = performance.now();

  try {
    const body: TestRequest = await req.json();

    if (!body.functionCode || !body.testCode) {
      return Response.json({
        success: false,
        error: "Missing functionCode or testCode",
        duration: performance.now() - startTime,
      } as TestResponse, { status: 400 });
    }

    console.log("[SERVER] Received test request");

    // Parse test suite from code
    const testModule = await importTestModule(body.testCode);
    const testSuite = testModule.default;

    if (!testSuite || typeof testSuite !== "object") {
      return Response.json({
        success: false,
        error: "Test file must export default test suite",
        duration: performance.now() - startTime,
      } as TestResponse, { status: 400 });
    }

    // Run the tests
    const results = await runTestSuite(
      body.functionCode,
      testSuite as TestSuite,
      {
        verbose: false,
      },
    );

    const allPassed = results.every((r) => r.passed || r.skipped);
    const duration = performance.now() - startTime;

    console.log(
      `[SERVER] Test completed: ${results.length} tests, ${
        allPassed ? "all passed" : "some failed"
      } (${duration.toFixed(0)}ms)`,
    );

    return Response.json({
      success: allPassed,
      results,
      duration,
    } as TestResponse);
  } catch (err) {
    console.error("[SERVER] Error processing test:", err);

    return Response.json({
      success: false,
      error: err instanceof Error ? err.message : String(err),
      duration: performance.now() - startTime,
    } as TestResponse, { status: 500 });
  }
}

async function importTestModule(testCode: string): Promise<{
  default: unknown;
}> {
  // Create a temporary file for the test code
  const tempDir = await Deno.makeTempDir({ prefix: "test-server-" });
  const testFile = join(tempDir, "test.ts");

  try {
    await Deno.writeTextFile(testFile, testCode);
    const module = await import(`file://${testFile}`);
    return module;
  } finally {
    // Cleanup
    try {
      await Deno.remove(tempDir, { recursive: true });
    } catch {
      // Ignore cleanup errors
    }
  }
}

function handleHealthCheck(_req: Request): Response {
  return Response.json({ status: "ok", timestamp: Date.now() });
}

async function handler(req: Request): Promise<Response> {
  const url = new URL(req.url);

  console.log(`[SERVER] ${req.method} ${url.pathname}`);

  // CORS headers
  const headers = {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
  };

  if (req.method === "OPTIONS") {
    return new Response(null, { status: 204, headers });
  }

  let response: Response;

  if (url.pathname === "/health") {
    response = await handleHealthCheck(req);
  } else if (url.pathname === "/test") {
    response = await handleTestRequest(req);
  } else {
    response = new Response("Not found", { status: 404 });
  }

  // Add CORS headers to response
  Object.entries(headers).forEach(([key, value]) => {
    response.headers.set(key, value);
  });

  return response;
}

console.log(`[SERVER] SI Test Framework Server starting on port ${PORT}`);
console.log(`[SERVER] Health check: http://localhost:${PORT}/health`);
console.log(`[SERVER] Submit tests: POST http://localhost:${PORT}/test`);

await serve(handler, { port: PORT });
