import { assertEquals } from "@std/assert";
import { Context } from "../context.ts";
import { TemplateComponent, TemplateContext } from "./context.ts";
import { updateNamesOfWorkingSet } from "./names.ts";

// Initialize Context once - this runs at module load time
await Context.init({ verbose: 0, noColor: true });

Deno.test("updateNamesOfWorkingSet() - applies pattern to component name", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "prod-server-01",
    resourceId: "res-1",
    attributes: {},
  }];
  tctx.workingSet(workingSet);

  // Set name pattern
  tctx.namePattern([{
    pattern: /^prod-/,
    replacement: "dev-",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "dev-server-01");
});

Deno.test("updateNamesOfWorkingSet() - applies pattern to /si/name attribute", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set with /si/name attribute
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "prod-server-01",
    resourceId: "res-1",
    attributes: {
      "/si/name": "prod-server-01",
    },
  }];
  tctx.workingSet(workingSet);

  // Set name pattern
  tctx.namePattern([{
    pattern: /^prod-/,
    replacement: "dev-",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "dev-server-01");
  assertEquals(result![0].attributes["/si/name"], "dev-server-01");
});

Deno.test("updateNamesOfWorkingSet() - handles component without /si/name attribute", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set without /si/name
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "prod-server-01",
    resourceId: "res-1",
    attributes: {
      "/si/other": "value",
    },
  }];
  tctx.workingSet(workingSet);

  // Set name pattern
  tctx.namePattern([{
    pattern: /^prod-/,
    replacement: "dev-",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "dev-server-01");
  assertEquals(result![0].attributes["/si/name"], undefined);
  assertEquals(result![0].attributes["/si/other"], "value");
});

Deno.test("updateNamesOfWorkingSet() - no-op when namePattern is undefined", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "prod-server-01",
    resourceId: "res-1",
    attributes: {
      "/si/name": "prod-server-01",
    },
  }];
  tctx.workingSet(workingSet);

  // Don't set namePattern - should be no-op
  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "prod-server-01");
  assertEquals(result![0].attributes["/si/name"], "prod-server-01");
});

Deno.test("updateNamesOfWorkingSet() - no-op when namePattern is empty array", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "prod-server-01",
    resourceId: "res-1",
    attributes: {
      "/si/name": "prod-server-01",
    },
  }];
  tctx.workingSet(workingSet);

  // Set empty array - should be no-op
  tctx.namePattern([]);
  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "prod-server-01");
  assertEquals(result![0].attributes["/si/name"], "prod-server-01");
});

Deno.test("updateNamesOfWorkingSet() - no-op when workingSet is undefined", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set name pattern but don't set working set
  tctx.namePattern([{
    pattern: /^prod-/,
    replacement: "dev-",
  }]);

  // Should not throw - just no-op
  updateNamesOfWorkingSet(tctx);

  assertEquals(tctx.workingSet(), undefined);
});

Deno.test("updateNamesOfWorkingSet() - handles empty workingSet", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up empty working set
  tctx.workingSet([]);

  // Set name pattern
  tctx.namePattern([{
    pattern: /^prod-/,
    replacement: "dev-",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result, []);
});

Deno.test("updateNamesOfWorkingSet() - applies pattern to multiple components", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set with multiple components
  const workingSet: TemplateComponent[] = [
    {
      id: "comp-1",
      schemaId: "schema-1",
      name: "prod-server-01",
      resourceId: "res-1",
      attributes: { "/si/name": "prod-server-01" },
    },
    {
      id: "comp-2",
      schemaId: "schema-1",
      name: "prod-server-02",
      resourceId: "res-2",
      attributes: { "/si/name": "prod-server-02" },
    },
    {
      id: "comp-3",
      schemaId: "schema-1",
      name: "prod-database-01",
      resourceId: "res-3",
      attributes: { "/si/name": "prod-database-01" },
    },
  ];
  tctx.workingSet(workingSet);

  // Set name pattern
  tctx.namePattern([{
    pattern: /^prod-/,
    replacement: "dev-",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "dev-server-01");
  assertEquals(result![0].attributes["/si/name"], "dev-server-01");
  assertEquals(result![1].name, "dev-server-02");
  assertEquals(result![1].attributes["/si/name"], "dev-server-02");
  assertEquals(result![2].name, "dev-database-01");
  assertEquals(result![2].attributes["/si/name"], "dev-database-01");
});

Deno.test("updateNamesOfWorkingSet() - applies pattern with capture groups", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "server-prod-01",
    resourceId: "res-1",
    attributes: {
      "/si/name": "server-prod-01",
    },
  }];
  tctx.workingSet(workingSet);

  // Set name pattern with capture groups
  tctx.namePattern([{
    pattern: /^(.*)-prod-(.*)$/,
    replacement: "$1-dev-$2",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "server-dev-01");
  assertEquals(result![0].attributes["/si/name"], "server-dev-01");
});

// ========== updateNamesOfWorkingSet() with EJS Tests ==========

Deno.test("updateNamesOfWorkingSet() - applies EJS template with inputs data", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up input data
  tctx.inputData({ environment: "prod" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "demo-server-01",
    resourceId: "resource-1",
    attributes: {
      "/si/name": "demo-server-01",
    },
  }];
  tctx.workingSet(workingSet);

  // Set name pattern with EJS template
  tctx.namePattern([{
    pattern: /demo-(.+)/g,
    replacement: "<%= inputs.environment %>-$1",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "prod-server-01");
  assertEquals(result![0].attributes["/si/name"], "prod-server-01");
});

Deno.test("updateNamesOfWorkingSet() - applies EJS template with context access", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set custom name
  tctx.name("my-template");

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "demo-server",
    resourceId: "resource-1",
    attributes: {
      "/si/name": "demo-server",
    },
  }];
  tctx.workingSet(workingSet);

  // Set name pattern with EJS template accessing context
  tctx.namePattern([{
    pattern: /demo-(.+)/g,
    replacement: "<%= c.name() %>-$1",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "my-template-server");
  assertEquals(result![0].attributes["/si/name"], "my-template-server");
});

Deno.test("updateNamesOfWorkingSet() - combines EJS and regex capture groups", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up input data
  tctx.inputData({ env: "staging", region: "us-west" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "old-database-01",
    resourceId: "resource-1",
    attributes: {
      "/si/name": "old-database-01",
    },
  }];
  tctx.workingSet(workingSet);

  // Set name pattern combining EJS and capture groups
  tctx.namePattern([{
    pattern: /old-(.+)-(.+)/g,
    replacement: "<%= inputs.env %>-<%= inputs.region %>-$1-$2",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "staging-us-west-database-01");
  assertEquals(
    result![0].attributes["/si/name"],
    "staging-us-west-database-01",
  );
});

Deno.test("updateNamesOfWorkingSet() - handles missing input data gracefully", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Don't set input data - inputs will be undefined

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "demo-server",
    resourceId: "resource-1",
    attributes: {},
  }];
  tctx.workingSet(workingSet);

  // Set name pattern that references undefined input
  tctx.namePattern([{
    pattern: /demo-(.+)/g,
    replacement: "<%= inputs?.environment || 'default' %>-$1",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "default-server");
});

Deno.test("updateNamesOfWorkingSet() - throws error on EJS syntax error", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "demo-server",
    resourceId: "resource-1",
    attributes: {},
  }];
  tctx.workingSet(workingSet);

  // Set name pattern with invalid EJS syntax
  tctx.namePattern([{
    pattern: /demo-(.+)/g,
    replacement: "<%= unclosed tag",
  }]);

  let errorThrown = false;
  try {
    updateNamesOfWorkingSet(tctx);
  } catch (error) {
    errorThrown = true;
    const errorMsg = error instanceof Error ? error.message : String(error);
    assertEquals(errorMsg.includes("EJS template evaluation failed"), true);
  }

  assertEquals(errorThrown, true, "Expected EJS syntax error to be thrown");
});

Deno.test("updateNamesOfWorkingSet() - works without EJS (plain replacement string)", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "prod-server-01",
    resourceId: "resource-1",
    attributes: {
      "/si/name": "prod-server-01",
    },
  }];
  tctx.workingSet(workingSet);

  // Set name pattern with plain replacement (no EJS)
  tctx.namePattern([{
    pattern: /prod-/g,
    replacement: "dev-",
  }]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "dev-server-01");
  assertEquals(result![0].attributes["/si/name"], "dev-server-01");
});

// ========== Sequential Pattern Tests ==========

Deno.test("updateNamesOfWorkingSet() - applies two patterns sequentially", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "old-prod-server-01",
    resourceId: "resource-1",
    attributes: {
      "/si/name": "old-prod-server-01",
    },
  }];
  tctx.workingSet(workingSet);

  // First pattern removes "old-", second pattern changes "prod-" to "dev-"
  tctx.namePattern([
    {
      pattern: /^old-/,
      replacement: "",
    },
    {
      pattern: /^prod-/,
      replacement: "dev-",
    },
  ]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "dev-server-01");
  assertEquals(result![0].attributes["/si/name"], "dev-server-01");
});

Deno.test("updateNamesOfWorkingSet() - applies three patterns sequentially", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "legacy-prod-server-us",
    resourceId: "resource-1",
    attributes: {
      "/si/name": "legacy-prod-server-us",
    },
  }];
  tctx.workingSet(workingSet);

  // Apply three patterns in sequence
  tctx.namePattern([
    {
      pattern: /^legacy-/,
      replacement: "",
    },
    {
      pattern: /^prod-/,
      replacement: "staging-",
    },
    {
      pattern: /-us$/,
      replacement: "-west",
    },
  ]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "staging-server-west");
  assertEquals(result![0].attributes["/si/name"], "staging-server-west");
});

Deno.test("updateNamesOfWorkingSet() - sequential patterns with EJS templates", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up input data
  tctx.inputData({ env: "prod", region: "east" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "demo-server-temp",
    resourceId: "resource-1",
    attributes: {
      "/si/name": "demo-server-temp",
    },
  }];
  tctx.workingSet(workingSet);

  // First pattern uses EJS to replace demo with environment, second adds region
  tctx.namePattern([
    {
      pattern: /^demo-/,
      replacement: "<%= inputs.env %>-",
    },
    {
      pattern: /-temp$/,
      replacement: "-<%= inputs.region %>",
    },
  ]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "prod-server-east");
  assertEquals(result![0].attributes["/si/name"], "prod-server-east");
});

Deno.test("updateNamesOfWorkingSet() - sequential patterns with capture groups", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set
  const workingSet: TemplateComponent[] = [{
    id: "comp-1",
    schemaId: "schema-1",
    name: "web-server-prod-01",
    resourceId: "resource-1",
    attributes: {
      "/si/name": "web-server-prod-01",
    },
  }];
  tctx.workingSet(workingSet);

  // First pattern swaps first two parts, second changes prod to dev
  tctx.namePattern([
    {
      pattern: /^([^-]+)-([^-]+)-(.+)$/,
      replacement: "$2-$1-$3",
    },
    {
      pattern: /prod/g,
      replacement: "dev",
    },
  ]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "server-web-dev-01");
  assertEquals(result![0].attributes["/si/name"], "server-web-dev-01");
});

Deno.test("updateNamesOfWorkingSet() - sequential patterns apply to multiple components", () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test-key" });

  // Set up working set with multiple components
  const workingSet: TemplateComponent[] = [
    {
      id: "comp-1",
      schemaId: "schema-1",
      name: "old-prod-web-01",
      resourceId: "res-1",
      attributes: { "/si/name": "old-prod-web-01" },
    },
    {
      id: "comp-2",
      schemaId: "schema-1",
      name: "old-prod-db-01",
      resourceId: "res-2",
      attributes: { "/si/name": "old-prod-db-01" },
    },
    {
      id: "comp-3",
      schemaId: "schema-1",
      name: "old-prod-cache-01",
      resourceId: "res-3",
      attributes: { "/si/name": "old-prod-cache-01" },
    },
  ];
  tctx.workingSet(workingSet);

  // Apply two patterns sequentially to all components
  tctx.namePattern([
    {
      pattern: /^old-/,
      replacement: "new-",
    },
    {
      pattern: /prod/g,
      replacement: "staging",
    },
  ]);

  updateNamesOfWorkingSet(tctx);

  const result = tctx.workingSet();
  assertEquals(result![0].name, "new-staging-web-01");
  assertEquals(result![0].attributes["/si/name"], "new-staging-web-01");
  assertEquals(result![1].name, "new-staging-db-01");
  assertEquals(result![1].attributes["/si/name"], "new-staging-db-01");
  assertEquals(result![2].name, "new-staging-cache-01");
  assertEquals(result![2].attributes["/si/name"], "new-staging-cache-01");
});
