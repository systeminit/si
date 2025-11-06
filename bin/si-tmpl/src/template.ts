import { Context } from "./context.ts";
import { isAbsolute, resolve, toFileUrl } from "@std/path";
import { transpileTemplate } from "./transpile.ts";

// Re-export all types and functions from submodules
export type {
  AttributePredicate,
  ComponentViewV1,
  NamePattern,
  TemplateComponent,
  TemplateContextOptions,
  TransformFunction,
} from "./template/context.ts";
export {
  componentViewToTemplateComponent,
  filterComponentAttributes,
  TemplateContext,
} from "./template/context.ts";
export {
  loadDataFromFile,
  loadInputData,
  schemaHasRequiredFields,
} from "./template/input.ts";
export { loadBaselineFromFile, setBaseline } from "./template/baseline.ts";
export { getHeadChangeSetId } from "./si_client.ts";
export type { BaselineCache } from "./template/cache.ts";
export { cacheBaseline } from "./template/cache.ts";
export { initializeWorkingSet } from "./template/working_set.ts";
export { updateNamesOfWorkingSet } from "./template/names.ts";
export { applyTransform } from "./template/transform.ts";
export { queryExistingSet } from "./template/existing_set.ts";
export {
  attributeDiffToUpdatePayload,
  computeAttributeDiff,
  isEmptyDiff,
} from "./template/attribute_diff.ts";
export type {
  AttributeDiff,
  ComponentChange,
  CreateChange,
  DeleteChange,
  ExistingSetComponent,
  PendingChanges,
  SubscriptionSource,
  UpdateChange,
} from "./template/converge_types.ts";
export { buildPendingChanges } from "./template/pending_changes.ts";
export { rewriteSubscriptions } from "./template/subscriptions.ts";
export { topologicalSort } from "./template/topology.ts";
export { executeChanges } from "./template/execute.ts";
export { convergeTemplate } from "./template/converge.ts";

// Import for internal use
import {
  TemplateContext,
  type TemplateContextOptions,
} from "./template/context.ts";
import { loadInputData, schemaHasRequiredFields } from "./template/input.ts";
import { loadBaselineFromFile, setBaseline } from "./template/baseline.ts";
import { cacheBaseline } from "./template/cache.ts";
import { initializeWorkingSet } from "./template/working_set.ts";
import { updateNamesOfWorkingSet } from "./template/names.ts";
import { applyTransform } from "./template/transform.ts";
import { convergeTemplate } from "./template/converge.ts";
import { z } from "zod";

/**
 * Zod schema for validating subscription input data.
 *
 * Subscriptions allow components to receive values from other components. This schema
 * defines two types of subscriptions:
 * - `search`: Find source component using a search query
 * - `$source`: Reference source component by name or ID
 *
 * Both types specify a `path` to the source attribute and an optional `func` transformation.
 *
 * @see {@link TemplateContext.setSubscription} for usage examples
 */
export const SubscriptionInput: z.ZodDiscriminatedUnion<
  [
    z.ZodObject<{
      kind: z.ZodLiteral<"search">;
      query: z.ZodString;
      path: z.ZodString;
      func: z.ZodOptional<z.ZodString>;
    }>,
    z.ZodObject<{
      kind: z.ZodLiteral<"$source">;
      component: z.ZodString;
      path: z.ZodString;
      func: z.ZodOptional<z.ZodString>;
    }>,
  ],
  "kind"
> = z.discriminatedUnion("kind", [
  z.object({
    kind: z.literal("search"),
    query: z.string(),
    path: z.string(),
    func: z.string().optional(),
  }),
  z.object({
    kind: z.literal("$source"),
    component: z.string(),
    path: z.string(),
    func: z.string().optional(),
  }),
]).describe("component subscription using either search or $source");

/**
 * TypeScript type inferred from the SubscriptionInput Zod schema.
 *
 * Used by {@link TemplateContext.setSubscription} to specify subscription configuration.
 *
 * @example
 * ```ts
 * const subscription: SubscriptionInputType = {
 *   kind: "search",
 *   query: 'name:"postgres-db"',
 *   path: "/domain/connectionString"
 * };
 * ```
 */
export type SubscriptionInputType = z.infer<typeof SubscriptionInput>;

function createTemplateContext(
  templatePath: string,
  options: TemplateContextOptions,
  // deno-lint-ignore no-explicit-any
): TemplateContext<any> {
  return new TemplateContext(templatePath, options);
}

/**
 * Main entry point for running a System Initiative template.
 *
 * This function orchestrates the complete template execution pipeline:
 * 1. Load and transpile the template file (if TypeScript)
 * 2. Execute the template to configure the TemplateContext
 * 3. Load and validate input data (if provided)
 * 4. Load or query the baseline component set
 * 5. Cache baseline (if requested)
 * 6. Initialize working set from baseline
 * 7. Apply name transformations
 * 8. Execute custom transform function
 * 9. Converge changes to System Initiative workspace
 *
 * @param template - Path or URL to the template file (.ts or .js)
 * @param options - Configuration options for template execution
 * @throws Error if template is invalid, required inputs are missing, or API calls fail
 *
 * @example
 * ```ts
 * import { runTemplate } from "@systeminit/template";
 *
 * await runTemplate("./my-template.ts", {
 *   key: "production-v1",
 *   input: "./inputs.json",
 *   cacheBaseline: "./baseline.json"
 * });
 * ```
 */
export async function runTemplate(
  template: string,
  options: TemplateContextOptions,
) {
  const ctx = Context.instance();

  const specifier = /^https?:\/\//.test(template)
    ? template
    : toFileUrl(isAbsolute(template) ? template : resolve(template)).href;
  ctx.logger.info(`Loading Template: {specifier}`, { specifier });

  // Check if this is a TypeScript file that needs transpilation
  const isTypeScript = specifier.endsWith(".ts") || specifier.endsWith(".tsx");

  let importSpecifier = specifier;
  let tempFilePath: string | undefined;

  if (isTypeScript) {
    try {
      // Transpile TypeScript to JavaScript and write to temp file
      tempFilePath = await transpileTemplate(specifier);
      // Convert the temp file path to a file URL for importing
      importSpecifier = toFileUrl(tempFilePath).href;
      ctx.logger.debug(`Using transpiled file for import: {path}`, {
        path: tempFilePath,
      });
    } catch (error) {
      ctx.logger.error(
        `Failed to transpile TypeScript template: {message}`,
        { message: error instanceof Error ? error.message : String(error) },
      );
      throw error;
    }
  }

  try {
    // deno-lint-ignore no-explicit-any
    const mod = await import(importSpecifier) as any;
    const run = typeof mod === "function"
      ? mod
      : typeof mod.default === "function"
      ? mod.default
      : mod.run;

    if (typeof run !== "function") {
      ctx.logger.error(
        "The module must export a function (default) or a named run(ctx)",
      );
      Deno.exit(1);
    }
    const tctx = createTemplateContext(template, options);
    await run(tctx);

    // Check if input data is required but not provided
    const inputSchema = tctx.inputs();
    if (inputSchema && schemaHasRequiredFields(inputSchema) && !options.input) {
      ctx.logger.error(
        "Template defines required input fields but no input file was provided. Use --input to specify an input data file.",
      );
      Deno.exit(1);
    }

    // Load and validate input data if provided
    if (options.input) {
      await loadInputData(tctx, options.input);
    }

    // Load baseline from file if provided
    if (options.baseline) {
      await loadBaselineFromFile(tctx, options.baseline);
    }

    // Set baseline (will skip if already set from file or template)
    await setBaseline(tctx);

    // Cache baseline if requested
    if (options.cacheBaseline) {
      await cacheBaseline(tctx, options.cacheBaseline);

      // Exit early if only caching baseline
      if (options.cacheBaselineOnly) {
        ctx.logger.info("Baseline cache written successfully. Exiting.");
        Deno.exit(0);
      }
    } else if (options.cacheBaselineOnly) {
      // Error if --cache-baseline-only used without --cache-baseline
      ctx.logger.error(
        "--cache-baseline-only requires --cache-baseline to be specified",
      );
      Deno.exit(1);
    }

    // Initialize working set from baseline (deep copy)
    initializeWorkingSet(tctx);

    // Update names in working set using namePattern if defined
    updateNamesOfWorkingSet(tctx);

    // Apply transformation function to working set if defined
    await applyTransform(tctx);

    // Converge the template to the change set
    await convergeTemplate(tctx, options.dryRun || false);
  } finally {
    // Clean up temporary transpiled file if it was created
    if (tempFilePath) {
      try {
        await Deno.remove(tempFilePath);
        ctx.logger.debug(`Cleaned up transpiled file: {path}`, {
          path: tempFilePath,
        });
      } catch (cleanupError) {
        // Ignore cleanup errors - file might not exist or already be deleted
        ctx.logger.debug(
          `Failed to clean up transpiled file: {message}`,
          {
            message: cleanupError instanceof Error
              ? cleanupError.message
              : String(cleanupError),
          },
        );
      }
    }
  }
}
