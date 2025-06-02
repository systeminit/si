#!/usr/bin/env -S deno run --allow-net --allow-env --allow-read

import { load } from "https://deno.land/std/dotenv/mod.ts";

// Load environment variables from .env file if available
await load({ export: true });

// Required configuration - use same env vars as test framework with fallbacks
const AUTH_TOKEN = Deno.env.get("LUMINORK_AUTH_TOKEN");
const API_URL = Deno.env.get("LUMINORK_API_URL") || "http://localhost:5380";
const WORKSPACE_ID = Deno.env.get("LUMINORK_WORKSPACE_ID");

// Handle quoted values (same as test-utils.ts)
let authToken = AUTH_TOKEN || "";
if (authToken.startsWith('"') && authToken.endsWith('"')) {
  authToken = authToken.slice(1, -1);
}

let workspaceId = WORKSPACE_ID || "";
if (workspaceId.startsWith('"') && workspaceId.endsWith('"')) {
  workspaceId = workspaceId.slice(1, -1);
}

if (!authToken) {
  console.error(
    "ERROR: LUMINORK_AUTH_TOKEN or AUTH_TOKEN environment variable is required",
  );
  Deno.exit(1);
}

if (!workspaceId) {
  console.error(
    "ERROR: LUMINORK_WORKSPACE_ID or WORKSPACE_ID environment variable is required",
  );
  Deno.exit(1);
}

console.log(`Using auth token starting with: ${authToken.substring(0, 20)}...`);
console.log(`Using workspace ID: ${workspaceId}`);

console.log(`Using API URL: ${API_URL}`);

// Helper function to make API requests
async function apiRequest(
  path: string,
  method = "GET",
  body?: unknown,
): Promise<any> {
  const url = `${API_URL}${path.startsWith("/") ? path : "/" + path}`;

  const headers: HeadersInit = {
    "Content-Type": "application/json",
    Authorization: `Bearer ${authToken}`,
  };

  const options: RequestInit = {
    method,
    headers,
  };

  if (body) {
    options.body = JSON.stringify(body);
  }

  try {
    const response = await fetch(url, options);

    console.log(`Response status: ${response.status}`);

    if (response.status >= 200 && response.status < 300) {
      const contentType = response.headers.get("content-type");
      if (contentType && contentType.includes("application/json")) {
        const json = await response.json();
        console.log("Response body:", JSON.stringify(json, null, 2));
        return json;
      } else {
        const text = await response.text();
        console.log("Response body:", text);
        return text;
      }
    } else {
      const text = await response.text();
      console.error(`HTTP Error: ${response.status} ${response.statusText}`);
      console.error("Response body:", text);
      throw new Error(`HTTP Error: ${response.status} ${response.statusText}`);
    }
  } catch (error) {
    console.error("Request failed:", error);
    throw error;
  }
}

async function testSystemStatus() {
  const result = await apiRequest("/");
  console.log("\nSystem Status Test Result:", result["API Documentation"]);
}

async function testWhoami() {
  const result = await apiRequest("/whoami");
  console.log("\nWhoami Test Result:", result);
}

async function testCreateChangeSet() {
  const changeSetName = `Test Change Set ${new Date().toISOString()}`;

  // This format matches what we see in the API client
  const result = await apiRequest(`/v1/w/${workspaceId}/change-sets`, "POST", {
    changeSetName: changeSetName,
  });

  console.log("\nCreate Change Set Test Result:", result);
  return result;
}

async function testGetChangeSet(changeSetId: string) {
  const result = await apiRequest(
    `/v1/w/${workspaceId}/change-sets/${changeSetId}`,
  );
  console.log("\nGet Change Set Test Result:", result);
  return result;
}

async function testListChangeSet() {
  const result = await apiRequest(`/v1/w/${workspaceId}/change-sets`);
  console.log("\nList Change Sets Test Result:", result);
  return result;
}

async function testListSchemas(changeSetId: string) {
  const result = await apiRequest(
    `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas`,
  );
  console.log("\nList Schemas Test Result:", result);
  // Return a consistent structure that includes 'items' for schemas
  return { items: result.schemas || [] };
}

async function testFindSchemaByName(changeSetId: string, name: string) {
  const result = await apiRequest(
    `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas?name=${encodeURIComponent(name)}`,
  );

  console.log("\nFind Schema Test Result:", result);
  return result;
}

async function testGetSchema(changeSetId: string, schemaId: string) {
  const result = await apiRequest(
    `/v1/w/${workspaceId}/change-sets/${changeSetId}/schemas/${schemaId}`,
  );
  console.log("\nGet Schema Test Result:", result);
  return result;
}

async function testCreateComponent(changeSetId: string) {
  const result = await apiRequest(
    `/v1/w/${workspaceId}/change-sets/${changeSetId}/components`,
    "POST",
    {
      schemaName: "AWS::EC2::Instance",
      name: `Test Component ${new Date().toISOString()}`,
    },
  );

  console.log("\nCreate Component Test Result:", result);
  return result;
}

async function testListComponents(changeSetId: string) {
  const result = await apiRequest(
    `/v1/w/${workspaceId}/change-sets/${changeSetId}/components`,
  );

  console.log("\nList Components Test Result:", result);
  return result;
}

async function testFindComponentByName(changeSetId: string, name: string) {
  console.log(`\n=== Testing Find Component by Name (${name}) ===`);

  // Note that we use 'component' as the query parameter name, not 'name'
  // This matches the API's expected parameter format
  const result = await apiRequest(
    `/v1/w/${workspaceId}/change-sets/${changeSetId}/components/find?component=${encodeURIComponent(name)}`,
  );

  console.log("\nFind Component Test Result:", result);
  return result;
}

// We're using the API client for authentication, so we don't need the curl test anymore

// Main test function
async function runTests() {
  try {
    console.log("=== Testing System Status ===");
    await testSystemStatus();

    console.log("\n=== Testing Whoami ===");
    await testWhoami();

    console.log("\n=== Testing List Change Sets ===");
    const changeSets = await testListChangeSet();

    console.log("\n=== Testing Create Change Set ===");
    const changeSetResult = await testCreateChangeSet();

    if (
      changeSetResult &&
      changeSetResult.changeSet &&
      changeSetResult.changeSet.id
    ) {
      const changeSetId = changeSetResult.changeSet.id;
      console.log(`Created change set with ID: ${changeSetId}`);

      console.log("\n=== Testing Get Change Set ===");
      await testGetChangeSet(changeSetId);

      console.log("\n=== Testing List Schemas ===");
      const schemas = await testListSchemas(changeSetId);

      if (schemas && schemas.items && schemas.items.length > 0) {
        const schemaId = schemas.items[0].schemaId;

        console.log(`\n=== Testing Get Schema (ID: ${schemaId}) ===`);
        await testGetSchema(changeSetId, schemaId);

        console.log(
          `\n=== Testing Find Schema by Name (${schemas?.items[0]?.schemaName || "Unknown"}) ===`,
        );
        if (schemas?.items[0]?.schemaName) {
          await testFindSchemaByName(changeSetId, schemas.items[0].schemaName);
        }

        console.log(
          `\n=== Testing Create Component (with Change Set ID: ${changeSetId}) ===`,
        );
        const component = await testCreateComponent(changeSetId);

        console.log(
          `\n=== Testing List Components (with Change Set ID: ${changeSetId}) ===`,
        );
        await testListComponents(changeSetId);

        const componentName = component?.component?.name || "";
        if (componentName) {
          console.log(
            `\n=== Testing Find Component by Name (${componentName}) ===`,
          );
          await testFindComponentByName(changeSetId, componentName);

          // Also test with a simple test component name
          console.log(
            `\n=== Testing Find Component by Name (TestComponent) ===`,
          );
          await testFindComponentByName(changeSetId, "TestComponent");
        }
      } else {
        console.log("No schemas available to test with");
      }
    } else {
      console.log(
        "Failed to create change set or missing change set ID in response",
      );
    }
  } catch (error) {
    console.error("Test failed:", error);
  }
}

// Run the tests
runTests();
