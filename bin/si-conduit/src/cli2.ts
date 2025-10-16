/**
 * CLI Module - SI Conduit Command-Line Interface
 *
 * This module provides the primary command-line interface for the SI Conduit
 * tool, which helps generate and manage schema-related code for System
 * Initiative projects.
 *
 * @module
 */

import { Command } from "@cliffy/command";
import { CompletionsCommand } from "@cliffy/command/completions";
import * as prompt from "./cli/prompt.ts";
import { RootPath, RootPathType } from "./cli/root-path.ts";
import { call_schema_action_generate } from "./command/schema/action/generate.ts";
import { call_schema_codegen_generate } from "./command/schema/codegen/generate.ts";
import { call_schema_scaffold_generate } from "./command/schema/scaffold/generate.ts";
import { Project } from "./project.ts";

/** Current version of the SI Conduit CLI */
const VERSION = "0.1.0";

/** Environment variable prefix */
const ENV_VAR_PREFIX = "SI_CONDUIT_";

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
  await buildCommand().parse(Deno.args);
}

/**
 * Creates and configures the main CLI command structure.
 *
 * This function builds the complete command tree with all subcommands, options,
 * and their respective handlers. It:
 * - Registers global types for custom argument parsing (e.g., root-path)
 * - Sets up global environment variables and options
 * - Defines all command hierarchies and their actions
 * - Configures interactive prompts for missing arguments
 *
 * @returns The configured root Command instance
 * @internal
 */
function buildCommand() {
  return new Command()
    .name("si-conduit")
    .version(VERSION)
    .description("FIXME: System Initiative Conduit CLI")
    .globalType("root-path", new RootPathType())
    .globalEnv(
      "SI_CONDUIT_ROOT=<path:root-path>",
      "Project root directory (searches for .conduitroot if not specified)",
      { prefix: ENV_VAR_PREFIX },
    )
    .globalOption(
      "--root <path:root-path>",
      "Project root directory (searches for .conduitroot if not specified)",
    )
    .action(function () {
      this.showHelp();
    })
    .command("completion", new CompletionsCommand())
    .command("schema", buildSchemaCommand())
    .command("remote", buildRemoteCommand());
}

/**
 * Builds the schema command group with all subcommands.
 *
 * @returns A SubCommand configured for schema operations
 * @internal
 */
function buildSchemaCommand() {
  return createSubCommand()
    .description("FIXME: schema desc")
    .action(function () {
      this.showHelp();
    })
    .command("action", buildSchemaActionCommand())
    .command("codegen", buildSchemaCodegenCommand())
    .command("management", buildSchemaManagementCommand())
    .command("qualification", buildSchemaQualificationCommand())
    .command("scaffold", buildSchemaScaffoldCommand());
}

/**
 * Builds the remote command group with all subcommands.
 *
 * @returns A SubCommand configured for remote operations
 * @internal
 */
function buildRemoteCommand() {
  return createSubCommand()
    .description("FIXME: remote desc")
    .action(function () {
      this.showHelp();
    })
    .command(
      "push",
      createSubCommand()
        .description("FIXME: push desc")
        .action((options) => {
          console.log(options, "options");
        }),
    );
}

/**
 * Builds the schema action subcommands.
 *
 * @returns A SubCommand configured for action operations
 * @internal
 */
function buildSchemaActionCommand() {
  return createSubCommand()
    .description("FIXME: action desc")
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description("FIXME: generate desc")
        .arguments("[SCHEMA_NAME:string] [ACTION_NAME:string]")
        .action(async ({ root }, schemaName, actionName) => {
          const project = createProject(root);
          const finalSchemaName = await prompt.schemaName(schemaName, project);
          const finalActionName = await prompt.actionName(actionName, project);

          await call_schema_action_generate(
            project,
            finalSchemaName,
            finalActionName,
          );
        }),
    );
}

/**
 * Builds the schema codegen subcommands.
 *
 * @returns A SubCommand configured for codegen operations
 * @internal
 */
function buildSchemaCodegenCommand() {
  return createSubCommand()
    .description("FIXME: codegen desc")
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description("FIXME: generate desc")
        .arguments("[SCHEMA_NAME:string] [CODEGEN_NAME:string]")
        .action(async ({ root }, schemaName, codegenName) => {
          const project = createProject(root);
          const finalSchemaName = await prompt.schemaName(schemaName, project);
          const finalCodegenName = await prompt.codegenName(codegenName);

          await call_schema_codegen_generate(
            project,
            finalSchemaName,
            finalCodegenName,
          );
        }),
    );
}

/**
 * Builds the schema management subcommands.
 *
 * @returns A SubCommand configured for management operations
 * @internal
 */
function buildSchemaManagementCommand() {
  return createSubCommand()
    .description("FIXME: management desc")
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description("FIXME: generate desc")
        .action((options) => {
          console.log(options, "options");
        }),
    );
}

/**
 * Builds the schema qualification subcommands.
 *
 * @returns A SubCommand configured for qualification operations
 * @internal
 */
function buildSchemaQualificationCommand() {
  return createSubCommand()
    .description("FIXME: qualification desc")
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description("FIXME: generate desc")
        .action((options) => {
          console.log(options, "options");
        }),
    );
}

/**
 * Builds the schema scaffold subcommands.
 *
 * @returns A SubCommand configured for scaffold operations
 * @internal
 */
function buildSchemaScaffoldCommand() {
  return createSubCommand()
    .description("FIXME: scaffold desc")
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description("FIXME: generate desc")
        .arguments("[SCHEMA_NAME:string]")
        .action(async ({ root }, schemaName) => {
          const project = createProject(root);
          const finalSchemaName = await prompt.schemaName(schemaName, project);

          await call_schema_scaffold_generate(project, finalSchemaName);
        }),
    );
}

/** Creates a new SubCommand with root path options configured */
function createSubCommand(): Command<
  { root?: RootPath },
  { rootPath: typeof RootPathType }
> {
  return new Command();
}

/**
 * Creates a Project instance from the given root path or discovers it from the
 * current directory.
 *
 * This helper function handles the common pattern of creating a Project:
 * - If a root path is provided, uses it directly
 * - If no root is provided, searches for `.conduitroot` from the current
 *   working directory
 *
 * @param root - Optional RootPath instance specifying the project root
 * @returns A Project instance configured with the resolved root path
 * @throws Error if no `.conduitroot` file is found when discovering from cwd
 *
 * @example
 * ```ts
 * // Use explicit root
 * const project = createProject(rootPath);
 *
 * // Discover from current directory
 * const project = createProject();
 * ```
 * @internal
 */
function createProject(root?: RootPath): Project {
  return root ? root.toProject() : RootPath.findFromCwd().toProject();
}
