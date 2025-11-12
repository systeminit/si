import ejs from "ejs";
import type { TemplateContext } from "./context.ts";

/**
 * Update the names of components in the working set using the namePatterns.
 * Applies patterns sequentially in array order to both the component.name field
 * and the /si/name attribute. Each pattern is applied to the result of the previous pattern.
 * The replacement string is evaluated as an EJS template with access to:
 * - `inputs`: the validated input data
 * - `c`: the template context
 *
 * @param ctx - The template context
 */
export function updateNamesOfWorkingSet(ctx: TemplateContext): void {
  const namePatterns = ctx.namePattern();
  const workingSet = ctx.workingSet();

  // Early return if no name patterns are defined (no-op)
  if (!namePatterns || namePatterns.length === 0) {
    return;
  }

  // Early return if working set is not initialized (no-op)
  if (!workingSet) {
    return;
  }

  // Apply each pattern sequentially
  for (let i = 0; i < namePatterns.length; i++) {
    const namePattern = namePatterns[i];

    // Evaluate the replacement string as an EJS template
    let evaluatedReplacement: string;
    try {
      const ejsContext = {
        inputs: ctx.inputData(),
        c: ctx,
      };
      evaluatedReplacement = ejs.render(namePattern.replacement, ejsContext);
      ctx.logger.debug(
        `Pattern {index}: EJS template evaluated: "{template}" -> "{result}"`,
        {
          index: i + 1,
          template: namePattern.replacement,
          result: evaluatedReplacement,
        },
      );
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      ctx.logger.error(
        `Failed to evaluate EJS template in namePattern[{index}] replacement: {error}`,
        {
          index: i,
          error: errorMsg,
        },
      );
      throw new Error(
        `EJS template evaluation failed for pattern ${i}: ${errorMsg}`,
      );
    }

    ctx.logger.info(
      `Applying pattern {index}/{total}: {pattern} -> {replacement}`,
      {
        index: i + 1,
        total: namePatterns.length,
        pattern: namePattern.pattern.source,
        replacement: evaluatedReplacement,
      },
    );

    for (const component of workingSet) {
      // Apply pattern to component name
      const original = component.name;
      component.name = component.name.replace(
        namePattern.pattern,
        evaluatedReplacement,
      );
      if (original !== component.name) {
        ctx.logger.trace("Renamed {original} to {new}", {
          original,
          new: component.name,
        });
      }

      // Apply pattern to /si/name attribute if it exists
      if (component.attributes["/si/name"]) {
        component.attributes["/si/name"] =
          (component.attributes["/si/name"] as string).replace(
            namePattern.pattern,
            evaluatedReplacement,
          );
      }
    }
  }
}
