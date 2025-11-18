import type { TemplateContext } from "./context.ts";

/**
 * Apply the transform function to the working set if one has been defined.
 * The transform function receives the working set and input data as arguments,
 * and should return the modified working set. The working set on the context
 * is then updated with the returned value.
 *
 * @param ctx - The template context
 */
export async function applyTransform(ctx: TemplateContext): Promise<void> {
  const transformFn = ctx.transform();
  const workingSet = ctx.workingSet();

  // Early return if no transform function is defined (no-op)
  if (!transformFn) {
    return;
  }

  // Early return if working set is not initialized (no-op)
  if (!workingSet) {
    return;
  }

  ctx.logger.info("Executing transformation function");

  // Call the transform function with working set and input data
  const inputData = ctx.inputData();
  const transformedWorkingSet = await transformFn(workingSet, inputData);

  // If transform function doesn't return anything, use original working set
  if (!transformedWorkingSet) {
    ctx.logger.warn(
      "Transform function did not return a working set - using original working set. " +
        "Make sure your transform function returns the working set at the end.",
    );
    ctx.workingSet(workingSet);
    return;
  }

  // Validate that the returned value is an array
  if (!Array.isArray(transformedWorkingSet)) {
    throw new Error(
      `Transform function must return an array of components, but returned ${typeof transformedWorkingSet}`,
    );
  }

  ctx.logger.debug(`Transform function returned {count} components`, {
    count: transformedWorkingSet.length,
  });

  // Update the working set with the transformed result
  ctx.workingSet(transformedWorkingSet);
}
