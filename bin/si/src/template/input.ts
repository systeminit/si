import { extname } from "@std/path";
import { parse as parseYaml } from "@std/yaml";
import type { z } from "zod";
import { Context } from "../context.ts";
import type { TemplateContext } from "./context.ts";

/**
 * Load and parse data from a JSON or YAML file.
 *
 * @param filePath - Path to the file to load
 * @returns The parsed data
 */
export async function loadDataFromFile(filePath: string): Promise<unknown> {
  const content = await Deno.readTextFile(filePath);
  const ext = extname(filePath).toLowerCase();

  let data: unknown;
  if (ext === ".json") {
    data = JSON.parse(content);
  } else if (ext === ".yaml" || ext === ".yml") {
    data = parseYaml(content);
  } else {
    throw new Error(
      `Unsupported file format: ${ext}. Use .json, .yaml, or .yml`,
    );
  }

  return data;
}

/**
 * Check if a Zod schema has any required fields (non-optional).
 *
 * @param schema - The Zod schema to check
 * @returns true if the schema has required fields, false otherwise
 */
export function schemaHasRequiredFields(schema: z.ZodSchema): boolean {
  try {
    // Try parsing an empty object - if it fails, there are required fields
    const result = schema.safeParse({});
    return !result.success;
  } catch {
    // If we can't determine, assume there are required fields to be safe
    return true;
  }
}

/**
 * Load and validate input data from a JSON or YAML file.
 * Sets the input data on the provided context after validation.
 *
 * @param tctx - Template context to populate
 * @param filePath - Path to the input data file
 */
export async function loadInputData(
  tctx: TemplateContext,
  filePath: string,
): Promise<void> {
  const ctx = Context.instance();

  ctx.logger.info("Loading input data from: {filePath}", { filePath });

  // Load the data from file
  const data = await loadDataFromFile(filePath);

  // Validate it's an object
  if (typeof data !== "object" || data === null) {
    throw new Error(`Input file must contain an object`);
  }

  // Get the input schema
  const inputSchema = tctx.inputs();

  // If no schema is defined, just set the data without validation
  if (!inputSchema) {
    ctx.logger.warn("No input schema defined; skipping validation");
    tctx.inputData(data as Record<string, unknown>);
    return;
  }

  // Validate the data against the schema
  ctx.logger.debug("Validating input data");
  const result = inputSchema.safeParse(data);

  if (!result.success) {
    // Format zod errors in a readable way
    const errors = result.error.issues.map((err: z.ZodIssue) => {
      const path = err.path.length > 0 ? err.path.join(".") : "root";
      return `  - ${path}: ${err.message}`;
    }).join("\n");

    const errorMessage = `Input validation failed:\n${errors}`;
    ctx.logger.error(errorMessage);
    throw new Error(errorMessage);
  }

  // Set the validated data
  ctx.logger.debug("Input data validated");
  tctx.inputData(result.data as Record<string, unknown>);
  ctx.logger.debug("Input data: {data}", { data: result.data });
}
