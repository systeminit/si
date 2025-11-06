import { assertEquals, assertRejects } from "@std/assert";
import { stringify as stringifyYaml } from "@std/yaml";
import { z } from "zod";
import { Context } from "../context.ts";
import { TemplateContext } from "./context.ts";
import {
  loadDataFromFile,
  loadInputData,
  schemaHasRequiredFields,
} from "./input.ts";

// Initialize Context once - this runs at module load time
await Context.init({ verbose: 0, noColor: true });

// ========== loadDataFromFile Tests ==========

Deno.test("loadDataFromFile - loads JSON file", async () => {
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  const data = { name: "test", value: 42 };
  await Deno.writeTextFile(tempFile, JSON.stringify(data));

  const result = await loadDataFromFile(tempFile);

  assertEquals(result, data);

  await Deno.remove(tempFile);
});

Deno.test("loadDataFromFile - loads YAML file (.yaml)", async () => {
  const tempFile = await Deno.makeTempFile({ suffix: ".yaml" });
  const data = { name: "test", value: 42 };
  await Deno.writeTextFile(tempFile, stringifyYaml(data));

  const result = await loadDataFromFile(tempFile);

  assertEquals(result, data);

  await Deno.remove(tempFile);
});

Deno.test("loadDataFromFile - loads YAML file (.yml)", async () => {
  const tempFile = await Deno.makeTempFile({ suffix: ".yml" });
  const data = { name: "test", value: 42 };
  await Deno.writeTextFile(tempFile, stringifyYaml(data));

  const result = await loadDataFromFile(tempFile);

  assertEquals(result, data);

  await Deno.remove(tempFile);
});

Deno.test("loadDataFromFile - throws error for unsupported format", async () => {
  const tempFile = await Deno.makeTempFile({ suffix: ".txt" });
  await Deno.writeTextFile(tempFile, "some text");

  await assertRejects(
    async () => await loadDataFromFile(tempFile),
    Error,
    "Unsupported file format: .txt",
  );

  await Deno.remove(tempFile);
});

Deno.test("loadDataFromFile - throws error for invalid JSON", async () => {
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  await Deno.writeTextFile(tempFile, "{ invalid json }");

  await assertRejects(
    async () => await loadDataFromFile(tempFile),
    Error,
  );

  await Deno.remove(tempFile);
});

// ========== loadInputData Tests ==========

Deno.test("loadInputData - loads and validates input data successfully", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test" });

  // Define input schema
  const schema = z.object({
    name: z.string(),
    count: z.number(),
  });
  tctx.inputs(schema);

  // Create temp input file
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  const inputData = { name: "test-input", count: 5 };
  await Deno.writeTextFile(tempFile, JSON.stringify(inputData));

  await loadInputData(tctx, tempFile);

  const result = tctx.inputData();
  assertEquals(result, inputData);

  await Deno.remove(tempFile);
});

Deno.test("loadInputData - loads YAML input successfully", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test" });

  // Define input schema
  const schema = z.object({
    name: z.string(),
    count: z.number(),
  });
  tctx.inputs(schema);

  // Create temp input file
  const tempFile = await Deno.makeTempFile({ suffix: ".yaml" });
  const inputData = { name: "test-input", count: 5 };
  await Deno.writeTextFile(tempFile, stringifyYaml(inputData));

  await loadInputData(tctx, tempFile);

  const result = tctx.inputData();
  assertEquals(result, inputData);

  await Deno.remove(tempFile);
});

Deno.test("loadInputData - throws error when validation fails", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test" });

  // Define input schema requiring specific fields
  const schema = z.object({
    name: z.string(),
    count: z.number(),
  });
  tctx.inputs(schema);

  // Create temp input file with invalid data (missing count)
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  const inputData = { name: "test-input" };
  await Deno.writeTextFile(tempFile, JSON.stringify(inputData));

  await assertRejects(
    async () => await loadInputData(tctx, tempFile),
    Error,
    "Input validation failed",
  );

  await Deno.remove(tempFile);
});

Deno.test("loadInputData - provides detailed validation error messages", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test" });

  // Define input schema with multiple required fields
  const schema = z.object({
    name: z.string(),
    count: z.number(),
    enabled: z.boolean(),
  });
  tctx.inputs(schema);

  // Create temp input file with multiple validation errors
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  const inputData = { name: "test" }; // missing count and enabled
  await Deno.writeTextFile(tempFile, JSON.stringify(inputData));

  try {
    await loadInputData(tctx, tempFile);
    throw new Error("Expected validation to fail");
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    assertEquals(
      errorMsg.includes("count"),
      true,
      "Error should mention 'count'",
    );
    assertEquals(
      errorMsg.includes("enabled"),
      true,
      "Error should mention 'enabled'",
    );
  }

  await Deno.remove(tempFile);
});

Deno.test("loadInputData - accepts data without schema (no validation)", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test" });

  // No schema defined - should accept any data

  // Create temp input file
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  const inputData = { arbitrary: "data", without: "schema" };
  await Deno.writeTextFile(tempFile, JSON.stringify(inputData));

  await loadInputData(tctx, tempFile);

  const result = tctx.inputData();
  assertEquals(result, inputData);

  await Deno.remove(tempFile);
});

Deno.test("loadInputData - validates complex nested schema", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test" });

  // Define complex nested schema
  const schema = z.object({
    name: z.string(),
    config: z.object({
      region: z.string(),
      replicas: z.number().min(1),
    }),
    tags: z.array(z.string()),
  });
  tctx.inputs(schema);

  // Create temp input file with valid nested data
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  const inputData = {
    name: "my-service",
    config: {
      region: "us-west-2",
      replicas: 3,
    },
    tags: ["prod", "critical"],
  };
  await Deno.writeTextFile(tempFile, JSON.stringify(inputData));

  await loadInputData(tctx, tempFile);

  const result = tctx.inputData();
  assertEquals(result, inputData);

  await Deno.remove(tempFile);
});

Deno.test("loadInputData - throws error for non-object data", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test" });

  // Create temp input file with primitive (not object)
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  await Deno.writeTextFile(tempFile, JSON.stringify("just a string"));

  await assertRejects(
    async () => await loadInputData(tctx, tempFile),
    Error,
    "Input file must contain an object",
  );

  await Deno.remove(tempFile);
});

Deno.test("loadInputData - validates with optional fields", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test" });

  // Define schema with optional fields
  const schema = z.object({
    name: z.string(),
    description: z.string().optional(),
    count: z.number().default(1),
  });
  tctx.inputs(schema);

  // Create temp input file with only required field
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  const inputData = { name: "test-input" };
  await Deno.writeTextFile(tempFile, JSON.stringify(inputData));

  await loadInputData(tctx, tempFile);

  const result = tctx.inputData() as { name: string; count: number };
  assertEquals(result!.name, "test-input");
  assertEquals(result!.count, 1); // default value

  await Deno.remove(tempFile);
});

Deno.test("loadInputData - validates with array fields", async () => {
  const tctx = new TemplateContext("/test/template.ts", { key: "test" });

  // Define schema with array validation
  const schema = z.object({
    name: z.string(),
    tags: z.array(z.string()).min(1),
  });
  tctx.inputs(schema);

  // Create temp input file
  const tempFile = await Deno.makeTempFile({ suffix: ".json" });
  const inputData = { name: "test-input", tags: ["tag1", "tag2"] };
  await Deno.writeTextFile(tempFile, JSON.stringify(inputData));

  await loadInputData(tctx, tempFile);

  const result = tctx.inputData() as { name: string; tags: string[] };
  assertEquals(result!.name, "test-input");
  assertEquals(result!.tags, ["tag1", "tag2"]);

  await Deno.remove(tempFile);
});

// ========== schemaHasRequiredFields Tests ==========

Deno.test("schemaHasRequiredFields - returns true for schema with required fields", () => {
  const schema = z.object({
    name: z.string(),
    count: z.number(),
  });

  assertEquals(schemaHasRequiredFields(schema), true);
});

Deno.test("schemaHasRequiredFields - returns false for schema with all optional fields", () => {
  const schema = z.object({
    name: z.string().optional(),
    count: z.number().optional(),
  });

  assertEquals(schemaHasRequiredFields(schema), false);
});

Deno.test("schemaHasRequiredFields - returns false for schema with all default fields", () => {
  const schema = z.object({
    name: z.string().default("default-name"),
    count: z.number().default(0),
  });

  assertEquals(schemaHasRequiredFields(schema), false);
});

Deno.test("schemaHasRequiredFields - returns true for schema with mixed required and optional fields", () => {
  const schema = z.object({
    name: z.string(), // required
    description: z.string().optional(), // optional
  });

  assertEquals(schemaHasRequiredFields(schema), true);
});

Deno.test("schemaHasRequiredFields - returns false for empty schema", () => {
  const schema = z.object({});

  assertEquals(schemaHasRequiredFields(schema), false);
});

Deno.test("schemaHasRequiredFields - returns true for nested schema with required fields", () => {
  const schema = z.object({
    config: z.object({
      region: z.string(), // required nested field
    }),
  });

  assertEquals(schemaHasRequiredFields(schema), true);
});
