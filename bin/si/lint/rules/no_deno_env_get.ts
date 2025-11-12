/**
 * Lint rule to prevent direct usage of Deno.env.get() which bypasses
 * proper environment variable handling and configuration management.
 *
 * Exceptions are allowed for bootstrap/initialization files like context.ts,
 * config.ts, and si_client.ts which need to read environment variables directly.
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
        // Allow Deno.env.get() in specific bootstrap/initialization files
        const filename = context.getFilename();
        const allowedFiles = [
          'context.ts',
          'config.ts',
          'si_client.ts',
        ];

        const isAllowed = allowedFiles.some(file => filename.endsWith(file));

        if (!isAllowed) {
          context.report({
            node,
            message:
              "Direct usage of Deno.env.get() is not allowed. Use proper CLI parsing and configuration injection methods instead.",
          });
        }
      },
    };
  },
};
