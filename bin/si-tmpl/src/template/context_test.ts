import { assertEquals, assertExists } from "@std/assert";
import { Context } from "../context.ts";
import {
  componentViewToTemplateComponent,
  type ComponentViewV1,
  filterComponentAttributes,
  type TemplateComponent,
  TemplateContext,
} from "./context.ts";
import { z } from "zod";

// Clear SI_API_TOKEN to ensure clean test environment
Deno.env.delete("SI_API_TOKEN");

// Initialize Context once - this runs at module load time
await Context.init({ verbose: 0, noColor: true });

Deno.test("TemplateContext - initialization with default values", () => {
  const templatePath = "/path/to/my-template.ts";
  const options = { key: "test-key-123" };

  const ctx = new TemplateContext(templatePath, options);

  // Verify logger is assigned
  assertExists(ctx.logger);

  // Verify default name is extracted from template path
  assertEquals(ctx.name(), "my-template");

  // Verify default changeSet is name-key format
  assertEquals(ctx.changeSet(), "my-template-test-key-123");
});

Deno.test("TemplateContext - name extraction from various paths", () => {
  const testCases = [
    { path: "/absolute/path/to/template.ts", expected: "template" },
    { path: "relative/path/template.ts", expected: "template" },
    { path: "simple-template.ts", expected: "simple-template" },
    {
      path: "/complex/path/my-awesome-template.ts",
      expected: "my-awesome-template",
    },
    {
      path: "https://complex.com/path/my-awesome-template.ts",
      expected: "my-awesome-template",
    },
  ];

  for (const testCase of testCases) {
    const ctx = new TemplateContext(testCase.path, { key: "key" });
    assertEquals(
      ctx.name(),
      testCase.expected,
      `Failed for path: ${testCase.path}`,
    );
  }
});

Deno.test("TemplateContext - name() getter returns current name", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const currentName = ctx.name();
  assertEquals(currentName, "template");
});

Deno.test("TemplateContext - name() setter updates name", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initial name
  assertEquals(ctx.name(), "template");

  // Update name
  ctx.name("new-template-name");

  // Verify name was updated
  assertEquals(ctx.name(), "new-template-name");
});

Deno.test("TemplateContext - changeSet() getter returns current changeSet", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "my-key" });

  const currentChangeSet = ctx.changeSet();
  assertEquals(currentChangeSet, "template-my-key");
});

Deno.test("TemplateContext - changeSet() setter updates changeSet", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initial changeSet
  assertEquals(ctx.changeSet(), "template-key");

  // Update changeSet
  ctx.changeSet("custom-changeset-name");

  // Verify changeSet was updated
  assertEquals(ctx.changeSet(), "custom-changeset-name");
});

Deno.test("TemplateContext - invocation key affects default changeSet", () => {
  const ctx1 = new TemplateContext("/path/template.ts", { key: "key-1" });
  const ctx2 = new TemplateContext("/path/template.ts", { key: "key-2" });

  // Different keys should produce different default changeSets
  assertEquals(ctx1.changeSet(), "template-key-1");
  assertEquals(ctx2.changeSet(), "template-key-2");
});

Deno.test("TemplateContext - name change doesn't affect changeSet", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const initialChangeSet = ctx.changeSet();
  assertEquals(initialChangeSet, "template-key");

  // Change the name
  ctx.name("new-name");

  // changeSet should remain unchanged
  assertEquals(ctx.changeSet(), "template-key");
});

Deno.test("TemplateContext - search() defaults to empty array", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const currentSearch = ctx.search();
  assertEquals(currentSearch, []);
});

Deno.test("TemplateContext - search() getter returns current search array", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const currentSearch = ctx.search();
  assertEquals(currentSearch, []);
});

Deno.test("TemplateContext - search() setter updates search array", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initial search is empty
  assertEquals(ctx.search(), []);

  // Update search
  ctx.search(["term1", "term2"]);

  // Verify search was updated
  assertEquals(ctx.search(), ["term1", "term2"]);
});

Deno.test("TemplateContext - search() can be set to multiple search strings", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const searchTerms = ["aws", "ec2", "kubernetes", "docker"];
  ctx.search(searchTerms);

  assertEquals(ctx.search(), searchTerms);
});

Deno.test("TemplateContext - search() can be set to empty array after having values", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set some values
  ctx.search(["term1", "term2"]);
  assertEquals(ctx.search(), ["term1", "term2"]);

  // Reset to empty
  ctx.search([]);
  assertEquals(ctx.search(), []);
});

Deno.test("TemplateContext - namePattern() defaults to undefined", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const currentPattern = ctx.namePattern();
  assertEquals(currentPattern, undefined);
});

Deno.test("TemplateContext - namePattern() getter returns current pattern", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initially undefined
  assertEquals(ctx.namePattern(), undefined);
});

Deno.test("TemplateContext - namePattern() setter updates pattern", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initial pattern is undefined
  assertEquals(ctx.namePattern(), undefined);

  // Update pattern
  const pattern = {
    pattern: /prod-(.+)/g,
    replacement: "sandbox-$1",
  };
  ctx.namePattern([pattern]);

  // Verify pattern was updated
  const retrieved = ctx.namePattern();
  assertEquals(retrieved?.[0].pattern.source, "prod-(.+)");
  assertEquals(retrieved?.[0].pattern.flags, "g");
  assertEquals(retrieved?.[0].replacement, "sandbox-$1");
});

Deno.test("TemplateContext - namePattern() can be used with String.replace()", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set the pattern (s/prod-(.+)/sandbox-$1/g equivalent)
  ctx.namePattern([{
    pattern: /prod-(.+)/g,
    replacement: "sandbox-$1",
  }]);

  const patterns = ctx.namePattern();
  assertExists(patterns);

  // Test the pattern transformation
  const input = "prod-database";
  const output = input.replace(patterns[0].pattern, patterns[0].replacement);
  assertEquals(output, "sandbox-database");
});

Deno.test("TemplateContext - namePattern() with multiple capture groups", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set pattern with multiple capture groups
  ctx.namePattern([{
    pattern: /(\w+)-(\w+)-(\d+)/g,
    replacement: "$3-$1-$2",
  }]);

  const patterns = ctx.namePattern();
  assertExists(patterns);

  // Test transformation
  const input = "prod-server-001";
  const output = input.replace(patterns[0].pattern, patterns[0].replacement);
  assertEquals(output, "001-prod-server");
});

Deno.test("TemplateContext - inputs() defaults to undefined", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const currentInputs = ctx.inputs();
  assertEquals(currentInputs, undefined);
});

Deno.test("TemplateContext - inputs() getter returns current schema", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initially undefined
  assertEquals(ctx.inputs(), undefined);
});

Deno.test("TemplateContext - inputs() setter updates schema", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initial schema is undefined
  assertEquals(ctx.inputs(), undefined);

  // Update schema
  const schema = z.object({
    name: z.string(),
    count: z.number(),
  });
  ctx.inputs(schema);

  // Verify schema was updated
  const retrieved = ctx.inputs();
  assertExists(retrieved);
  assertEquals(retrieved, schema);
});

Deno.test("TemplateContext - inputs() schema with defaults", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set schema with default values
  const schema = z.object({
    name: z.string().default("test"),
    count: z.number().default(1),
    enabled: z.boolean().default(true),
  });
  ctx.inputs(schema);

  const retrieved = ctx.inputs();
  assertExists(retrieved);

  // Test that defaults are applied
  const result = retrieved.parse({});
  assertEquals(result, { name: "test", count: 1, enabled: true });
});

Deno.test("TemplateContext - inputs() schema validation with parse()", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set schema
  const schema = z.object({
    environment: z.string().default("production"),
    replicas: z.number().default(3),
  });
  ctx.inputs(schema);

  const retrieved = ctx.inputs();
  assertExists(retrieved);

  // Test validation with valid data
  const validData = { environment: "staging", replicas: 5 };
  const result = retrieved.parse(validData);
  assertEquals(result, { environment: "staging", replicas: 5 });
});

Deno.test("TemplateContext - inputs() schema with optional and required fields", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set schema with mix of required, optional, and default fields
  const schema = z.object({
    name: z.string(), // required
    region: z.string().optional(), // optional
    port: z.number().default(8080), // default
  });
  ctx.inputs(schema);

  const retrieved = ctx.inputs();
  assertExists(retrieved);

  // Test with only required field
  const result1 = retrieved.parse({ name: "test-service" });
  assertEquals(result1, { name: "test-service", port: 8080 });

  // Test with all fields
  const result2 = retrieved.parse({
    name: "test-service",
    region: "us-east",
    port: 9000,
  });
  assertEquals(result2, {
    name: "test-service",
    region: "us-east",
    port: 9000,
  });
});

Deno.test("TemplateContext - inputs() schema applies defaults to incomplete data", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set schema with multiple defaults
  const schema = z.object({
    environment: z.string().default("production"),
    replicas: z.number().default(3),
    autoscaling: z.boolean().default(false),
    region: z.string().default("us-east-1"),
  });
  ctx.inputs(schema);

  const retrieved = ctx.inputs();
  assertExists(retrieved);

  // Parse incomplete data - should fill in defaults
  const partialData = { environment: "staging", replicas: 5 };
  const result = retrieved.parse(partialData);
  assertEquals(result, {
    environment: "staging",
    replicas: 5,
    autoscaling: false,
    region: "us-east-1",
  });
});

Deno.test("TemplateContext - transform() defaults to undefined", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const currentTransform = ctx.transform();
  assertEquals(currentTransform, undefined);
});

Deno.test("TemplateContext - transform() getter returns current function", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initially undefined
  assertEquals(ctx.transform(), undefined);
});

Deno.test("TemplateContext - transform() setter updates function", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initial transform is undefined
  assertEquals(ctx.transform(), undefined);

  // Update transform
  const transformFn = (workingSet: TemplateComponent[]) => workingSet;
  ctx.transform(transformFn);

  // Verify transform was updated
  const retrieved = ctx.transform();
  assertExists(retrieved);
  assertEquals(retrieved, transformFn);
});

Deno.test("TemplateContext - transform() function can be called with array", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set a simple identity transform
  ctx.transform((workingSet) => workingSet);

  const transform = ctx.transform();
  assertExists(transform);

  // Test calling the function
  const testComponents: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "test-component",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
  ];

  const result = await transform(testComponents);
  assertEquals(result, testComponents);
});

Deno.test("TemplateContext - transform() with filter logic", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set a filter transform
  ctx.transform((workingSet) => {
    return workingSet.filter((c) => c.name.startsWith("prod-"));
  });

  const transform = ctx.transform();
  assertExists(transform);

  // Test with multiple components
  const testComponents: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "prod-database",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
    {
      id: "comp-2",
      name: "dev-database",
      schemaId: "schema-1",
      resourceId: "res-2",
      attributes: {},
    },
    {
      id: "comp-3",
      name: "prod-api",
      schemaId: "schema-2",
      resourceId: "res-3",
      attributes: {},
    },
  ];

  const result = await transform(testComponents);
  assertEquals(result.length, 2);
  assertEquals(result[0].name, "prod-database");
  assertEquals(result[1].name, "prod-api");
});

Deno.test("TemplateContext - transform() with map logic", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set a map transform
  ctx.transform((workingSet) => {
    return workingSet.map((c) => ({
      ...c,
      name: c.name.replace("prod-", "staging-"),
    }));
  });

  const transform = ctx.transform();
  assertExists(transform);

  // Test transformation
  const testComponents: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "prod-database",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
  ];

  const result = await transform(testComponents);
  assertEquals(result.length, 1);
  assertEquals(result[0].name, "staging-database");
  assertEquals(result[0].id, "comp-1"); // Other properties unchanged
});

Deno.test("TemplateContext - apiConfig() returns undefined when SI_API_TOKEN not set", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // When SI_API_TOKEN is not set, apiConfig should be undefined
  assertEquals(ctx.apiConfig(), undefined);
});

Deno.test("TemplateContext - workspaceId() returns undefined when SI_API_TOKEN not set", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // When SI_API_TOKEN is not set, workspaceId should be undefined
  assertEquals(ctx.workspaceId(), undefined);
});

Deno.test("TemplateContext - userId() returns undefined when SI_API_TOKEN not set", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // When SI_API_TOKEN is not set, userId should be undefined
  assertEquals(ctx.userId(), undefined);
});

Deno.test("TemplateContext - baseline() defaults to undefined", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const currentBaseline = ctx.baseline();
  assertEquals(currentBaseline, undefined);
});

Deno.test("TemplateContext - baseline() getter returns current baseline", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initially undefined
  assertEquals(ctx.baseline(), undefined);
});

Deno.test("TemplateContext - baseline() setter updates baseline", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initial baseline is undefined
  assertEquals(ctx.baseline(), undefined);

  // Update baseline
  const testComponents: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "test-component",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
  ];
  ctx.baseline(testComponents);

  // Verify baseline was updated
  const retrieved = ctx.baseline();
  assertExists(retrieved);
  assertEquals(retrieved.length, 1);
  assertEquals(retrieved[0].id, "comp-1");
});

Deno.test("TemplateContext - baseline() can be set to empty array", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  ctx.baseline([]);

  const retrieved = ctx.baseline();
  assertExists(retrieved);
  assertEquals(retrieved.length, 0);
});

Deno.test("TemplateContext - baseline() can be updated after initial set", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set initial baseline
  const initial: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "component-1",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
  ];
  ctx.baseline(initial);
  assertEquals(ctx.baseline()?.length, 1);

  // Update with new baseline
  const updated: TemplateComponent[] = [
    {
      id: "comp-2",
      name: "component-2",
      schemaId: "schema-2",
      resourceId: "res-2",
      attributes: {},
    },
    {
      id: "comp-3",
      name: "component-3",
      schemaId: "schema-3",
      resourceId: "res-3",
      attributes: {},
    },
  ];
  ctx.baseline(updated);

  const retrieved = ctx.baseline();
  assertExists(retrieved);
  assertEquals(retrieved.length, 2);
  assertEquals(retrieved[0].id, "comp-2");
  assertEquals(retrieved[1].id, "comp-3");
});

Deno.test("TemplateContext - workingSet() defaults to undefined", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const currentWorkingSet = ctx.workingSet();
  assertEquals(currentWorkingSet, undefined);
});

Deno.test("TemplateContext - workingSet() getter returns current working set", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initially undefined
  assertEquals(ctx.workingSet(), undefined);
});

Deno.test("TemplateContext - workingSet() setter updates working set", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Initial working set is undefined
  assertEquals(ctx.workingSet(), undefined);

  // Update working set
  const testComponents: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "test-component",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
  ];
  ctx.workingSet(testComponents);

  // Verify working set was updated
  const retrieved = ctx.workingSet();
  assertExists(retrieved);
  assertEquals(retrieved.length, 1);
  assertEquals(retrieved[0].id, "comp-1");
});

Deno.test("TemplateContext - workingSet() can be set to empty array", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  ctx.workingSet([]);

  const retrieved = ctx.workingSet();
  assertExists(retrieved);
  assertEquals(retrieved.length, 0);
});

Deno.test("TemplateContext - workingSet() can store multiple components", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  const testComponents: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "component-1",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
    {
      id: "comp-2",
      name: "component-2",
      schemaId: "schema-2",
      resourceId: "res-2",
      attributes: {},
    },
  ];
  ctx.workingSet(testComponents);

  const retrieved = ctx.workingSet();
  assertExists(retrieved);
  assertEquals(retrieved.length, 2);
  assertEquals(retrieved[0].id, "comp-1");
  assertEquals(retrieved[1].id, "comp-2");
});

Deno.test("TemplateContext - workingSet() can be updated after initial set", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Set initial working set
  const initial: TemplateComponent[] = [
    {
      id: "comp-1",
      name: "component-1",
      schemaId: "schema-1",
      resourceId: "res-1",
      attributes: {},
    },
  ];
  ctx.workingSet(initial);
  assertEquals(ctx.workingSet()?.length, 1);

  // Update with new working set
  const updated: TemplateComponent[] = [
    {
      id: "comp-2",
      name: "component-2",
      schemaId: "schema-2",
      resourceId: "res-2",
      attributes: {},
    },
  ];
  ctx.workingSet(updated);

  const retrieved = ctx.workingSet();
  assertExists(retrieved);
  assertEquals(retrieved.length, 1);
  assertEquals(retrieved[0].id, "comp-2");
});

Deno.test("filterComponentAttributes() - filters /si attributes", () => {
  const attributes = {
    "/si/name": "test-name",
    "/si/color": "#ff0000",
    "/resource/data": "should-be-removed",
    "other": "should-be-removed",
  };

  const filtered = filterComponentAttributes(attributes);

  assertEquals(Object.keys(filtered).length, 2);
  assertEquals(filtered["/si/name"], "test-name");
  assertEquals(filtered["/si/color"], "#ff0000");
  assertEquals(filtered["/resource/data"], undefined);
  assertEquals(filtered["other"], undefined);
});

Deno.test("filterComponentAttributes() - filters /domain attributes", () => {
  const attributes = {
    "/domain/region": "us-west-2",
    "/domain/tags": ["prod", "web"],
    "/resource/data": "should-be-removed",
    "other": "should-be-removed",
  };

  const filtered = filterComponentAttributes(attributes);

  assertEquals(Object.keys(filtered).length, 2);
  assertEquals(filtered["/domain/region"], "us-west-2");
  assertEquals(filtered["/domain/tags"], ["prod", "web"]);
  assertEquals(filtered["/resource/data"], undefined);
});

Deno.test("filterComponentAttributes() - filters /secrets attributes", () => {
  const attributes = {
    "/secrets/password": "secret123",
    "/secrets/apiKey": "key-abc",
    "/resource/data": "should-be-removed",
    "other": "should-be-removed",
  };

  const filtered = filterComponentAttributes(attributes);

  assertEquals(Object.keys(filtered).length, 2);
  assertEquals(filtered["/secrets/password"], "secret123");
  assertEquals(filtered["/secrets/apiKey"], "key-abc");
  assertEquals(filtered["/resource/data"], undefined);
});

Deno.test("filterComponentAttributes() - handles mixed attributes", () => {
  const attributes = {
    "/si/name": "component",
    "/domain/env": "prod",
    "/secrets/token": "secret",
    "/resource/data": "removed",
    "/code/info": "removed",
    "plain": "removed",
  };

  const filtered = filterComponentAttributes(attributes);

  assertEquals(Object.keys(filtered).length, 3);
  assertEquals(filtered["/si/name"], "component");
  assertEquals(filtered["/domain/env"], "prod");
  assertEquals(filtered["/secrets/token"], "secret");
  assertEquals(filtered["/resource/data"], undefined);
  assertEquals(filtered["/code/info"], undefined);
  assertEquals(filtered["plain"], undefined);
});

Deno.test("filterComponentAttributes() - handles empty attributes", () => {
  const filtered = filterComponentAttributes({});
  assertEquals(Object.keys(filtered).length, 0);
});

Deno.test("filterComponentAttributes() - handles attributes with no matching prefixes", () => {
  const attributes = {
    "/resource/data": "removed",
    "/code/info": "removed",
    "plain": "removed",
  };

  const filtered = filterComponentAttributes(attributes);
  assertEquals(Object.keys(filtered).length, 0);
});

Deno.test("componentViewToTemplateComponent() - converts full component", () => {
  const componentView: ComponentViewV1 = {
    id: "comp-123",
    name: "test-component",
    schemaId: "schema-456",
    resourceId: "res-789",
    attributes: {
      "/si/name": "My Component",
      "/domain/region": "us-east-1",
      "/secrets/key": "secret",
      "/resource/data": "should-be-removed",
      "other": "should-be-removed",
    },
    canBeUpgraded: true,
    connections: [],
    domainProps: [],
    resourceProps: [],
    schemaVariantId: "variant-123",
    toDelete: false,
    views: [],
  };

  const template = componentViewToTemplateComponent(componentView);

  assertEquals(template.id, "comp-123");
  assertEquals(template.name, "test-component");
  assertEquals(template.schemaId, "schema-456");
  assertEquals(template.resourceId, "res-789");
  assertEquals(Object.keys(template.attributes).length, 3);
  assertEquals(template.attributes["/si/name"], "My Component");
  assertEquals(template.attributes["/domain/region"], "us-east-1");
  assertEquals(template.attributes["/secrets/key"], "secret");
  assertEquals(template.attributes["/resource/data"], undefined);
});

Deno.test("componentViewToTemplateComponent() - handles empty attributes", () => {
  const componentView: ComponentViewV1 = {
    id: "comp-123",
    name: "test-component",
    schemaId: "schema-456",
    resourceId: "res-789",
    attributes: {},
    canBeUpgraded: false,
    connections: [],
    domainProps: [],
    resourceProps: [],
    schemaVariantId: "variant-123",
    toDelete: false,
    views: [],
  };

  const template = componentViewToTemplateComponent(componentView);

  assertEquals(template.id, "comp-123");
  assertEquals(template.attributes, {});
});

Deno.test("componentViewToTemplateComponent() - handles undefined attributes", () => {
  const componentView: ComponentViewV1 = {
    id: "comp-123",
    name: "test-component",
    schemaId: "schema-456",
    resourceId: "res-789",
    // deno-lint-ignore no-explicit-any
    attributes: undefined as any,
    canBeUpgraded: false,
    connections: [],
    domainProps: [],
    resourceProps: [],
    schemaVariantId: "variant-123",
    toDelete: false,
    views: [],
  };

  const template = componentViewToTemplateComponent(componentView);

  assertEquals(template.id, "comp-123");
  assertEquals(template.attributes, {});
});

Deno.test("TemplateContext.setSubscription() - idempotent for direct $source subscription", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Create a test component
  const component: TemplateComponent = {
    id: "comp-123",
    schemaId: "schema-456",
    name: "test-component",
    resourceId: "res-789",
    attributes: {},
  };

  // Use a valid ULID format (26 alphanumeric characters) so API config is not needed
  const targetUlid = "01HQZX3Y4N5P6Q7R8S9T0V1W2X";

  // Set a subscription for the first time
  await ctx.setSubscription(component, "/domain/config", {
    kind: "$source",
    component: targetUlid,
    path: "/domain/output",
  });

  // Verify subscription was set
  assertEquals(component.attributes["/domain/config"], {
    $source: {
      component: targetUlid,
      path: "/domain/output",
    },
  });

  // Store the original reference
  const originalSubscription = component.attributes["/domain/config"];

  // Call setSubscription again with the same values
  await ctx.setSubscription(component, "/domain/config", {
    kind: "$source",
    component: targetUlid,
    path: "/domain/output",
  });

  // Verify that the subscription object reference didn't change (idempotent)
  assertEquals(component.attributes["/domain/config"], originalSubscription);
  assertEquals(component.attributes["/domain/config"], {
    $source: {
      component: targetUlid,
      path: "/domain/output",
    },
  });
});

Deno.test("TemplateContext.setSubscription() - updates when subscription differs", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Use valid ULID formats (26 alphanumeric characters)
  const oldTargetUlid = "01HQZX3Y4N5P6Q7R8S9T0V1W2X";
  const newTargetUlid = "01HQZX3Y4N5P6Q7R8S9T0V1W3Y";

  // Create a test component with an existing subscription
  const component: TemplateComponent = {
    id: "comp-123",
    schemaId: "schema-456",
    name: "test-component",
    resourceId: "res-789",
    attributes: {
      "/domain/config": {
        $source: {
          component: oldTargetUlid,
          path: "/domain/old-output",
        },
      },
    },
  };

  // Set a subscription to a different target
  await ctx.setSubscription(component, "/domain/config", {
    kind: "$source",
    component: newTargetUlid,
    path: "/domain/new-output",
  });

  // Verify subscription was updated
  assertEquals(component.attributes["/domain/config"], {
    $source: {
      component: newTargetUlid,
      path: "/domain/new-output",
    },
  });
});

Deno.test("TemplateContext.setSubscription() - idempotent with func parameter", async () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Use a valid ULID format (26 alphanumeric characters)
  const targetUlid = "01HQZX3Y4N5P6Q7R8S9T0V1W2X";

  // Create a test component
  const component: TemplateComponent = {
    id: "comp-123",
    schemaId: "schema-456",
    name: "test-component",
    resourceId: "res-789",
    attributes: {},
  };

  // Set a subscription with a func parameter
  await ctx.setSubscription(component, "/domain/config", {
    kind: "$source",
    component: targetUlid,
    path: "/domain/output",
    func: "si:normalizeToArray",
  });

  // Verify subscription was set
  assertEquals(component.attributes["/domain/config"], {
    $source: {
      component: targetUlid,
      path: "/domain/output",
      func: "si:normalizeToArray",
    },
  });

  // Store the original reference
  const originalSubscription = component.attributes["/domain/config"];

  // Call setSubscription again with the same values including func
  await ctx.setSubscription(component, "/domain/config", {
    kind: "$source",
    component: targetUlid,
    path: "/domain/output",
    func: "si:normalizeToArray",
  });

  // Verify that the subscription object reference didn't change (idempotent)
  assertEquals(component.attributes["/domain/config"], originalSubscription);
});

Deno.test("TemplateContext - search cache is initialized", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Access the private cache through a type assertion to verify it exists
  // deno-lint-ignore no-explicit-any
  const ctxWithCache = ctx as any;
  assertExists(ctxWithCache._searchCache);
  assertEquals(ctxWithCache._searchCache.size, 0);
});

Deno.test("TemplateContext - component cache is initialized", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "key" });

  // Access the private cache through a type assertion to verify it exists
  // deno-lint-ignore no-explicit-any
  const ctxWithCache = ctx as any;
  assertExists(ctxWithCache._componentCache);
  assertEquals(ctxWithCache._componentCache.size, 0);
});

Deno.test("TemplateContext - caches are independent between instances", () => {
  const ctx1 = new TemplateContext("/path/to/template1.ts", { key: "key1" });
  const ctx2 = new TemplateContext("/path/to/template2.ts", { key: "key2" });

  // Access the private caches through type assertions
  // deno-lint-ignore no-explicit-any
  const ctx1WithCache = ctx1 as any;
  // deno-lint-ignore no-explicit-any
  const ctx2WithCache = ctx2 as any;

  // Verify they are different Map instances
  const searchCache1 = ctx1WithCache._searchCache;
  const searchCache2 = ctx2WithCache._searchCache;
  const componentCache1 = ctx1WithCache._componentCache;
  const componentCache2 = ctx2WithCache._componentCache;

  // Add an item to ctx1's caches
  searchCache1.set("test-key", { components: [] });
  componentCache1.set("comp-id", { component: {} });

  // Verify ctx2's caches are still empty (independent)
  assertEquals(searchCache1.size, 1);
  assertEquals(searchCache2.size, 0);
  assertEquals(componentCache1.size, 1);
  assertEquals(componentCache2.size, 0);
});

Deno.test("deleteAttribute - deletes exact string match", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "value" },
      "/domain/other": { other: "data" },
      "/si/tags/env": "production",
    },
  };

  ctx.deleteAttribute(component, "/domain/config");

  assertEquals(Object.keys(component.attributes).length, 2);
  assertEquals(component.attributes["/domain/config"], undefined);
  assertEquals(component.attributes["/domain/other"], { other: "data" });
  assertEquals(component.attributes["/si/tags/env"], "production");
});

Deno.test("deleteAttribute - does nothing when no match found", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "value" },
      "/si/tags/env": "production",
    },
  };

  ctx.deleteAttribute(component, "/domain/nonexistent");

  assertEquals(Object.keys(component.attributes).length, 2);
  assertEquals(component.attributes["/domain/config"], { key: "value" });
  assertEquals(component.attributes["/si/tags/env"], "production");
});

Deno.test("deleteAttribute - deletes multiple attributes with regex", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/temp1": "value1",
      "/domain/temp2": "value2",
      "/domain/config": { key: "value" },
      "/si/tags/env": "production",
    },
  };

  ctx.deleteAttribute(component, /^\/domain\/temp/);

  assertEquals(Object.keys(component.attributes).length, 2);
  assertEquals(component.attributes["/domain/temp1"], undefined);
  assertEquals(component.attributes["/domain/temp2"], undefined);
  assertEquals(component.attributes["/domain/config"], { key: "value" });
  assertEquals(component.attributes["/si/tags/env"], "production");
});

Deno.test("deleteAttribute - deletes with regex partial match", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/testing/config": "value1",
      "/domain/production/config": "value2",
      "/si/tags/test": "value3",
    },
  };

  ctx.deleteAttribute(component, /testing/);

  assertEquals(Object.keys(component.attributes).length, 2);
  assertEquals(component.attributes["/domain/testing/config"], undefined);
  assertEquals(component.attributes["/domain/production/config"], "value2");
  assertEquals(component.attributes["/si/tags/test"], "value3");
});

Deno.test("deleteAttribute - deletes with predicate function (path only)", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "value" },
      "/si/tags/env": "production",
      "/si/tags/region": "us-west-2",
      "/secrets/api_key": "secret",
    },
  };

  ctx.deleteAttribute(component, (path) => path.startsWith("/si/tags/"));

  assertEquals(Object.keys(component.attributes).length, 2);
  assertEquals(component.attributes["/domain/config"], { key: "value" });
  assertEquals(component.attributes["/si/tags/env"], undefined);
  assertEquals(component.attributes["/si/tags/region"], undefined);
  assertEquals(component.attributes["/secrets/api_key"], "secret");
});

Deno.test("deleteAttribute - deletes with predicate function (path and value)", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/si/tags/status": "deprecated",
      "/si/tags/env": "production",
      "/si/tags/temp": "deprecated",
      "/domain/config": "deprecated",
    },
  };

  ctx.deleteAttribute(
    component,
    (path, value) => path.startsWith("/si/tags/") && value === "deprecated",
  );

  assertEquals(Object.keys(component.attributes).length, 2);
  assertEquals(component.attributes["/si/tags/status"], undefined);
  assertEquals(component.attributes["/si/tags/env"], "production");
  assertEquals(component.attributes["/si/tags/temp"], undefined);
  assertEquals(component.attributes["/domain/config"], "deprecated");
});

Deno.test("deleteAttribute - deletes with predicate function (all three arguments)", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "TestComponent",
    resourceId: "resource-1",
    attributes: {
      "/si/tags/owner": "TestComponent",
      "/si/tags/env": "production",
      "/domain/name": "TestComponent",
    },
  };

  ctx.deleteAttribute(
    component,
    (path, value, comp) => path.startsWith("/si/tags/") && value === comp.name,
  );

  assertEquals(Object.keys(component.attributes).length, 2);
  assertEquals(component.attributes["/si/tags/owner"], undefined);
  assertEquals(component.attributes["/si/tags/env"], "production");
  assertEquals(component.attributes["/domain/name"], "TestComponent");
});

Deno.test("deleteAttribute - handles empty attributes object", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {},
  };

  ctx.deleteAttribute(component, "/domain/config");
  ctx.deleteAttribute(component, /anything/);
  ctx.deleteAttribute(component, () => true);

  assertEquals(Object.keys(component.attributes).length, 0);
});

Deno.test("deleteAttribute - idempotent deletion", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "value" },
      "/si/tags/env": "production",
    },
  };

  ctx.deleteAttribute(component, "/domain/config");
  assertEquals(Object.keys(component.attributes).length, 1);

  // Delete again - should not error
  ctx.deleteAttribute(component, "/domain/config");
  assertEquals(Object.keys(component.attributes).length, 1);
  assertEquals(component.attributes["/si/tags/env"], "production");
});

Deno.test("deleteAttribute - deletes all attributes with predicate returning true", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "value" },
      "/si/tags/env": "production",
      "/secrets/api_key": "secret",
    },
  };

  ctx.deleteAttribute(component, () => true);

  assertEquals(Object.keys(component.attributes).length, 0);
});

Deno.test("setAttribute - sets exact string match", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "oldValue" },
      "/si/tags/env": "production",
    },
  };

  ctx.setAttribute(component, "/domain/config", { key: "newValue" });

  assertEquals(Object.keys(component.attributes).length, 2);
  assertEquals(component.attributes["/domain/config"], { key: "newValue" });
  assertEquals(component.attributes["/si/tags/env"], "production");
});

Deno.test("setAttribute - creates new attribute with string path", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "value" },
    },
  };

  ctx.setAttribute(component, "/domain/newConfig", { new: "data" });

  assertEquals(Object.keys(component.attributes).length, 2);
  assertEquals(component.attributes["/domain/config"], { key: "value" });
  assertEquals(component.attributes["/domain/newConfig"], { new: "data" });
});

Deno.test("setAttribute - sets multiple attributes with regex", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/temp1": "oldValue1",
      "/domain/temp2": "oldValue2",
      "/domain/config": { key: "value" },
      "/si/tags/env": "production",
    },
  };

  ctx.setAttribute(component, /^\/domain\/temp/, "updated");

  assertEquals(Object.keys(component.attributes).length, 4);
  assertEquals(component.attributes["/domain/temp1"], "updated");
  assertEquals(component.attributes["/domain/temp2"], "updated");
  assertEquals(component.attributes["/domain/config"], { key: "value" });
  assertEquals(component.attributes["/si/tags/env"], "production");
});

Deno.test("setAttribute - sets with regex partial match", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/testing/config": "value1",
      "/domain/production/config": "value2",
      "/si/tags/test": "value3",
    },
  };

  ctx.setAttribute(component, /testing/, "updated");

  assertEquals(Object.keys(component.attributes).length, 3);
  assertEquals(component.attributes["/domain/testing/config"], "updated");
  assertEquals(component.attributes["/domain/production/config"], "value2");
  assertEquals(component.attributes["/si/tags/test"], "value3");
});

Deno.test("setAttribute - does nothing with regex when no match found", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "value" },
      "/si/tags/env": "production",
    },
  };

  ctx.setAttribute(component, /nonexistent/, "newValue");

  assertEquals(Object.keys(component.attributes).length, 2);
  assertEquals(component.attributes["/domain/config"], { key: "value" });
  assertEquals(component.attributes["/si/tags/env"], "production");
});

Deno.test("setAttribute - sets with predicate function (path only)", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "value" },
      "/si/tags/env": "production",
      "/si/tags/region": "us-west-2",
      "/secrets/api_key": "secret",
    },
  };

  ctx.setAttribute(
    component,
    (path) => path.startsWith("/si/tags/"),
    "updated",
  );

  assertEquals(Object.keys(component.attributes).length, 4);
  assertEquals(component.attributes["/domain/config"], { key: "value" });
  assertEquals(component.attributes["/si/tags/env"], "updated");
  assertEquals(component.attributes["/si/tags/region"], "updated");
  assertEquals(component.attributes["/secrets/api_key"], "secret");
});

Deno.test("setAttribute - sets with predicate function (path and value)", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/si/tags/status": "deprecated",
      "/si/tags/env": "production",
      "/si/tags/temp": "deprecated",
      "/domain/config": "deprecated",
    },
  };

  ctx.setAttribute(
    component,
    (path, value) => path.startsWith("/si/tags/") && value === "deprecated",
    "updated",
  );

  assertEquals(Object.keys(component.attributes).length, 4);
  assertEquals(component.attributes["/si/tags/status"], "updated");
  assertEquals(component.attributes["/si/tags/env"], "production");
  assertEquals(component.attributes["/si/tags/temp"], "updated");
  assertEquals(component.attributes["/domain/config"], "deprecated");
});

Deno.test("setAttribute - sets with predicate function (all three arguments)", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "TestComponent",
    resourceId: "resource-1",
    attributes: {
      "/si/tags/owner": "TestComponent",
      "/si/tags/env": "production",
      "/domain/name": "TestComponent",
    },
  };

  ctx.setAttribute(
    component,
    (path, value, comp) => path.startsWith("/si/tags/") && value === comp.name,
    "UpdatedOwner",
  );

  assertEquals(Object.keys(component.attributes).length, 3);
  assertEquals(component.attributes["/si/tags/owner"], "UpdatedOwner");
  assertEquals(component.attributes["/si/tags/env"], "production");
  assertEquals(component.attributes["/domain/name"], "TestComponent");
});

Deno.test("setAttribute - handles empty attributes object with string matcher", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {},
  };

  ctx.setAttribute(component, "/domain/config", { key: "value" });

  assertEquals(Object.keys(component.attributes).length, 1);
  assertEquals(component.attributes["/domain/config"], { key: "value" });
});

Deno.test("setAttribute - does nothing with regex/predicate on empty attributes", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {},
  };

  ctx.setAttribute(component, /anything/, "value");
  ctx.setAttribute(component, () => true, "value");

  assertEquals(Object.keys(component.attributes).length, 0);
});

Deno.test("setAttribute - idempotent setting", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "value1" },
      "/si/tags/env": "production",
    },
  };

  ctx.setAttribute(component, "/domain/config", { key: "value2" });
  assertEquals(component.attributes["/domain/config"], { key: "value2" });

  // Set again with same value - should overwrite
  ctx.setAttribute(component, "/domain/config", { key: "value2" });
  assertEquals(component.attributes["/domain/config"], { key: "value2" });
  assertEquals(Object.keys(component.attributes).length, 2);
});

Deno.test("setAttribute - sets all attributes with predicate returning true", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": { key: "value" },
      "/si/tags/env": "production",
      "/secrets/api_key": "secret",
    },
  };

  ctx.setAttribute(component, () => true, "updated");

  assertEquals(Object.keys(component.attributes).length, 3);
  assertEquals(component.attributes["/domain/config"], "updated");
  assertEquals(component.attributes["/si/tags/env"], "updated");
  assertEquals(component.attributes["/secrets/api_key"], "updated");
});

Deno.test("setAttribute - supports different value types", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {},
  };

  // String value
  ctx.setAttribute(component, "/domain/string", "text");
  assertEquals(component.attributes["/domain/string"], "text");

  // Number value
  ctx.setAttribute(component, "/domain/number", 42);
  assertEquals(component.attributes["/domain/number"], 42);

  // Boolean value
  ctx.setAttribute(component, "/domain/boolean", true);
  assertEquals(component.attributes["/domain/boolean"], true);

  // Array value
  ctx.setAttribute(component, "/domain/array", [1, 2, 3]);
  assertEquals(component.attributes["/domain/array"], [1, 2, 3]);

  // Object value
  ctx.setAttribute(component, "/domain/object", { nested: { key: "value" } });
  assertEquals(component.attributes["/domain/object"], {
    nested: { key: "value" },
  });

  // Null value
  ctx.setAttribute(component, "/domain/null", null);
  assertEquals(component.attributes["/domain/null"], null);

  assertEquals(Object.keys(component.attributes).length, 6);
});

// ===== setSiblingAttribute Tests =====

Deno.test("setSiblingAttribute - sets sibling with string matcher and exact value", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/Tags/0/Key": "Name",
      "/domain/Tags/0/Value": "old-value",
    },
  };

  ctx.setSiblingAttribute(
    component,
    "/domain/Tags/0/Key",
    "Name",
    "Value",
    "poop-canoe",
  );

  assertEquals(component.attributes["/domain/Tags/0/Value"], "poop-canoe");
  assertEquals(component.attributes["/domain/Tags/0/Key"], "Name"); // Original unchanged
});

Deno.test("setSiblingAttribute - sets sibling with string matcher and value predicate", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/Tags/0/Key": "Environment",
      "/domain/Tags/0/Value": "old-value",
    },
  };

  ctx.setSiblingAttribute(
    component,
    "/domain/Tags/0/Key",
    // deno-lint-ignore no-explicit-any
    (v: any) => v === "Environment",
    "Value",
    "production",
  );

  assertEquals(component.attributes["/domain/Tags/0/Value"], "production");
});

Deno.test("setSiblingAttribute - sets multiple siblings with regex matcher", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/Tags/0/Key": "Name",
      "/domain/Tags/0/Value": "old-value-0",
      "/domain/Tags/1/Key": "Name",
      "/domain/Tags/1/Value": "old-value-1",
      "/domain/Tags/2/Key": "Environment",
      "/domain/Tags/2/Value": "old-value-2",
    },
  };

  ctx.setSiblingAttribute(
    component,
    /\/domain\/Tags\/\d+\/Key/,
    "Name",
    "Value",
    "updated-name",
  );

  assertEquals(component.attributes["/domain/Tags/0/Value"], "updated-name");
  assertEquals(component.attributes["/domain/Tags/1/Value"], "updated-name");
  assertEquals(component.attributes["/domain/Tags/2/Value"], "old-value-2"); // Not Name, unchanged
});

Deno.test("setSiblingAttribute - sets sibling with predicate key matcher", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/SecurityGroupIngress/0/IpProtocol": "tcp",
      "/domain/SecurityGroupIngress/0/FromPort": 80,
      "/domain/SecurityGroupIngress/1/IpProtocol": "udp",
      "/domain/SecurityGroupIngress/1/FromPort": 53,
    },
  };

  ctx.setSiblingAttribute(
    component,
    (path) =>
      path.includes("SecurityGroupIngress") && path.endsWith("/IpProtocol"),
    "tcp",
    "FromPort",
    443,
  );

  assertEquals(
    component.attributes["/domain/SecurityGroupIngress/0/FromPort"],
    443,
  );
  assertEquals(
    component.attributes["/domain/SecurityGroupIngress/1/FromPort"],
    53,
  ); // UDP unchanged
});

Deno.test("setSiblingAttribute - does nothing when key matches but value doesn't", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/Tags/0/Key": "Name",
      "/domain/Tags/0/Value": "old-value",
    },
  };

  ctx.setSiblingAttribute(
    component,
    "/domain/Tags/0/Key",
    "WrongValue", // This doesn't match "Name"
    "Value",
    "new-value",
  );

  assertEquals(component.attributes["/domain/Tags/0/Value"], "old-value"); // Unchanged
});

Deno.test("setSiblingAttribute - does nothing when value matches but key doesn't", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/Tags/0/Key": "Name",
      "/domain/Tags/0/Value": "old-value",
    },
  };

  ctx.setSiblingAttribute(
    component,
    "/domain/Tags/99/Key", // Wrong path
    "Name",
    "Value",
    "new-value",
  );

  assertEquals(component.attributes["/domain/Tags/0/Value"], "old-value"); // Unchanged
});

Deno.test("setSiblingAttribute - uses deep equality for object values", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/Config/0/Settings": { enabled: true, count: 5 },
      "/domain/Config/0/Name": "old-name",
    },
  };

  ctx.setSiblingAttribute(
    component,
    "/domain/Config/0/Settings",
    { enabled: true, count: 5 }, // Deep equality check
    "Name",
    "matched-config",
  );

  assertEquals(component.attributes["/domain/Config/0/Name"], "matched-config");
});

Deno.test("setSiblingAttribute - handles non-array element paths with warning", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/config": "test-value", // Not an array element
    },
  };

  // This should log a warning but not throw
  ctx.setSiblingAttribute(
    component,
    "/domain/config",
    "test-value",
    "sibling",
    "new-value",
  );

  // Sibling should not be created since path is invalid
  assertEquals(component.attributes["/domain/config/sibling"], undefined);
  assertEquals(component.attributes["/domain/config"], "test-value"); // Original unchanged
});

Deno.test("setSiblingAttribute - creates sibling even if it doesn't exist", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/Tags/0/Key": "Name",
      // No /domain/Tags/0/Value exists yet
    },
  };

  ctx.setSiblingAttribute(
    component,
    "/domain/Tags/0/Key",
    "Name",
    "Value",
    "new-value",
  );

  assertEquals(component.attributes["/domain/Tags/0/Value"], "new-value");
  assertEquals(component.attributes["/domain/Tags/0/Key"], "Name");
});

Deno.test("setSiblingAttribute - sets different sibling property", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/Tags/0/Key": "Name",
      "/domain/Tags/0/Value": "original",
      "/domain/Tags/0/Description": "old-description",
    },
  };

  // Set Description instead of Value
  ctx.setSiblingAttribute(
    component,
    "/domain/Tags/0/Key",
    "Name",
    "Description",
    "new-description",
  );

  assertEquals(
    component.attributes["/domain/Tags/0/Description"],
    "new-description",
  );
  assertEquals(component.attributes["/domain/Tags/0/Value"], "original"); // Value unchanged
  assertEquals(component.attributes["/domain/Tags/0/Key"], "Name"); // Key unchanged
});

Deno.test("setSiblingAttribute - handles nested path structures", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/Network/Subnets/0/CIDR": "10.0.0.0/24",
      "/domain/Network/Subnets/0/Name": "old-name",
    },
  };

  ctx.setSiblingAttribute(
    component,
    "/domain/Network/Subnets/0/CIDR",
    "10.0.0.0/24",
    "Name",
    "public-subnet",
  );

  assertEquals(
    component.attributes["/domain/Network/Subnets/0/Name"],
    "public-subnet",
  );
});

Deno.test("setSiblingAttribute - value predicate with complex logic", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {
      "/domain/Tags/0/Key": "env-production",
      "/domain/Tags/0/Value": "old",
      "/domain/Tags/1/Key": "env-staging",
      "/domain/Tags/1/Value": "old",
      "/domain/Tags/2/Key": "other",
      "/domain/Tags/2/Value": "old",
    },
  };

  ctx.setSiblingAttribute(
    component,
    /\/domain\/Tags\/\d+\/Key/,
    // deno-lint-ignore no-explicit-any
    (v: any) => typeof v === "string" && v.startsWith("env-"),
    "Value",
    "updated-env",
  );

  assertEquals(component.attributes["/domain/Tags/0/Value"], "updated-env");
  assertEquals(component.attributes["/domain/Tags/1/Value"], "updated-env");
  assertEquals(component.attributes["/domain/Tags/2/Value"], "old"); // Doesn't start with env-
});

Deno.test("setSiblingAttribute - handles empty attributes object", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const component: TemplateComponent = {
    id: "comp-1",
    schemaId: "schema-1",
    name: "Test Component",
    resourceId: "resource-1",
    attributes: {},
  };

  // Should not throw, just log warning
  ctx.setSiblingAttribute(
    component,
    "/domain/Tags/0/Key",
    "Name",
    "Value",
    "new-value",
  );

  assertEquals(Object.keys(component.attributes).length, 0);
});

// ===== copyComponent Tests =====

Deno.test("copyComponent - creates copy with new name and ID", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const original: TemplateComponent = {
    id: "original-id-123",
    schemaId: "schema-1",
    name: "original-component",
    resourceId: "res-1",
    attributes: {
      "/si/name": "original-component",
      "/domain/config": { key: "value" },
      "/secrets/api_key": "secret",
    },
  };

  const copy = ctx.copyComponent(original, "copied-component");

  // Verify new ID
  assertEquals(copy.id.length, 26); // ULID is 26 characters
  assertEquals(copy.id === original.id, false); // IDs should be different

  // Verify name was updated
  assertEquals(copy.name, "copied-component");
  assertEquals(copy.attributes["/si/name"], "copied-component");

  // Verify other fields are copied
  assertEquals(copy.schemaId, "schema-1");
  assertEquals(copy.resourceId, "res-1");
});

Deno.test("copyComponent - deep clones attributes", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const original: TemplateComponent = {
    id: "original-id",
    schemaId: "schema-1",
    name: "original",
    resourceId: "res-1",
    attributes: {
      "/domain/config": { nested: { key: "value" } },
      "/domain/array": [1, 2, 3],
    },
  };

  const copy = ctx.copyComponent(original, "copy");

  // Mutate the copy's nested object
  (copy.attributes["/domain/config"] as { nested: { key: string } }).nested
    .key = "modified";
  (copy.attributes["/domain/array"] as number[]).push(4);

  // Original should be unchanged
  assertEquals(
    (original.attributes["/domain/config"] as { nested: { key: string } })
      .nested.key,
    "value",
  );
  assertEquals((original.attributes["/domain/array"] as number[]).length, 3);
});

Deno.test("copyComponent - preserves subscriptions", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const original: TemplateComponent = {
    id: "original-id",
    schemaId: "schema-1",
    name: "original",
    resourceId: "res-1",
    attributes: {
      "/domain/config": {
        $source: {
          component: "other-component-id",
          path: "/domain/output",
          func: "identity",
        },
      },
    },
  };

  const copy = ctx.copyComponent(original, "copy");

  // Verify subscription was preserved
  assertEquals(copy.attributes["/domain/config"], {
    $source: {
      component: "other-component-id",
      path: "/domain/output",
      func: "identity",
    },
  });
});

Deno.test("copyComponent - throws on empty name", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const original: TemplateComponent = {
    id: "original-id",
    schemaId: "schema-1",
    name: "original",
    resourceId: "res-1",
    attributes: {},
  };

  let errorThrown = false;
  try {
    ctx.copyComponent(original, "");
  } catch (error) {
    errorThrown = true;
    assertEquals((error as Error).message, "Component name cannot be empty");
  }

  assertEquals(errorThrown, true);
});

Deno.test("copyComponent - throws on whitespace-only name", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const original: TemplateComponent = {
    id: "original-id",
    schemaId: "schema-1",
    name: "original",
    resourceId: "res-1",
    attributes: {},
  };

  let errorThrown = false;
  try {
    ctx.copyComponent(original, "   ");
  } catch (error) {
    errorThrown = true;
    assertEquals((error as Error).message, "Component name cannot be empty");
  }

  assertEquals(errorThrown, true);
});

Deno.test("copyComponent - handles component without /si/name attribute", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const original: TemplateComponent = {
    id: "original-id",
    schemaId: "schema-1",
    name: "original",
    resourceId: "res-1",
    attributes: {
      "/domain/config": { key: "value" },
      // No /si/name attribute
    },
  };

  const copy = ctx.copyComponent(original, "copy");

  // Verify name field is updated
  assertEquals(copy.name, "copy");

  // Verify /si/name is not added (since it didn't exist)
  assertEquals(copy.attributes["/si/name"], undefined);

  // Verify other attributes are copied
  assertEquals(copy.attributes["/domain/config"], { key: "value" });
});

Deno.test("copyComponent - generates unique IDs for multiple copies", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const original: TemplateComponent = {
    id: "original-id",
    schemaId: "schema-1",
    name: "original",
    resourceId: "res-1",
    attributes: {},
  };

  const copy1 = ctx.copyComponent(original, "copy-1");
  const copy2 = ctx.copyComponent(original, "copy-2");
  const copy3 = ctx.copyComponent(original, "copy-3");

  // All IDs should be different
  assertEquals(copy1.id === copy2.id, false);
  assertEquals(copy1.id === copy3.id, false);
  assertEquals(copy2.id === copy3.id, false);
  assertEquals(copy1.id === original.id, false);
});

Deno.test("copyComponent - works in transform function with loop", () => {
  const ctx = new TemplateContext("/path/to/template.ts", { key: "test" });

  const workingSet: TemplateComponent[] = [
    {
      id: "template-id",
      schemaId: "schema-1",
      name: "server-template",
      resourceId: "res-1",
      attributes: {
        "/si/name": "server-template",
        "/domain/instance_type": "t2.micro",
      },
    },
  ];

  // Simulate a transform function that creates 3 copies
  const template = workingSet[0];
  for (let i = 1; i <= 3; i++) {
    const copy = ctx.copyComponent(template, `server-${i}`);
    workingSet.push(copy);
  }

  // Verify we have 4 components total (original + 3 copies)
  assertEquals(workingSet.length, 4);

  // Verify original is unchanged
  assertEquals(workingSet[0].name, "server-template");

  // Verify copies have correct names
  assertEquals(workingSet[1].name, "server-1");
  assertEquals(workingSet[2].name, "server-2");
  assertEquals(workingSet[3].name, "server-3");

  // Verify all have unique IDs
  const ids = workingSet.map((c) => c.id);
  const uniqueIds = new Set(ids);
  assertEquals(uniqueIds.size, 4);

  // Verify attributes are copied
  for (let i = 1; i <= 3; i++) {
    assertEquals(workingSet[i].attributes["/domain/instance_type"], "t2.micro");
  }
});
