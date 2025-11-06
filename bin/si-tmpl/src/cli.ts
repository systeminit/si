/**
 * CLI Module - SI Template Command-Line Interface
 *
 * This module provides the primary command-line interface for the SI Template
 * tool, which helps generate and manage templates for System Initiative.
 *
 * @module
 */

import { Command } from "@cliffy/command";
import { CompletionsCommand } from "@cliffy/command/completions";
import { Context } from "./context.ts";
import { extractErrorDetails, unknownValueToErrorMessage } from "./helpers.ts";
import { runTemplate, TemplateContextOptions } from "./template.ts";
import axios from "axios";

/** Current version of the SI Template CLI */
const VERSION = "0.1.0";

/**
 * Global options available to all commands
 */
export type GlobalOptions = {
  verbose?: number;
  noColor?: boolean;
};

/**
 * Main entry point for the CLI application.
 *
 * Parses command-line arguments and dispatches to the appropriate command
 * handler. This function is called from the main script to start the CLI.
 *
 * @example
 * ```ts
 * import { start } from "./cli.ts";
 * await start();
 * ```
 */
export async function start() {
  let exitCode = 0;
  let ctx: Context | undefined;

  try {
    await buildCommand().parse(Deno.args);
  } catch (error) {
    const errorMsg = unknownValueToErrorMessage(error);

    if (Context.isInitialized()) {
      ctx = Context.instance();
    } else {
      // Create a minimal context for error logging
      ctx = await Context.init({ verbose: 0 });
    }

    ctx.logger.error(errorMsg);

    // Print detailed error information for Axios errors
    if (axios.isAxiosError(error)) {
      ctx.logger.error("Axios error details: {details}", {
        details: extractErrorDetails(error),
      });
    }

    // Print stack trace if available
    if (error instanceof Error && error.stack) {
      ctx.logger.error("Stack trace: {stack}", { stack: error.stack });
    }

    const [_command, ..._args] = Deno.args;

    exitCode = 1;
  }

  if (ctx) {
    await ctx.analytics.shutdown();
  }

  Deno.exit(exitCode);
}

/**
 * Creates and configures the main CLI command structure.
 *
 * This function builds the complete command tree with all subcommands, options,
 * and their respective handlers.
 *
 * @returns The configured root Command instance
 * @internal
 */
function buildCommand() {
  return new Command()
    .name("si-tmpl")
    .version(VERSION)
    .description(
      "A command-line tool for managing System Initiative templates",
    )
    .globalOption(
      "-v, --verbose [level:number]",
      "Enable verbose logging (0=errors only, 1=+warnings, 2=+info, 3=+debug, 4=+trace)",
      { default: 2, value: (value) => (value === true ? 2 : value) },
    )
    .globalOption("--no-color", "Disable colored output")
    .globalAction(async (options) => {
      await Context.init({ ...options });
    })
    .action(function () {
      this.showHelp();
    })
    .command(
      "run",
      new Command()
        .description("Run a SI template file")
        .arguments("<template:string")
        .env("SI_API_TOKEN=<value:string>", "A System Initiative API Token", {
          required: true,
        })
        .env(
          "SI_BASE_URL=<url:string>",
          "The System Initiative Base URL for your workspace",
        )
        .option(
          "-k, --key <invocationKey:string>",
          "the invocation key for the template; used for idempotency",
          { required: true },
        )
        .option(
          "-i, --input <file:string>",
          "path to input data file (JSON or YAML); validated against template's input schema",
        )
        .option(
          "-b, --baseline <file:string>",
          "path to baseline data file (JSON or YAML)",
        )
        .option(
          "-c, --cache-baseline <file:string>",
          "path to cache baseline results; format (JSON/YAML) determined by file extension (.json, .yaml, .yml)",
        )
        .option(
          "--cache-baseline-only",
          "exit after writing baseline cache (requires --cache-baseline)",
        )
        .option(
          "--dry-run",
          "Show planned changes without executing them",
        )
        .action(async (options, template) => {
          // Validate SI_API_TOKEN is present before proceeding
          const apiToken = Deno.env.get("SI_API_TOKEN");
          if (!apiToken) {
            const ctx = Context.instance();
            ctx.logger.error(
              "SI_API_TOKEN is not defined; re-run with your authentication token set in the environment",
            );
            Deno.exit(10);
          }

          await runTemplate(
            template as string,
            options as TemplateContextOptions,
          );
        }),
    )
    .command("completion", new CompletionsCommand());
}
