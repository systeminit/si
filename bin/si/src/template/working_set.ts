import type { TemplateContext } from "./context.ts";

/**
 * Initialize the working set from the baseline.
 * Creates a deep copy of the baseline data to ensure the working set is independent.
 *
 * @param ctx - The template context
 */
export function initializeWorkingSet(ctx: TemplateContext): void {
  const baseline = ctx.baseline();

  if (!baseline) {
    ctx.logger.warn("Cannot initialize working set: baseline is not set");
    return;
  }

  ctx.logger.info("Initializing working set: {count} components", {
    count: baseline.length,
  });

  // Create a deep copy to ensure independence from baseline
  const workingSetCopy = structuredClone(baseline);
  ctx.workingSet(workingSetCopy);
}
