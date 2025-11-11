import { assertEquals, assertExists } from "@std/assert";
import { Context } from "../context.ts";
import { cacheComponentData, type ComponentGetCache } from "./cache.ts";

// Initialize Context once - this runs at module load time
await Context.init({ verbose: 0, noColor: true });

Deno.test("cacheComponentData() - writes JSON format correctly", async () => {
  const ctx = Context.instance();

  const cacheData: ComponentGetCache = {
    componentId: "comp-123",
    schemaId: "schema-456",
    schemaName: "AWS EC2 Instance",
    resourceId: "i-1234567890abcdef0",
    toDelete: false,
    canBeUpgraded: false,
    qualified: true,
    attributes: {
      "/si/name": "my-instance",
      "/si/color": "#ff0000",
      "/domain/instanceType": "t3.micro",
      "/domain/region": "us-east-1",
      "/resource_value/InstanceId": "i-1234567890abcdef0",
    },
    resourceData: {
      InstanceId: "i-1234567890abcdef0",
      State: { Name: "running" },
    },
    qualifications: [
      { name: "ValidConfiguration", status: "success", message: "Valid" },
    ],
    actions: [
      { id: "action-123", name: "create", state: "queued" },
    ],
  };

  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  try {
    await cacheComponentData(cacheData, tempFile, ctx.logger);

    const content = await Deno.readTextFile(tempFile);
    const parsed = JSON.parse(content);

    assertEquals(parsed.componentId, "comp-123");
    assertEquals(parsed.schemaName, "AWS EC2 Instance");
    assertEquals(parsed.qualified, true);
    assertEquals(parsed.attributes["/domain/instanceType"], "t3.micro");
    assertEquals(parsed.qualifications.length, 1);
    assertEquals(parsed.actions.length, 1);
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("cacheComponentData() - writes YAML format correctly", async () => {
  const ctx = Context.instance();

  const cacheData: ComponentGetCache = {
    componentId: "comp-456",
    schemaId: "schema-789",
    schemaName: "AWS S3 Bucket",
    toDelete: false,
    canBeUpgraded: false,
    qualified: false,
    attributes: {
      "/si/name": "my-bucket",
      "/domain/bucketName": "my-test-bucket",
    },
    qualifications: [],
    actions: [],
  };

  const tempFile = await Deno.makeTempFile({ suffix: ".yaml" });
  try {
    await cacheComponentData(cacheData, tempFile, ctx.logger);

    const content = await Deno.readTextFile(tempFile);

    // Verify YAML contains expected data
    assertExists(content);
    assertEquals(content.includes("componentId: comp-456"), true);
    assertEquals(content.includes("schemaName: AWS S3 Bucket"), true);
    assertEquals(content.includes("qualified: false"), true);
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("cacheComponentData() - writes YAML with .yml extension", async () => {
  const ctx = Context.instance();

  const cacheData: ComponentGetCache = {
    componentId: "comp-789",
    schemaId: "schema-123",
    schemaName: "Test Schema",
    toDelete: false,
    canBeUpgraded: false,
    qualified: true,
    attributes: {},
    qualifications: [],
    actions: [],
  };

  const tempFile = await Deno.makeTempFile({ suffix: ".yml" });
  try {
    await cacheComponentData(cacheData, tempFile, ctx.logger);

    const content = await Deno.readTextFile(tempFile);
    assertExists(content);
    assertEquals(content.includes("componentId: comp-789"), true);
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("cacheComponentData() - handles optional fields correctly", async () => {
  const ctx = Context.instance();

  const cacheData: ComponentGetCache = {
    componentId: "comp-minimal",
    schemaId: "schema-minimal",
    schemaName: "Minimal Schema",
    toDelete: false,
    canBeUpgraded: false,
    qualified: true,
    attributes: {},
    qualifications: [],
    actions: [],
  };

  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  try {
    await cacheComponentData(cacheData, tempFile, ctx.logger);

    const content = await Deno.readTextFile(tempFile);
    const parsed = JSON.parse(content);

    assertEquals(parsed.resourceId, undefined);
    assertEquals(parsed.resourceData, undefined);
    assertEquals(parsed.qualifications, []);
    assertEquals(parsed.actions, []);
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("cacheComponentData() - handles complex nested attributes", async () => {
  const ctx = Context.instance();

  const cacheData: ComponentGetCache = {
    componentId: "comp-complex",
    schemaId: "schema-complex",
    schemaName: "Complex Schema",
    toDelete: false,
    canBeUpgraded: true,
    qualified: true,
    attributes: {
      "/si/name": "complex",
      "/si/nested": { deep: { value: 123 } },
      "/domain/array": [1, 2, 3],
      "/domain/object": { key1: "value1", key2: "value2" },
      "/secrets/apiKey": "secret-123",
      "/resource_value/status": "active",
      "/resource_value/metadata": { created: "2025-01-01" },
    },
    qualifications: [
      { name: "Qual1", status: "success" },
      { name: "Qual2", status: "failure", message: "Error message" },
    ],
    actions: [
      { id: "act-1", name: "create", state: "queued" },
      { id: "act-2", name: "update", state: "running" },
      { id: "act-3", name: "delete", state: "failed" },
    ],
  };

  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  try {
    await cacheComponentData(cacheData, tempFile, ctx.logger);

    const content = await Deno.readTextFile(tempFile);
    const parsed = JSON.parse(content);

    assertEquals(parsed.attributes["/si/nested"].deep.value, 123);
    assertEquals(parsed.attributes["/domain/array"].length, 3);
    assertEquals(parsed.qualifications.length, 2);
    assertEquals(parsed.qualifications[1].message, "Error message");
    assertEquals(parsed.actions.length, 3);
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("ComponentGetCache - structure validation", () => {
  const cache: ComponentGetCache = {
    componentId: "test-id",
    schemaId: "test-schema",
    schemaName: "Test Schema",
    resourceId: "test-resource",
    toDelete: false,
    canBeUpgraded: false,
    qualified: true,
    attributes: {
      "/si/name": "test",
      "/domain/key": "value",
    },
    resourceData: { data: "value" },
    qualifications: [
      { name: "test", status: "success", message: "ok" },
    ],
    actions: [
      { id: "1", name: "create", state: "queued" },
    ],
  };

  // Verify structure is valid
  assertEquals(cache.componentId, "test-id");
  assertEquals(cache.schemaId, "test-schema");
  assertEquals(cache.schemaName, "Test Schema");
  assertEquals(cache.resourceId, "test-resource");
  assertEquals(cache.qualified, true);
  assertExists(cache.attributes);
  assertEquals(cache.attributes["/si/name"], "test");
  assertEquals(cache.attributes["/domain/key"], "value");
  assertEquals(cache.qualifications.length, 1);
  assertEquals(cache.actions.length, 1);
});

Deno.test("cacheComponentData() - handles empty strings and special characters", async () => {
  const ctx = Context.instance();

  const cacheData: ComponentGetCache = {
    componentId: "comp-special",
    schemaId: "schema-special",
    schemaName: "Schema with \"quotes\" and 'apostrophes'",
    toDelete: true,
    canBeUpgraded: false,
    qualified: false,
    attributes: {
      "/si/name": "test\nwith\nnewlines",
      "/si/special": "unicode: 🚀",
      "/domain/empty": "",
      "/domain/whitespace": "   ",
    },
    qualifications: [
      {
        name: "Qualification with spaces",
        status: "failure",
        message: "Error: Something went wrong!\nLine 2",
      },
    ],
    actions: [],
  };

  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  try {
    await cacheComponentData(cacheData, tempFile, ctx.logger);

    const content = await Deno.readTextFile(tempFile);
    const parsed = JSON.parse(content);

    assertEquals(parsed.schemaName, "Schema with \"quotes\" and 'apostrophes'");
    assertEquals(parsed.attributes["/si/special"], "unicode: 🚀");
    assertEquals(parsed.qualifications[0].message?.includes("\n"), true);
  } finally {
    await Deno.remove(tempFile);
  }
});

Deno.test("cacheComponentData() - handles undefined optional fields in YAML", async () => {
  const ctx = Context.instance();

  // Component with undefined optional fields (resourceId, resourceData)
  const cacheData: ComponentGetCache = {
    componentId: "comp-no-resource",
    schemaId: "schema-123",
    schemaName: "Test Schema",
    toDelete: false,
    canBeUpgraded: false,
    qualified: true,
    attributes: {
      "/si/name": "test-component",
    },
    qualifications: [
      { name: "test", status: "success" }, // no message (undefined)
    ],
    actions: [],
  };

  const tempFile = await Deno.makeTempFile({ suffix: ".yaml" });
  try {
    await cacheComponentData(cacheData, tempFile, ctx.logger);

    const content = await Deno.readTextFile(tempFile);

    // Verify YAML was written successfully
    assertExists(content);
    assertEquals(content.includes("componentId: comp-no-resource"), true);
    assertEquals(content.includes("qualified: true"), true);
    // resourceId and resourceData should not appear since they're undefined
    assertEquals(content.includes("resourceId"), false);
    assertEquals(content.includes("resourceData"), false);
  } finally {
    await Deno.remove(tempFile);
  }
});
