/**
 * Custom lint plugin to enforce code quality rules for System Initiative Deno
 * codebases.
 */
import * as rules from "./rules/mod.ts";

const plugin: Deno.lint.Plugin = {
  name: "si-rules",
  rules: {
    "no-deno-env-get": rules.noDenoEnvGet,
  },
};

export default plugin;
