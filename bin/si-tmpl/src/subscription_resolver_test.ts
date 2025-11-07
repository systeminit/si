/**
 * Tests for shared subscription resolution utilities.
 *
 * @module
 */

import { assertEquals, assertRejects } from "@std/assert";
import {
  resolveComponentReference,
  resolveSearchQuery,
  type SearchFunction,
} from "./subscription_resolver.ts";
import { Context } from "./context.ts";

// Initialize context for tests (suppress logging)
Context.init({ verbose: 0, noColor: true });

Deno.test("resolveComponentReference - passes through ULID unchanged", async () => {
  const ulid = "01HQZX3Y4N5P6Q7R8S9T0V1W2X";
  const ctx = Context.instance();

  // Mock search function that should never be called
  const searchFn: SearchFunction = () =>
    Promise.reject(new Error("Search should not be called for ULIDs"));

  const result = await resolveComponentReference(
    ulid,
    "workspace-id",
    "changeset-id",
    searchFn,
    ctx.logger,
  );

  assertEquals(result, ulid);
});

Deno.test("resolveComponentReference - resolves component name to ID via search", async () => {
  const componentName = "my-component";
  const expectedId = "01HQZX3Y4N5P6Q7R8S9T0V1W2X";
  const ctx = Context.instance();

  // Mock search function that returns one result
  const searchFn: SearchFunction = (
    _workspaceId: string,
    _changeSetId: string,
    query: string,
  ) => {
    assertEquals(query, `name: "${componentName}"`);
    return Promise.resolve({
      components: [
        {
          id: expectedId,
          name: componentName,
          schema: { name: "Test Schema" },
        },
      ],
    });
  };

  const result = await resolveComponentReference(
    componentName,
    "workspace-id",
    "changeset-id",
    searchFn,
    ctx.logger,
  );

  assertEquals(result, expectedId);
});

Deno.test("resolveComponentReference - throws when component name not found", async () => {
  const componentName = "nonexistent-component";
  const ctx = Context.instance();

  // Mock search function that returns no results
  const searchFn: SearchFunction = () => Promise.resolve({ components: [] });

  await assertRejects(
    async () => {
      await resolveComponentReference(
        componentName,
        "workspace-id",
        "changeset-id",
        searchFn,
        ctx.logger,
      );
    },
    Error,
    `No component found with name: "${componentName}"`,
  );
});

Deno.test("resolveComponentReference - throws when multiple components found with same name", async () => {
  const componentName = "duplicate-component";
  const ctx = Context.instance();

  // Mock search function that returns multiple results
  const searchFn: SearchFunction = () =>
    Promise.resolve({
      components: [
        {
          id: "01HQZX3Y4N5P6Q7R8S9T0V1W2X",
          name: componentName,
          schema: { name: "Test Schema" },
        },
        {
          id: "01HQZX3Y4N5P6Q7R8S9T0V1W3Y",
          name: componentName,
          schema: { name: "Test Schema" },
        },
      ],
    });

  await assertRejects(
    async () => {
      await resolveComponentReference(
        componentName,
        "workspace-id",
        "changeset-id",
        searchFn,
        ctx.logger,
      );
    },
    Error,
    `Multiple components found with name "${componentName}"`,
  );
});

Deno.test("resolveSearchQuery - returns component ID for single search result", async () => {
  const query = 'schemaName:"AWS EC2 Instance"';
  const expectedId = "01HQZX3Y4N5P6Q7R8S9T0V1W2X";
  const ctx = Context.instance();

  // Mock search function that returns one result
  const searchFn: SearchFunction = (
    _workspaceId: string,
    _changeSetId: string,
    receivedQuery: string,
  ) => {
    assertEquals(receivedQuery, query);
    return Promise.resolve({
      components: [
        {
          id: expectedId,
          name: "my-ec2-instance",
          schema: { name: "AWS EC2 Instance" },
        },
      ],
    });
  };

  const result = await resolveSearchQuery(
    query,
    "workspace-id",
    "changeset-id",
    searchFn,
    ctx.logger,
  );

  assertEquals(result, expectedId);
});

Deno.test("resolveSearchQuery - throws when no components found", async () => {
  const query = 'schemaName:"Nonexistent Schema"';
  const ctx = Context.instance();

  // Mock search function that returns no results
  const searchFn: SearchFunction = () => Promise.resolve({ components: [] });

  await assertRejects(
    async () => {
      await resolveSearchQuery(
        query,
        "workspace-id",
        "changeset-id",
        searchFn,
        ctx.logger,
      );
    },
    Error,
    `No components found for search query: "${query}"`,
  );
});

Deno.test("resolveSearchQuery - throws when multiple components found (simple error)", async () => {
  const query = 'schemaName:"AWS EC2 Instance"';
  const ctx = Context.instance();

  // Mock search function that returns multiple results
  const searchFn: SearchFunction = () =>
    Promise.resolve({
      components: [
        {
          id: "01HQZX3Y4N5P6Q7R8S9T0V1W2X",
          name: "instance-1",
          schema: { name: "AWS EC2 Instance" },
        },
        {
          id: "01HQZX3Y4N5P6Q7R8S9T0V1W3Y",
          name: "instance-2",
          schema: { name: "AWS EC2 Instance" },
        },
      ],
    });

  await assertRejects(
    async () => {
      await resolveSearchQuery(
        query,
        "workspace-id",
        "changeset-id",
        searchFn,
        ctx.logger,
      );
    },
    Error,
    "Search returned 2 components",
  );
});

Deno.test("resolveSearchQuery - enriched error with getComponent and getSchemaName functions", async () => {
  const query = 'tag:"production"';
  const ctx = Context.instance();

  // Mock search function that returns multiple results
  const searchFn: SearchFunction = () =>
    Promise.resolve({
      components: [
        {
          id: "01HQZX3Y4N5P6Q7R8S9T0V1W2X",
          name: "web-server",
          schema: { name: "AWS EC2 Instance" },
        },
        {
          id: "01HQZX3Y4N5P6Q7R8S9T0V1W3Y",
          name: "db-server",
          schema: { name: "AWS RDS Database" },
        },
      ],
    });

  // Mock get component function
  const getComponentFn = (
    _workspaceId: string,
    _changeSetId: string,
    componentId: string,
  ) => {
    if (componentId === "01HQZX3Y4N5P6Q7R8S9T0V1W2X") {
      return Promise.resolve({
        component: { schemaId: "schema-ec2", name: "web-server" },
      });
    }
    return Promise.resolve({
      component: { schemaId: "schema-rds", name: "db-server" },
    });
  };

  // Mock get schema name function
  const getSchemaNameFn = (
    _workspaceId: string,
    _changeSetId: string,
    schemaId: string,
  ) => {
    if (schemaId === "schema-ec2") return Promise.resolve("AWS EC2 Instance");
    return Promise.resolve("AWS RDS Database");
  };

  await assertRejects(
    async () => {
      await resolveSearchQuery(
        query,
        "workspace-id",
        "changeset-id",
        searchFn,
        ctx.logger,
        getComponentFn,
        getSchemaNameFn,
      );
    },
    Error,
    "AWS EC2 Instance: web-server",
  );
});

Deno.test("resolveSearchQuery - handles error in getComponent gracefully", async () => {
  const query = 'tag:"production"';
  const ctx = Context.instance();

  // Mock search function that returns multiple results
  const searchFn: SearchFunction = () =>
    Promise.resolve({
      components: [
        {
          id: "01HQZX3Y4N5P6Q7R8S9T0V1W2X",
          name: "web-server",
          schema: { name: "AWS EC2 Instance" },
        },
        {
          id: "01HQZX3Y4N5P6Q7R8S9T0V1W3Y",
          name: "db-server",
          schema: { name: "AWS RDS Database" },
        },
      ],
    });

  // Mock get component function that throws for first component
  const getComponentFn = (
    _workspaceId: string,
    _changeSetId: string,
    componentId: string,
  ) => {
    if (componentId === "01HQZX3Y4N5P6Q7R8S9T0V1W2X") {
      return Promise.reject(new Error("Component fetch failed"));
    }
    return Promise.resolve({
      component: { schemaId: "schema-rds", name: "db-server" },
    });
  };

  // Mock get schema name function
  const getSchemaNameFn = () => Promise.resolve("AWS RDS Database");

  const error = await assertRejects(
    async () => {
      await resolveSearchQuery(
        query,
        "workspace-id",
        "changeset-id",
        searchFn,
        ctx.logger,
        getComponentFn,
        getSchemaNameFn,
      );
    },
    Error,
  );

  // Should contain fallback for failed component and enriched info for successful one
  assertEquals(
    error.message.includes("(unknown schema): web-server"),
    true,
    "Should show unknown schema for failed component",
  );
  assertEquals(
    error.message.includes("AWS RDS Database: db-server"),
    true,
    "Should show enriched info for successful component",
  );
});

Deno.test("resolveComponentReference - handles various ULID formats", async () => {
  const ctx = Context.instance();
  const searchFn: SearchFunction = () =>
    Promise.reject(new Error("Should not be called"));

  // Lowercase ULID
  const lowerUlid = "01hqzx3y4n5p6q7r8s9t0v1w2x";
  const result1 = await resolveComponentReference(
    lowerUlid,
    "workspace-id",
    "changeset-id",
    searchFn,
    ctx.logger,
  );
  assertEquals(result1, lowerUlid);

  // Mixed case ULID
  const mixedUlid = "01HqZx3Y4n5P6Q7r8S9T0v1W2X";
  const result2 = await resolveComponentReference(
    mixedUlid,
    "workspace-id",
    "changeset-id",
    searchFn,
    ctx.logger,
  );
  assertEquals(result2, mixedUlid);

  // ULID with valid Crockford base32 characters only (no I, L, O, U)
  const validUlid = "01HJKMNPQRSTVWXYZ0123456AB";
  const result3 = await resolveComponentReference(
    validUlid,
    "workspace-id",
    "changeset-id",
    searchFn,
    ctx.logger,
  );
  assertEquals(result3, validUlid);
});

Deno.test("resolveComponentReference - treats invalid ULIDs as names", async () => {
  const ctx = Context.instance();
  const expectedId = "01HQZX3Y4N5P6Q7R8S9T0V1W2X";

  // Mock search function
  const searchFn: SearchFunction = (
    _workspaceId: string,
    _changeSetId: string,
    query: string,
  ) =>
    Promise.resolve({
      components: [
        {
          id: expectedId,
          name: query.match(/name: "(.+)"/)?.[1] || "",
          schema: { name: "Test Schema" },
        },
      ],
    });

  // Too short
  const shortId = "01HQZX3Y4N5P6Q7R8S9T0V1W2";
  const result1 = await resolveComponentReference(
    shortId,
    "workspace-id",
    "changeset-id",
    searchFn,
    ctx.logger,
  );
  assertEquals(result1, expectedId);

  // Too long
  const longId = "01HQZX3Y4N5P6Q7R8S9T0V1W2XY";
  const result2 = await resolveComponentReference(
    longId,
    "workspace-id",
    "changeset-id",
    searchFn,
    ctx.logger,
  );
  assertEquals(result2, expectedId);

  // Contains excluded Crockford characters (I, L, O, U)
  const invalidChars = "01HQZX3Y4N5P6Q7R8S9T0V1I2L";
  const result3 = await resolveComponentReference(
    invalidChars,
    "workspace-id",
    "changeset-id",
    searchFn,
    ctx.logger,
  );
  assertEquals(result3, expectedId);
});
