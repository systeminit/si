/**
 * CLI Module - SI Command-Line Interface
 *
 * This module provides the primary command-line interface for the SI CLI
 * tool, which helps manage System Initiative schemas, templates, and components.
 *
 * @module
 */

import { Command, ValidationError } from "@cliffy/command";
import { CompletionsCommand } from "@cliffy/command/completions";
import * as prompt from "./cli/prompt.ts";
import {
  RootPath,
  type RootPathNotFoundError,
  RootPathType,
} from "./cli/root-path.ts";
import { initializeCliContextWithAuth } from "./cli/helpers.ts";
import { callWhoami } from "./whoami.ts";
import { callProjectInit } from "./schema/init.ts";
import { callRemoteSchemaPull } from "./schema/pull.ts";
import {
  callSchemaFuncGenerate,
  callSchemaScaffoldGenerate,
} from "./schema/generate.ts";
import { type ApiContext, apiContext } from "./cli/api.ts";
import { unknownValueToErrorMessage } from "./helpers.ts";
import { Context } from "./context.ts";
import * as jwt from "./cli/jwt.ts";
import { FunctionKind, Project } from "./schema/project.ts";
import { VERSION } from "./git_metadata.ts";
import {
  callRemoteSchemaOverlaysPush,
  callRemoteSchemaPush,
} from "./schema/push.ts";
import { callRunTemplate } from "./template/run.ts";
import { callComponentGet } from "./component/get.ts";
import { callComponentUpdate } from "./component/update.ts";
import { callComponentDelete } from "./component/delete.ts";
import { callComponentSearch } from "./component/search.ts";
import type { TemplateContextOptions } from "./template/run.ts";
import type { ComponentGetOptions } from "./component/get.ts";
import type { ComponentUpdateOptions } from "./component/update.ts";
import type { ComponentDeleteOptions } from "./component/delete.ts";
import type { ComponentSearchOptions } from "./component/search.ts";
import {
  callAiAgentInit,
  type AiAgentInitOptions,
} from "./ai-agent/init.ts";
import {
  callAiAgentStart,
  type AiAgentStartOptions,
} from "./ai-agent/start.ts";
import {
  callAiAgentConfig,
  type AiAgentConfigOptions,
} from "./ai-agent/config.ts";

/**
 * Global options available to all commands
 */
export type GlobalOptions = {
  apiBaseUrl: string;
  apiToken?: string;
  noColor?: boolean;
  root?: RootPath | RootPathNotFoundError;
  verbose?: number;
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

    // Log error with stack trace for debugging
    ctx.logger.error(errorMsg);
    if (error instanceof Error && error.stack) {
      console.error(error.stack);
    }

    const [command, ...args] = Deno.args;

    ctx.analytics.trackEvent("cli_error", {
      error: errorMsg,
      command,
      args,
    });

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
    .name("si")
    .version(VERSION)
    .description(
      "A command-line tool for managing System Initiative schemas, templates, and components",
    )
    .globalType("root-path", new RootPathType())
    .globalEnv("SI_API_BASE_URL=<URL:string>", "API endpoint URL", {
      prefix: "SI_",
    })
    .globalOption("--api-base-url <URL:string>", "API endpoint URL", {
      default: "https://api.systeminit.com",
    })
    .globalEnv(
      "SI_API_TOKEN=<TOKEN:string>",
      "Your System Initiative API token (required for authenticated commands)",
      { prefix: "SI_" },
    )
    .globalOption(
      "--api-token <TOKEN:string>",
      "Your System Initiative API token (required for authenticated commands)",
    )
    .globalEnv(
      "SI_ROOT=<PATH:root-path>",
      "Project root directory (searches for .siroot if not specified)",
      { prefix: "SI_" },
    )
    .globalOption(
      "--root <PATH:root-path>",
      "Project root directory (searches for .siroot if not specified)",
    )
    .globalOption(
      "-v, --verbose [level:number]",
      "Enable verbose logging (0=errors only, 1=+warnings, 2=+info, 3=+debug, 4=+trace)",
      { default: 2, value: (value) => (value === true ? 2 : value) },
    )
    .globalOption("--no-color", "Disable colored output")
    .globalAction(async (options) => {
      let userData: ReturnType<typeof jwt.getUserDataFromToken>;
      try {
        userData = jwt.getUserDataFromToken(options.apiToken);
      } catch (error) {
        // If token decode fails, just continue without user data
        // This allows MCP server commands to run even with invalid tokens in env
        userData = undefined;
      }
      await Context.init({ ...options, userData });
    })
    .action(function () {
      this.showHelp();
    })
    .command("completion", new CompletionsCommand())
    // deno-lint-ignore no-explicit-any
    .command("ai-agent", buildAiAgentCommand() as any)
    // deno-lint-ignore no-explicit-any
    .command("component", buildComponentCommand() as any)
    // deno-lint-ignore no-explicit-any
    .command("schema", buildSchemaCommand() as any)
    // deno-lint-ignore no-explicit-any
    .command("template", buildTemplateCommand() as any)
    // deno-lint-ignore no-explicit-any
    .command("whoami", buildWhoamiCommand() as any);
}

/**
 * Builds the schema command group with all subcommands.
 *
 * @returns A SubCommand configured for schema operations
 * @internal
 */
function buildSchemaCommand() {
  return createSubCommand()
    .description("Manage schemas: initialize project, generate functions locally, pull from and push to remote workspaces")
    .action(function () {
      this.showHelp();
    })
    .command("init", buildInitCommand())
    .command("generate", buildSchemaGenerateCommand())
    .command("overlay", buildOverlayCommand())
    .command(
      "pull",
      createSubCommand()
        .description(
          "Pulls schemas from your remote System Initiative workspace. " +
            "Supports wildcard patterns like 'Fastly::*' to pull all schemas in a category, " +
            "or '*' to pull all schemas.",
        )
        .arguments("[...SCHEMA_NAME:string]")
        .option(
          "--builtins",
          "Include builtin schemas (schemas you don't own). By default, builtins are skipped.",
        )
        .action(
          async ({ root, apiBaseUrl, apiToken, builtins }, ...schemaNames) => {
            const project = createProject(root);
            const apiCtx = await createApiContext(apiBaseUrl, apiToken);
            let finalSchemaNames;
            if (schemaNames.length > 0) {
              finalSchemaNames = schemaNames;
            } else {
              finalSchemaNames = [await prompt.schemaName(undefined, project)];
            }

            await callRemoteSchemaPull(
              Context.instance(),
              project,
              apiCtx,
              finalSchemaNames,
              builtins ?? false,
            );
          },
        ),
    )
    .command(
      "push",
      createSubCommand()
        .description(
          "Pushes schemas to your remote System Initiative workspace",
        )
        .option("-s, --skip-confirmation", "Skip confirmation prompt")
        .option("-b, --update-builtins", "Change builtin schema, without creating overlays. SI Admin Only", {
          hidden: false,
        })
        .arguments("[...SCHEMA_NAME:string]")
        .action(async ({ root, skipConfirmation, updateBuiltins }, ...schemaNames) => {
          const project = createProject(root);

          const ctx = Context.instance();
          const cliContext = await initializeCliContextWithAuth({ ctx });

          await callRemoteSchemaPush(
            cliContext,
            project,
            schemaNames,
            !!updateBuiltins,
            skipConfirmation,
          );
        }),
    );
}

function buildSchemaGenerateCommand() {
  return createSubCommand()
    .description("Generate schema function definitions for actions, authentication, code generation, and more")
    .action(function () {
      this.showHelp();
    })
    .command("action", buildSchemaActionCommand())
    .command("authentication", buildSchemaAuthenticationCommand())
    .command("codegen", buildSchemaCodegenCommand())
    .command("management", buildSchemaManagementCommand())
    .command("qualification", buildSchemaQualificationCommand())
    .command("scaffold", buildSchemaScaffoldCommand());
}

function buildOverlayCommand() {
  return createSubCommand()
    .description("Manage schema overlays: generate overlay functions and push them to remote workspaces")
    .action(function () {
      this.showHelp();
    })
    .command("generate", buildOverlayGenerateCommand())
    .command(
      "push",
      createSubCommand()
        .description(
          "Pushes overlay funcs to your remote System Initiative workspace",
        )
        .option("-s, --skip-confirmation", "Skip confirmation prompt")
        .action(async ({ root, skipConfirmation }) => {
          const project = createProject(root);

          const ctx = Context.instance();
          const cliContext = await initializeCliContextWithAuth({ ctx });

          await callRemoteSchemaOverlaysPush(
            cliContext,
            project,
            skipConfirmation,
          );
        }),
    );
}

function buildOverlayGenerateCommand() {
  return createSubCommand()
    .description("Generate overlay function definitions that customize or extend existing schemas")
    .action(function () {
      this.showHelp();
    })
    .command("action", buildSchemaActionCommand({ isOverlay: true }))
    .command(
      "authentication",
      buildSchemaAuthenticationCommand({ isOverlay: true }),
    )
    .command("codegen", buildSchemaCodegenCommand({ isOverlay: true }))
    .command("management", buildSchemaManagementCommand({ isOverlay: true }))
    .command(
      "qualification",
      buildSchemaQualificationCommand({ isOverlay: true }),
    );
}


/**
 * Builds the whoami command.
 *
 * @returns A SubCommand configured for displaying user information
 * @internal
 */
function buildWhoamiCommand() {
  return createSubCommand()
    .description("Displays authenticated user information")
    .action(async ({ apiBaseUrl, apiToken }) => {
      const apiCtx = await createApiContext(apiBaseUrl, apiToken);

      await callWhoami(Context.instance(), apiCtx);
    });
}

/**
 * Builds the ai-agent command group with all subcommands.
 *
 * @returns A SubCommand configured for AI agent operations
 * @internal
 */
function buildAiAgentCommand() {
  return createSubCommand()
    .description("Manages the SI AI Agent (MCP server)")
    .action(function () {
      this.showHelp();
    })
    .command(
      "init",
      createSubCommand()
        .description(
          "Initialize AI agent (one-time setup: configure token and create MCP files)",
        )
        .option(
          "--target-dir <path:string>",
          "Directory to create config files (defaults to current directory)",
        )
        .option(
          "--api-token <token:string>",
          "System Initiative API token (will prompt if not provided)",
        )
        .option(
          "--tool <name:string>",
          "AI tool to use: claude (default), codex",
        )
        .action(async (options) => {
          await callAiAgentInit(Context.instance(), options as AiAgentInitOptions);
        }),
    )
    .command(
      "start",
      createSubCommand()
        .description("Launch Claude Code (MCP server starts automatically)")
        .option(
          "--tool <name:string>",
          "AI tool to launch (default: claude)",
        )
        .action(async (options) => {
          await callAiAgentStart(Context.instance(), options as AiAgentStartOptions);
        }),
    )
    .command(
      "config",
      createSubCommand()
        .description("View or update AI agent configuration")
        .option(
          "--show",
          "Show current configuration (default if no other options provided)",
        )
        .option(
          "--update-token",
          "Update the API token",
        )
        .option(
          "--tool <name:string>",
          "Update the AI tool: claude, cursor, windsurf, or none",
        )
        .action(async (options) => {
          await callAiAgentConfig(Context.instance(), options as AiAgentConfigOptions);
        }),
    )
    .command(
      "stdio",
      createSubCommand()
        .description("Run MCP server in stdio mode (for external AI tools to connect)")
        .action(async () => {
          // Dynamic import to avoid loading MCP server code until needed
          // Token validation happens in si_client.ts when modules are loaded
          const { start_stdio } = await import("./ai-agent/mcp-server/stdio_transport.ts");
          const { createServer } = await import("./ai-agent/mcp-server/server.ts");
          const { analytics } = await import("./ai-agent/mcp-server/analytics.ts");
          const { setAiAgentUserFlag } = await import("./ai-agent/mcp-server/user_state.ts");

          // Start the MCP server directly
          await analytics.trackServerStart();
          await setAiAgentUserFlag();

          const server = createServer();

          let ended = false;
          const shutdown = async (reason: string, exitCode: number | null = 0) => {
            if (ended) return;
            ended = true;
            console.log("MCP server shutdown:", reason);
            try {
              analytics.trackServerEnd();
            } catch {
              // ignore
            }
            await new Promise((r) => setTimeout(r, 25));
            if (exitCode !== null) Deno.exit(exitCode);
          };

          const onSigInt = () => shutdown("SIGINT", 0);
          const onSigTerm = () => shutdown("SIGTERM", 0);
          Deno.addSignalListener("SIGINT", onSigInt);
          Deno.addSignalListener("SIGTERM", onSigTerm);

          try {
            await start_stdio(server);
            await shutdown("transport_closed", null);
          } catch (err: unknown) {
            const name = err instanceof Error ? err.name : "unknown";
            await shutdown(`uncaught_error:${name}`, 1);
          } finally {
            Deno.removeSignalListener("SIGINT", onSigInt);
            Deno.removeSignalListener("SIGTERM", onSigTerm);
          }
        }),
    );
}

/**
 * Builds the init command for initializing new SI projects.
 *
 * @returns A SubCommand configured for project initialization
 * @internal
 */
function buildInitCommand() {
  return createSubCommand()
    .description(
      "Initialize a new System Initiative project in the current or specified directory",
    )
    .action(function () {
      this.showHelp();
    })
    .arguments("[ROOT_PATH:string]")
    .action(async ({ root }, rootPath) => {
      const logger = Context.instance().logger;

      // Both arg and option/env cannot be provided at once
      if (root && rootPath) {
        throw new ValidationError(
          "Project root provided via --root or environment variable " +
            "and as an argument; please provide either one or the other",
        );
      }

      let basePath;
      // No path provided, defaults to CWD
      if (!root && !rootPath) {
        basePath = Project.projectBasePath(Deno.cwd());
        logger.debug("base path from cwd: {path}", {
          path: basePath.toString(),
        });
      } // Arg provided
      else if (rootPath) {
        basePath = Project.projectBasePath(rootPath);
        logger.debug("base path from arg: {path}", {
          path: basePath.toString(),
        });
      } // Option/env provided and path exists
      else if (root && root instanceof RootPath) {
        basePath = Project.projectBasePath(root.path);
        logger.debug("base path from option/env (exists): {path}", {
          path: basePath.toString(),
        });
      } // Option/env provided and path does not yet exist
      else if (root) {
        basePath = Project.projectBasePath(root.path);
        logger.debug("base path from option/env (does not exist): {path}", {
          path: basePath.toString(),
        });
      } // All other scenarios are invalid
      else {
        throw new ValidationError("Failed to determine project root directory");
      }

      await callProjectInit(Context.instance(), basePath);
    });
}

/**
 * Builds the schema action subcommands.
 *
 * @returns A SubCommand configured for action operations
 * @internal
 */
function buildSchemaActionCommand(options?: { isOverlay?: boolean }) {
  const isOverlay = options?.isOverlay ?? false;
  const overlayMsg = isOverlay ? " overlay" : "";

  return createSubCommand()
    .description(`Generates action${overlayMsg} functions for schemas`)
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(
          "Generates action functions (create, destroy, refresh, update)",
        )
        .arguments("[SCHEMA_NAME:string] [ACTION_NAME:string]")
        .action(async ({ root }, schemaName, actionName) => {
          const project = createProject(root);
          const finalSchemaName = await prompt.schemaNameFromDirNames(
            schemaName,
            project,
          );
          const finalActionName = await prompt.actionName(actionName, project);

          await callSchemaFuncGenerate(
            Context.instance(),
            project,
            finalSchemaName,
            FunctionKind.Action,
            finalActionName,
            isOverlay,
          );
        }),
    );
}

/**
 * Builds the schema authentication subcommands.
 *
 * @returns A SubCommand configured for authentication operations
 * @internal
 */
function buildSchemaAuthenticationCommand(options?: { isOverlay?: boolean }) {
  const isOverlay = options?.isOverlay ?? false;
  const overlayMsg = isOverlay ? " overlay" : "";

  return createSubCommand()
    .description(`Generates authentication${overlayMsg} functions for schemas`)
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(
          "Generates authentication functions for credential validation",
        )
        .arguments("[SCHEMA_NAME:string] [AUTH_NAME:string]")
        .action(async ({ root }, schemaName, authName) => {
          const project = createProject(root);
          const finalSchemaName = await prompt.schemaNameFromDirNames(
            schemaName,
            project,
          );
          const finalAuthName = await prompt.authName(authName, project);

          await callSchemaFuncGenerate(
            Context.instance(),
            project,
            finalSchemaName,
            FunctionKind.Auth,
            finalAuthName,
            isOverlay,
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
function buildSchemaCodegenCommand(options?: { isOverlay?: boolean }) {
  const isOverlay = options?.isOverlay ?? false;
  const overlayMsg = isOverlay ? " overlay" : "";

  return createSubCommand()
    .description(`Generates code generator${overlayMsg} functions for schemas`)
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(
          "Generates code generator functions to produce configuration files",
        )
        .arguments("[SCHEMA_NAME:string] [CODEGEN_NAME:string]")
        .action(async ({ root }, schemaName, codegenName) => {
          const project = createProject(root);
          const finalSchemaName = await prompt.schemaNameFromDirNames(
            schemaName,
            project,
          );
          const finalCodegenName = await prompt.codegenName(
            codegenName,
            project,
          );

          await callSchemaFuncGenerate(
            Context.instance(),
            project,
            finalSchemaName,
            FunctionKind.Codegen,
            finalCodegenName,
            isOverlay,
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
function buildSchemaManagementCommand(options?: { isOverlay?: boolean }) {
  const isOverlay = options?.isOverlay ?? false;
  const overlayMsg = isOverlay ? " overlay" : "";

  return createSubCommand()
    .description(`Generates management${overlayMsg} functions for schemas`)
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(
          "Generates management functions for reconciliation and lifecycle operations",
        )
        .arguments("[SCHEMA_NAME:string] [MANAGEMENT_NAME:string]")
        .action(async ({ root }, schemaName, managementName) => {
          const project = createProject(root);
          const finalSchemaName = await prompt.schemaNameFromDirNames(
            schemaName,
            project,
          );
          const finalManagementName = await prompt.managementName(
            managementName,
            project,
          );

          await callSchemaFuncGenerate(
            Context.instance(),
            project,
            finalSchemaName,
            FunctionKind.Management,
            finalManagementName,
            isOverlay,
          );
        }),
    );
}

/**
 * Builds the schema qualification subcommands.
 *
 * @returns A SubCommand configured for qualification operations
 * @internal
 */
function buildSchemaQualificationCommand(options?: { isOverlay?: boolean }) {
  const isOverlay = options?.isOverlay ?? false;
  const overlayMsg = isOverlay ? " overlay" : "";

  return createSubCommand()
    .description(`Generates qualification${overlayMsg} functions for schemas`)
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(
          "Generates qualification functions to validate component state",
        )
        .arguments("[SCHEMA_NAME:string] [QUALIFICATION_NAME:string]")
        .action(async ({ root }, schemaName, qualificationName) => {
          const project = createProject(root);
          const finalSchemaName = await prompt.schemaNameFromDirNames(
            schemaName,
            project,
          );
          const finalQualificationName = await prompt.qualificationName(
            qualificationName,
            project,
          );

          await callSchemaFuncGenerate(
            Context.instance(),
            project,
            finalSchemaName,
            FunctionKind.Qualification,
            finalQualificationName,
            isOverlay,
          );
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
    .description("Scaffolds a complete schema structure")
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(
          "Scaffolds a complete schema with all default functions and metadata",
        )
        .arguments("[SCHEMA_NAME:string]")
        .action(async ({ root }, schemaName) => {
          const project = createProject(root);
          const finalSchemaName = await prompt.schemaNameFromDirNames(
            schemaName,
            project,
          );

          await callSchemaScaffoldGenerate(
            Context.instance(),
            project,
            finalSchemaName,
          );
        }),
    );
}

/**
 * Builds the template command group with all subcommands.
 *
 * @returns A SubCommand configured for template operations
 * @internal
 */
function buildTemplateCommand() {
  return createSubCommand()
    .description("Manages System Initiative templates")
    .action(function () {
      this.showHelp();
    })
    .command(
      "run",
      createSubCommand()
        .description("Run a SI template file")
        .arguments("<template:string>")
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
          await callRunTemplate(
            template as string,
            options as TemplateContextOptions,
          );
        }),
    );
}

/**
 * Builds the component command group with all subcommands.
 *
 * @returns A SubCommand configured for component operations
 * @internal
 */
function buildComponentCommand() {
  return createSubCommand()
    .description("Component-related operations")
    .action(function () {
      this.showHelp();
    })
    .command(
      "get",
      createSubCommand()
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
          await callComponentGet(
            component as string,
            options as ComponentGetOptions,
          );
        }),
    )
    .command(
      "update",
      createSubCommand()
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
          await callComponentUpdate(
            inputFile as string,
            options as ComponentUpdateOptions,
          );
        }),
    )
    .command(
      "delete",
      createSubCommand()
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
          await callComponentDelete(
            component as string,
            options as ComponentDeleteOptions,
          );
        }),
    )
    .command(
      "search",
      createSubCommand()
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
          await callComponentSearch(
            query as string,
            options as ComponentSearchOptions,
          );
        }),
    );
}

/** Creates a new SubCommand with root path options configured */
function createSubCommand(): Command<GlobalOptions> {
  return new Command();
}

/**
 * Creates a Project instance from the given root path or discovers it from the
 * current directory.
 *
 * This helper function handles the common pattern of creating a Project:
 * - If a root path is provided, uses it directly
 * - If no root is provided, searches for `.siroot` from the current
 *   working directory
 *
 * @param rootResult - Optional RootPath instance specifying the project root
 * @returns A Project instance configured with the resolved root path
 * @throws Error if no `.siroot` file is found when discovering from cwd
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
function createProject(rootResult?: RootPath | RootPathNotFoundError): Project {
  if (rootResult) {
    if (rootResult instanceof RootPath) {
      return rootResult.toProject();
    } else {
      throw rootResult;
    }
  } else {
    return RootPath.findFromCwd().toProject();
  }
}

async function createApiContext(
  apiBaseUrl: string,
  apiToken?: string,
): Promise<ApiContext> {
  if (!apiToken) {
    throw new ValidationError(
      'Missing required API token; use "--api-token" option or ' +
        '"SI_API_TOKEN" environment variable',
    );
  }

  return await apiContext(apiBaseUrl, apiToken);
}
