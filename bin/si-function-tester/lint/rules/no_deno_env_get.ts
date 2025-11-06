/**
 * Lint rule to prevent direct usage of Deno.env.get() which bypasses
 * proper environment variable handling and configuration management.
 */
export const noDenoEnvGet = {
  // deno-lint-ignore no-explicit-any
  create(context: any) {
    return {
      // Match calls to Deno.env.get()
      'CallExpression[callee.object.object.name="Deno"][callee.object.property.name="env"][callee.property.name="get"]'(
        // deno-lint-ignore no-explicit-any
        node: any,
      ) {
        context.report({
          node,
          message:
            "Direct usage of Deno.env.get() is not allowed. Use proper CLI parsing and configuration injection methods instead.",
        });
      },
    };
  },
};
