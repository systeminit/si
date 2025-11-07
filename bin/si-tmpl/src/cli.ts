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
import { runTemplate, type TemplateContextOptions } from "./template.ts";
import { type ComponentGetOptions, getComponent } from "./component/get.ts";
import {
  componentUpdate,
  type ComponentUpdateOptions,
} from "./component/update.ts";
import {
  deleteComponent,
  type ComponentDeleteOptions,
} from "./component/delete.ts";
import {
  componentSearch,
  type ComponentSearchOptions,
} from "./component/search.ts";
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
    .command(
      "component",
      new Command()
        .description("Component-related operations")
        .command(
          "get",
          new Command()
            .description("Get component data by name or ID")
            .arguments("<component:string>")
            .env(
              "SI_API_TOKEN=<value:string>",
              "A System Initiative API Token",
              {
                required: true,
              },
            )
            .env(
              "SI_BASE_URL=<url:string>",
              "The System Initiative Base URL for your workspace",
            )
            .option(
              "-c, --change-set <id:string>",
              "Change set ID or name (defaults to HEAD)",
            )
            .option(
              "-o, --output <format:string>",
              "Output format: info (default), json, or yaml",
              { default: "info" },
            )
            .option(
              "--cache <file:string>",
              "Cache output to file; format (JSON/YAML) determined by file extension (.json, .yaml, .yml)",
            )
            .option(
              "--raw",
              "Output raw API response as JSON and exit",
            )
            .action(async (options, component) => {
              // Validate SI_API_TOKEN is present before proceeding
              const apiToken = Deno.env.get("SI_API_TOKEN");
              if (!apiToken) {
                const ctx = Context.instance();
                ctx.logger.error(
                  "SI_API_TOKEN is not defined; re-run with your authentication token set in the environment",
                );
                Deno.exit(10);
              }

              await getComponent(
                component as string,
                options as ComponentGetOptions,
              );
            }),
        )
        .command(
          "update",
          new Command()
            .description(
              "Update a component from JSON/YAML file (idempotent)",
            )
            .arguments("<input-file:string>")
            .env(
              "SI_API_TOKEN=<value:string>",
              "A System Initiative API Token",
              {
                required: true,
              },
            )
            .env(
              "SI_BASE_URL=<url:string>",
              "The System Initiative Base URL for your workspace",
            )
            .option(
              "--component <id-or-name:string>",
              "Component ID or name (overrides componentId from file)",
            )
            .option(
              "-c, --change-set <id-or-name:string>",
              "Change set ID or name",
              { required: true },
            )
            .option(
              "--dry-run",
              "Show diff without applying changes",
            )
            .action(async (options, inputFile) => {
              // Validate SI_API_TOKEN is present before proceeding
              const apiToken = Deno.env.get("SI_API_TOKEN");
              if (!apiToken) {
                const ctx = Context.instance();
                ctx.logger.error(
                  "SI_API_TOKEN is not defined; re-run with your authentication token set in the environment",
                );
                Deno.exit(10);
              }

              await componentUpdate(
                inputFile as string,
                options as ComponentUpdateOptions,
              );
            }),
        )
        .command(
          "delete",
          new Command()
            .description(
              "Delete a component by name or ID",
            )
            .arguments("<component:string>")
            .env(
              "SI_API_TOKEN=<value:string>",
              "A System Initiative API Token",
              {
                required: true,
              },
            )
            .env(
              "SI_BASE_URL=<url:string>",
              "The System Initiative Base URL for your workspace",
            )
            .option(
              "-c, --change-set <id-or-name:string>",
              "Change set ID or name",
              { required: true },
            )
            .option(
              "--dry-run",
              "Preview deletion without applying changes",
            )
            .action(async (options, component) => {
              // Validate SI_API_TOKEN is present before proceeding
              const apiToken = Deno.env.get("SI_API_TOKEN");
              if (!apiToken) {
                const ctx = Context.instance();
                ctx.logger.error(
                  "SI_API_TOKEN is not defined; re-run with your authentication token set in the environment",
                );
                Deno.exit(10);
              }

              await deleteComponent(
                component as string,
                options as ComponentDeleteOptions,
              );
            }),
        )
        .command(
          "search",
          new Command()
            .description(
              "Search for components using a search query",
            )
            .arguments("<query:string>")
            .env(
              "SI_API_TOKEN=<value:string>",
              "A System Initiative API Token",
              {
                required: true,
              },
            )
            .env(
              "SI_BASE_URL=<url:string>",
              "The System Initiative Base URL for your workspace",
            )
            .option(
              "-c, --change-set <id-or-name:string>",
              "Change set ID or name (defaults to HEAD)",
            )
            .option(
              "-o, --output <format:string>",
              "Output format: info (default), json, or yaml",
              { default: "info" },
            )
            .option(
              "-a, --attribute <path:string>",
              "Attribute paths to include in output (can be specified multiple times)",
              { collect: true },
            )
            .option(
              "--full-component",
              "Show full component details for each result",
            )
            .action(async (options, query) => {
              // Validate SI_API_TOKEN is present before proceeding
              const apiToken = Deno.env.get("SI_API_TOKEN");
              if (!apiToken) {
                const ctx = Context.instance();
                ctx.logger.error(
                  "SI_API_TOKEN is not defined; re-run with your authentication token set in the environment",
                );
                Deno.exit(10);
              }

              await componentSearch(
                query as string,
                options as ComponentSearchOptions,
              );
            }),
        ),
    )
    .command("completion", new CompletionsCommand());
}
