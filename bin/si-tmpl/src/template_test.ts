import { assertEquals, assertExists } from "@std/assert";
import { TemplateContext } from "./template.ts";
import { Context } from "./context.ts";

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
    { path: "/complex/path/my-awesome-template.ts", expected: "my-awesome-template" },
    { path: "https://complex.com/path/my-awesome-template.ts", expected: "my-awesome-template" },
  ];

  for (const testCase of testCases) {
    const ctx = new TemplateContext(testCase.path, { key: "key" });
    assertEquals(ctx.name(), testCase.expected, `Failed for path: ${testCase.path}`);
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
