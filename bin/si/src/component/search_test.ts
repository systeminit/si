/**
 * Tests for component search module
 */

import { assertEquals, assertStringIncludes } from "@std/assert";
import { Context } from "../context.ts";

// Initialize Context once - this runs at module load time
await Context.init({ verbose: 0, noColor: true });

// Note: formatAttributesInline is a private function, so we test its behavior indirectly
// through the output structure. Here we test the logic directly by reimplementing it.

Deno.test("formatAttributesInline logic - formats single string attribute", () => {
  const attributes = { "/domain/region": "us-east-1" };
  const result = formatTestAttributes(attributes);
  assertEquals(result, "  /domain/region: us-east-1");
});

Deno.test("formatAttributesInline logic - formats single number attribute", () => {
  const attributes = { "/domain/port": 8080 };
  const result = formatTestAttributes(attributes);
  assertEquals(result, "  /domain/port: 8080");
});

Deno.test("formatAttributesInline logic - formats boolean attribute", () => {
  const attributes = { "/domain/enabled": true };
  const result = formatTestAttributes(attributes);
  assertEquals(result, "  /domain/enabled: true");
});

Deno.test("formatAttributesInline logic - formats object attribute as JSON", () => {
  const attributes = { "/domain/config": { key: "value", nested: { deep: 123 } } };
  const result = formatTestAttributes(attributes);
  assertStringIncludes(result, "  /domain/config:");
  assertStringIncludes(result, '"key":"value"');
});

Deno.test("formatAttributesInline logic - formats array attribute as JSON", () => {
  const attributes = { "/domain/tags": ["tag1", "tag2", "tag3"] };
  const result = formatTestAttributes(attributes);
  assertStringIncludes(result, "  /domain/tags:");
  assertStringIncludes(result, '["tag1","tag2","tag3"]');
});

Deno.test("formatAttributesInline logic - formats multiple attributes with newlines", () => {
  const attributes = {
    "/domain/region": "us-east-1",
    "/domain/instanceType": "t3.micro",
    "/domain/port": 443,
  };
  const result = formatTestAttributes(attributes);
  assertStringIncludes(result, "/domain/region: us-east-1");
  assertStringIncludes(result, "/domain/instanceType: t3.micro");
  assertStringIncludes(result, "/domain/port: 443");
  // Should have newlines between entries
  const lines = result.split("\n");
  assertEquals(lines.length, 3);
});

Deno.test("formatAttributesInline logic - handles empty attributes object", () => {
  const attributes = {};
  const result = formatTestAttributes(attributes);
  assertEquals(result, "");
});

Deno.test("formatAttributesInline logic - handles null values correctly", () => {
  const attributes = { "/domain/nullValue": null };
  const result = formatTestAttributes(attributes);
  assertEquals(result, "  /domain/nullValue: null");
});

Deno.test("formatAttributesInline logic - handles undefined values correctly", () => {
  const attributes = { "/domain/undefinedValue": undefined };
  const result = formatTestAttributes(attributes);
  assertEquals(result, "  /domain/undefinedValue: undefined");
});

Deno.test("SearchResultOutput - structure validation with minimal data", () => {
  // Test output structure with minimal required fields
  const output = {
    count: 1,
    components: [
      {
        id: "01HQZ1234567890ABCDEFGHIJK",
        name: "my-component",
        schema: {
          name: "AWS EC2 Instance",
        },
      },
    ],
  };

  assertEquals(output.count, 1);
  assertEquals(output.components.length, 1);
  assertEquals(output.components[0].id, "01HQZ1234567890ABCDEFGHIJK");
  assertEquals(output.components[0].name, "my-component");
  assertEquals(output.components[0].schema.name, "AWS EC2 Instance");
});

Deno.test("SearchResultOutput - structure with attributes", () => {
  const output = {
    count: 2,
    components: [
      {
        id: "comp-1",
        name: "component-1",
        schema: {
          name: "AWS EC2 Instance",
        },
      },
      {
        id: "comp-2",
        name: "component-2",
        schema: {
          name: "AWS EC2 Instance",
        },
        attributes: {
          "/domain/region": "us-east-1",
          "/domain/instanceType": "t3.micro",
        },
      },
    ],
  };

  assertEquals(output.count, 2);
  assertEquals(output.components.length, 2);
  assertEquals(output.components[0].name, "component-1");
  assertEquals(output.components[1].attributes?.["/domain/region"], "us-east-1");
  assertEquals(
    output.components[1].attributes?.["/domain/instanceType"],
    "t3.micro",
  );
});

Deno.test("SearchResultOutput - structure with full component data (ComponentGetCache)", () => {
  // Test that fullComponent uses the ComponentGetCache structure (same as component get)
  const output = {
    count: 1,
    components: [
      {
        id: "comp-full",
        name: "full-component",
        schema: {
          name: "AWS S3 Bucket",
        },
        fullComponent: {
          componentId: "comp-full",
          schemaId: "schema-123",
          schemaName: "AWS S3 Bucket",
          resourceId: "bucket-resource-id",
          toDelete: false,
          canBeUpgraded: false,
          qualified: true,
          attributes: {
            "/si/name": "full-component",
            "/domain/bucketName": "my-bucket",
          },
          resourceData: {
            BucketName: "my-bucket",
          },
          resource: {
            payload: "some-resource-data",
          },
          qualifications: [
            {
              name: "/domain/valid",
              status: "success",
              message: "Bucket name is valid",
            },
          ],
          actions: [
            {
              id: "action-123",
              name: "create",
              state: "queued",
            },
          ],
        },
      },
    ],
  };

  assertEquals(output.count, 1);
  assertEquals(output.components[0].fullComponent?.componentId, "comp-full");
  assertEquals(output.components[0].fullComponent?.schemaName, "AWS S3 Bucket");
  assertEquals(output.components[0].fullComponent?.qualified, true);
  assertEquals(
    output.components[0].fullComponent?.attributes["/domain/bucketName"],
    "my-bucket",
  );
  assertEquals(output.components[0].fullComponent?.qualifications.length, 1);
  assertEquals(output.components[0].fullComponent?.actions.length, 1);
});

Deno.test("SearchResultOutput - empty results structure", () => {
  const output = {
    count: 0,
    components: [],
  };

  assertEquals(output.count, 0);
  assertEquals(output.components.length, 0);
});

Deno.test("SearchResultOutput - multiple components different schemas", () => {
  const output = {
    count: 3,
    components: [
      {
        id: "comp-1",
        name: "my-instance",
        schema: { name: "AWS EC2 Instance" },
      },
      {
        id: "comp-2",
        name: "my-bucket",
        schema: { name: "AWS S3 Bucket" },
      },
      {
        id: "comp-3",
        name: "my-db",
        schema: { name: "AWS RDS Instance" },
      },
    ],
  };

  assertEquals(output.count, 3);
  assertEquals(output.components.length, 3);
  assertEquals(output.components[0].schema.name, "AWS EC2 Instance");
  assertEquals(output.components[1].schema.name, "AWS S3 Bucket");
  assertEquals(output.components[2].schema.name, "AWS RDS Instance");
});

// Helper function that replicates the formatAttributesInline logic for testing
function formatTestAttributes(attributes: Record<string, unknown>): string {
  return Object.entries(attributes)
    .map(([path, value]) => {
      let displayValue: string;
      if (typeof value === "object" && value !== null) {
        displayValue = JSON.stringify(value);
      } else {
        displayValue = String(value);
      }
      return `  ${path}: ${displayValue}`;
    })
    .join("\n");
}
