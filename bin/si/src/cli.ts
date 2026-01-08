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
import { callWhoami } from "./whoami.ts";
import { callProjectInit } from "./schema/init.ts";
import { callRemoteSchemaPull } from "./schema/pull.ts";
import {
  callSchemaContribute,
  type SchemaContributeOptions,
} from "./schema/contribute.ts";
import {
  callSchemaFuncGenerate,
  callSchemaScaffoldGenerate,
} from "./schema/generate.ts";
import { unknownValueToErrorMessage } from "./helpers.ts";
import { Context } from "./context.ts";
import { FunctionKind, Project } from "./schema/project.ts";
import { VERSION } from "./git_metadata.ts";
import {
  callRemoteSchemaOverlaysPush,
  callRemoteSchemaPush,
} from "./schema/push.ts";
import { callRunTemplate } from "./template/run.ts";
import {
  callGenerateTemplate,
  type GenerateTemplateOptions,
} from "./template/generate.ts";
import { callComponentGet } from "./component/get.ts";
import { callComponentUpdate } from "./component/update.ts";
import { callComponentDelete } from "./component/delete.ts";
import { callComponentSearch } from "./component/search.ts";
import { callComponentUpgrade } from "./component/upgrade.ts";
import type { TemplateContextOptions } from "./template/run.ts";
import type { ComponentGetOptions } from "./component/get.ts";
import type { ComponentUpdateOptions } from "./component/update.ts";
import type { ComponentDeleteOptions } from "./component/delete.ts";
import type { ComponentSearchOptions } from "./component/search.ts";
import type { ComponentUpgradeOptions } from "./component/upgrade.ts";
import { type AiAgentInitOptions, callAiAgentInit } from "./ai-agent/init.ts";
import {
  type AiAgentStartOptions,
  callAiAgentStart,
} from "./ai-agent/start.ts";
import {
  type AiAgentConfigOptions,
  callAiAgentConfig,
} from "./ai-agent/config.ts";
import { callSecretCreate, type SecretCreateOptions } from "./secret/create.ts";
import { callSecretUpdate, type SecretUpdateOptions } from "./secret/update.ts";
import {
  callChangeSetCreate,
  type ChangeSetCreateOptions,
} from "./change-set/create.ts";
import {
  callChangeSetAbandon,
  type ChangeSetAbandonOptions,
} from "./change-set/abandon.ts";
import {
  callChangeSetApply,
  type ChangeSetApplyOptions,
} from "./change-set/apply.ts";
import {
  callChangeSetList,
  type ChangeSetListOptions,
} from "./change-set/list.ts";
import {
  callChangeSetOpen,
  type ChangeSetOpenOptions,
} from "./change-set/open.ts";
import { doLogin } from "./cli/login.ts";
import {
  getCurrentUser,
  getCurrentWorkspace,
  getUserDetails,
  getWorkspaceDetails,
  logout,
  setCurrentWorkspace,
  writeWorkspace,
} from "./cli/config.ts";
import { AuthApiClient, isTokenAboutToExpire } from "./cli/auth.ts";
import {
  callComponentCreate,
  type ComponentCreateOptions,
} from "./component/create.ts";
import {
  callComponentErase,
  type ComponentEraseOptions,
} from "./component/erase.ts";
import {
  callWorkspaceCreate,
  type WorkspaceCreateOptions,
} from "./workspace/create.ts";
import {
  callWorkspaceLeave,
  type WorkspaceLeaveOptions,
} from "./workspace/leave.ts";
import {
  callWorkspaceDelete,
  type WorkspaceDeleteOptions,
} from "./workspace/delete.ts";
import {
  callWorkspaceInvite,
  type WorkspaceInviteOptions,
} from "./workspace/manage-members.ts";
import {
  callWorkspaceMemberList,
  type WorkspaceMemberListOptions,
} from "./workspace/member-list.ts";
import {
  callPolicyEvaluate,
  type PolicyEvaluateOptions,
} from "./policy/evaluate.ts";

/**
 * Global options available to all commands
 */
export type GlobalOptions = {
  baseUrl?: string;
  apiToken?: string;
  authApiUrl: string;
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
  return (
    new Command()
      .name("si")
      .version(VERSION)
      .description(
        "A command-line tool for managing System Initiative schemas, templates, and components",
      )
      .globalType("root-path", new RootPathType())
      .globalEnv("SI_AUTH_API_URL=<URL:string>", "Auth API endpoint URL", {
        prefix: "SI_",
      })
      .globalOption("--auth-api-url <URL:string>", "Auth API endpoint URL", {
        default: "https://auth-api.systeminit.com",
      })
      .globalOption("--base-url <URL:string>", "API endpoint URL")
      .globalEnv("SI_BASE_URL=<URL:string>", "API endpoint URL", {
        prefix: "SI_",
      })
      .globalEnv(
        "SI_API_TOKEN=<TOKEN:string>",
        "Your System Initiative Workspace API token (required for authenticated commands)",
        { prefix: "SI_" },
      )
      .globalOption(
        "--api-token <TOKEN:string>",
        "Your System Initiative API token",
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
        // Reads stored config for api setup, or sets up API via the environment
        // or command line overrides
        await Context.initFromConfig(options);
      })
      .action(function () {
        this.showHelp();
      })
      .command("completion", new CompletionsCommand())
      // deno-lint-ignore no-explicit-any
      .command("ai-agent", buildAiAgentCommand() as any)
      // deno-lint-ignore no-explicit-any
      .command("change-set", buildChangeSetCommand() as any)
      // deno-lint-ignore no-explicit-any
      .command("component", buildComponentCommand() as any)
      // deno-lint-ignore no-explicit-any
      .command("schema", buildSchemaCommand() as any)
      // deno-lint-ignore no-explicit-any
      .command("secret", buildSecretCommand() as any)
      // deno-lint-ignore no-explicit-any
      .command("template", buildTemplateCommand() as any)
      // deno-lint-ignore no-explicit-any
      .command("whoami", buildWhoamiCommand() as any)
      // deno-lint-ignore no-explicit-any
      .command("login", buildLoginCommand() as any)
      // deno-lint-ignore no-explicit-any
      .command("logout", buildLogoutCommand() as any)
      // deno-lint-ignore no-explicit-any
      .command("workspace", buildWorkspaceCommand() as any)
      // deno-lint-ignore no-explicit-any
      .command("policy", buildPolicyCommand() as any)
  );
}

/**
 * Builds the schema command group with all subcommands.
 *
 * @returns A SubCommand configured for schema operations
 * @internal
 */
function buildSchemaCommand() {
  return createSubCommand()
    .description(
      "Manage schemas: initialize project, generate functions locally, pull from and push to remote workspaces",
    )
    .action(function () {
      this.showHelp();
    })
    .command("init", buildInitCommand())
    .command("action", buildSchemaActionCommand())
    .command("authentication", buildSchemaAuthenticationCommand())
    .command("codegen", buildSchemaCodegenCommand())
    .command("management", buildSchemaManagementCommand())
    .command("qualification", buildSchemaQualificationCommand())
    .command("scaffold", buildSchemaScaffoldCommand())
    .command("overlay", buildOverlayCommand())
    .command(
      "pull",
      createSubCommand(true)
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
        .action(async ({ root, builtins }, ...schemaNames) => {
          const project = createProject(root);
          let finalSchemaNames;
          if (schemaNames.length > 0) {
            finalSchemaNames = schemaNames;
          } else {
            finalSchemaNames = [await prompt.schemaName(undefined, project)];
          }

          await callRemoteSchemaPull(
            Context.instance(),
            project,
            finalSchemaNames,
            builtins ?? false,
          );
        }),
    )
    .command(
      "push",
      createSubCommand()
        .description(
          "Pushes schemas to your remote System Initiative workspace",
        )
        .option("-s, --skip-confirmation", "Skip confirmation prompt")
        .option(
          "-b, --update-builtins",
          "Change builtin schema, without creating overlays.",
          {
            hidden: false,
          },
        )
        .arguments("[...SCHEMA_NAME:string]")
        .action(
          async (
            { root, skipConfirmation, updateBuiltins },
            ...schemaNames
          ) => {
            const project = createProject(root);

            await callRemoteSchemaPush(
              project,
              schemaNames,
              !!updateBuiltins,
              skipConfirmation,
            );
          },
        ),
    )
    .command(
      "contribute",
      createSubCommand(true)
        .description(
          "Contribute a schema to the module index (works on HEAD change set only).",
        )
        .arguments("<SCHEMA:string>")
        .action(async (_, schema) => {
          await callSchemaContribute({
            schema: schema as string,
          } as SchemaContributeOptions);
        }),
    );
}

function buildOverlayCommand() {
  return createSubCommand()
    .description(
      "Manage schema overlays: generate overlay functions and push them to remote workspaces",
    )
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
    )
    .command(
      "push",
      createSubCommand(true)
        .description(
          "Pushes overlay funcs to your remote System Initiative workspace",
        )
        .option("-s, --skip-confirmation", "Skip confirmation prompt")
        .action(async ({ root, skipConfirmation }) => {
          const project = createProject(root);
          await callRemoteSchemaOverlaysPush(project, skipConfirmation);
        }),
    );
}

/**
 * Builds the whoami command.
 *
 * @returns A SubCommand configured for displaying user information
 * @internal
 */
function buildWhoamiCommand() {
  return createSubCommand(true)
    .description("Displays authenticated user information")
    .action(async () => {
      await callWhoami();
    });
}

/**
 * Builds the login command for handling the OAuth flow.
 *
 * @returns A SubCommand configured for login operations
 * @internal
 */
function buildLoginCommand() {
  return createSubCommand()
    .description("Login to System Initiiatve")
    .action(async ({ authApiUrl }) => {
      await doLogin(authApiUrl);
    });
}

/**
 * Builds the logout command for clearing stored authentication.
 *
 * @returns A SubCommand configured for logout operations
 * @internal
 */
function buildLogoutCommand() {
  return createSubCommand()
    .description("Logout from System Initiative")
    .option(
      "--clear",
      "Also delete stored tokens for the current user from disk",
    )
    .action(({ clear }) => {
      const ctx = Context.instance();

      // Check if user is logged in
      const currentUserId = getCurrentUser();
      if (!currentUserId) {
        ctx.logger.info("Not currently logged in.");
        return;
      }

      // Get user details for confirmation message
      const { userDetails } = getUserDetails(currentUserId);
      const userEmail = userDetails?.email || "unknown user";

      // Clear stored authentication
      logout(clear ?? false);

      ctx.logger.info(`Logged out successfully. Goodbye ${userEmail}!`);
    });
}

/**
 * Builds the workspace command for managing workspaces.
 *
 * @returns A SubCommand configured for workspace management
 * @internal
 */
function buildWorkspaceCommand() {
  return createSubCommand()
    .description("Manage workspaces you have access to")
    .action(function () {
      this.showHelp();
      const currentUserId = getCurrentUser();
      const currentWorkspaceId = getCurrentWorkspace();
      if (currentWorkspaceId && currentUserId) {
        const { workspaceDetails } = getWorkspaceDetails(
          currentUserId,
          currentWorkspaceId,
        );
        if (workspaceDetails) {
          const ctx = Context.instance();
          ctx.logger.info(`Current user: ${currentUserId}`);
          ctx.logger.info(
            `Current workspace: ${
              workspaceDetails.displayName || currentWorkspaceId
            }`,
          );
          ctx.logger.info(
            `Workspace Instance URL: ${workspaceDetails.instanceUrl}`,
          );
        }
      }
    })
    .command("switch", buildWorkspaceSwitchCommand())
    .command("create", buildWorkspaceCreateCommand())
    .command("members", buildWorkspaceMembersCommand())
    .command("leave", buildWorkspaceLeaveCommand())
    .command("delete", buildWorkspaceDeleteCommand());
}

/**
 * Builds the switch-workspace command for changing the active workspace.
 *
 * @returns A SubCommand configured for workspace switching
 * @internal
 */
function buildWorkspaceSwitchCommand() {
  return createSubCommand()
    .description("Switch to a different workspace")
    .arguments("[workspace:string]")
    .action(async ({ authApiUrl }, workspaceArg) => {
      const ctx = Context.instance();

      // Check if user is logged in
      const currentUserId = getCurrentUser();
      if (!currentUserId) {
        ctx.logger.error(
          "Not logged in. Please run 'si login' to authenticate first.",
        );
        return;
      }

      // Get user details and token
      const { userDetails, token } = getUserDetails(currentUserId);
      if (!userDetails || !token) {
        ctx.logger.error(
          "User configuration corrupted. Please run 'si login' again.",
        );
        return;
      }

      ctx.logger.info(`Switching workspace for ${userDetails.email}...`);

      try {
        // Fetch available workspaces
        const authApiClient = new AuthApiClient(authApiUrl, token);
        const workspaces = await authApiClient.getWorkspaces();

        if (workspaces.length === 0) {
          ctx.logger.error("No workspaces available for this user.");
          return;
        }

        const currentWorkspaceId = getCurrentWorkspace();
        if (currentWorkspaceId) {
          const currentWorkspace = workspaces.find(
            (w) => w.id === currentWorkspaceId,
          );
          if (currentWorkspace) {
            ctx.logger.info(
              `Current workspace: ${
                currentWorkspace.displayName || currentWorkspace.id
              }`,
            );
          }
        }

        // Prompt for new workspace selection (or use provided argument)
        const selectedWorkspaceId = await prompt.workspace(
          workspaces,
          workspaceArg,
        );

        // Check if it's the same workspace
        if (selectedWorkspaceId === currentWorkspaceId) {
          ctx.logger.info("Already using this workspace.");
          return;
        }

        const selectedWorkspace = workspaces.find(
          (w) => w.id === selectedWorkspaceId,
        );
        if (!selectedWorkspace) {
          throw new Error(`Workspace not found: ${selectedWorkspaceId}`);
        }

        // Check if we already have a token for this workspace
        const { token: existingToken } = getWorkspaceDetails(
          currentUserId,
          selectedWorkspaceId,
        );

        const isAboutToExpire = existingToken &&
          isTokenAboutToExpire(existingToken);

        if (!existingToken || isAboutToExpire) {
          ctx.logger.info(
            `Generating workspace access token for ${selectedWorkspaceId}`,
          );
          const workspaceToken = await authApiClient.createWorkspaceToken(
            selectedWorkspaceId,
          );
          writeWorkspace(currentUserId, selectedWorkspace, workspaceToken);
        } else {
          ctx.logger.info(
            `Reusing existing workspace token for ${selectedWorkspaceId}`,
          );
        }

        // Update current workspace
        setCurrentWorkspace(selectedWorkspaceId);

        const workspaceName = selectedWorkspace.displayName ||
          selectedWorkspaceId;
        ctx.logger.info(`Switched to workspace: ${workspaceName}`);

        ctx.analytics.trackEvent("workspace switch", {
          workspaceName,
        });
      } catch (error) {
        ctx.logger.error(`Failed to switch workspace: ${error}`);
        throw error;
      }
    });
}

/**
 * Builds the create-workspace command for creating a new workspace.
 *
 * @returns A SubCommand configured for workspace creation
 * @internal
 */
function buildWorkspaceCreateCommand() {
  return createSubCommand(true)
    .description("Create a new workspace")
    .arguments("<name:string>")
    .option("--description <description:string>", "Workspace description")
    .option(
      "--instance-url <url:string>",
      "Instance URL (defaults to https://app.systeminit.com)",
    )
    .action(async (options, name) => {
      await callWorkspaceCreate({
        ...options,
        name: name as string,
      } as WorkspaceCreateOptions);
    });
}

/**
 * Builds the workspace members command group with subcommands.
 *
 * @returns A SubCommand configured for workspace member operations
 * @internal
 */
function buildWorkspaceMembersCommand() {
  return createSubCommand(true)
    .description("Manage workspace members")
    .command("list", buildWorkspaceMembersListCommand())
    .command("manage", buildWorkspaceMembersManageCommand());
}

/**
 * Builds the members list subcommand.
 *
 * @returns A SubCommand configured for listing workspace members
 * @internal
 */
function buildWorkspaceMembersListCommand() {
  return createSubCommand(true)
    .description("List all members of the current workspace")
    .action(async (options) => {
      await callWorkspaceMemberList({
        ...options,
      } as WorkspaceMemberListOptions);
    });
}

/**
 * Builds the members manage subcommand.
 *
 * @returns A SubCommand configured for inviting/updating workspace members
 * @internal
 */
function buildWorkspaceMembersManageCommand() {
  return createSubCommand(true)
    .description(
      "Invite or update workspace members (collaborators by default, or approvers with --approvers)",
    )
    .option(
      "--approvers <emails:string>",
      "Comma-separated list of emails to invite/update as approvers",
    )
    .arguments("[email:string]")
    .action(async (options, email) => {
      await callWorkspaceInvite({
        ...options,
        email: email as string | undefined,
      } as WorkspaceInviteOptions);
    });
}

/**
 * Builds the leave-workspace command for leaving a workspace.
 *
 * @returns A SubCommand configured for workspace leaving
 * @internal
 */
function buildWorkspaceLeaveCommand() {
  return createSubCommand(true)
    .description("Leave a workspace")
    .arguments("<workspace:string>")
    .action(async (options, workspace) => {
      await callWorkspaceLeave({
        ...options,
        workspace: workspace as string,
      } as WorkspaceLeaveOptions);
    });
}

/**
 * Builds the delete-workspace command for deleting a workspace.
 *
 * @returns A SubCommand configured for workspace deletion
 * @internal
 */
function buildWorkspaceDeleteCommand() {
  return createSubCommand(true)
    .description("Delete a workspace (soft delete - can be recovered)")
    .arguments("<workspace:string>")
    .action(async (options, workspace) => {
      await callWorkspaceDelete({
        ...options,
        workspace: workspace as string,
      } as WorkspaceDeleteOptions);
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
          "--tool <name:string>",
          "AI tool to use: claude (default), codex, opencode",
        )
        .action(async (options) => {
          await callAiAgentInit(
            Context.instance(),
            options as AiAgentInitOptions,
          );
        }),
    )
    .command(
      "start",
      createSubCommand()
        .description("Launch Claude Code (MCP server starts automatically)")
        .option("--tool <name:string>", "AI tool to launch (default: claude)")
        .action(async (options) => {
          await callAiAgentStart(
            Context.instance(),
            options as AiAgentStartOptions,
          );
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
        .option("--update-token", "Update the API token")
        .option(
          "--tool <name:string>",
          "Update the AI tool: claude, cursor, windsurf, or none",
        )
        .action(async (options) => {
          await callAiAgentConfig(
            Context.instance(),
            options as AiAgentConfigOptions,
          );
        }),
    )
    .command(
      "stdio",
      createSubCommand(true)
        .description(
          "Run MCP server in stdio mode (for external AI tools to connect)",
        )
        .action(async () => {
          // Dynamic import to avoid loading MCP server code until needed
          const { start_stdio } = await import(
            "./ai-agent/mcp-server/stdio_transport.ts"
          );
          const { createServer } = await import(
            "./ai-agent/mcp-server/server.ts"
          );
          const { analytics } = await import(
            "./ai-agent/mcp-server/analytics.ts"
          );
          const { setAiAgentUserFlag } = await import(
            "./ai-agent/mcp-server/user_state.ts"
          );

          // Start the MCP server directly
          analytics.trackServerStart();
          await setAiAgentUserFlag();

          const server = createServer();

          let ended = false;
          const shutdown = async (
            reason: string,
            exitCode: number | null = 0,
          ) => {
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
    .description(`Action${overlayMsg} function operations`)
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(`Generate action${overlayMsg} functions for schemas`)
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
    .description(`Authentication${overlayMsg} function operations`)
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(
          `Generate authentication${overlayMsg} functions for schemas`,
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
    .description(`Code generator${overlayMsg} function operations`)
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(
          `Generate code generator${overlayMsg} functions for schemas`,
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
    .description(`Management${overlayMsg} function operations`)
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(`Generate management${overlayMsg} functions for schemas`)
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
    .description(`Qualification${overlayMsg} function operations`)
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(
          `Generate qualification${overlayMsg} functions for schemas`,
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
 * Builds the schema scaffold command.
 *
 * @returns A SubCommand configured for scaffold operations
 * @internal
 */
function buildSchemaScaffoldCommand() {
  return createSubCommand()
    .description("Schema scaffold operations")
    .action(function () {
      this.showHelp();
    })
    .command(
      "generate",
      createSubCommand()
        .description(
          "Generate a complete schema scaffold with all default functions",
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
      "generate",
      createSubCommand()
        .description("Generate a new template structure file")
        .arguments("<name:string>")
        .option(
          "-o, --output-dir <path:string>",
          "Output directory for the template file (defaults to current directory)",
        )
        .action(async (options, name) => {
          await callGenerateTemplate(Context.instance(), {
            name: name as string,
            outputDir: options.outputDir as string | undefined,
          } as GenerateTemplateOptions);
        }),
    )
    .command(
      "run",
      createSubCommand(true)
        .description("Run a SI template file (local path or remote URL)")
        .arguments("<template:string>")
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
        .option("--dry-run", "Show planned changes without executing them")
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
      createSubCommand(true)
        .description("Get component data by name or ID")
        .arguments("<component:string>")
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
        .option("--raw", "Output raw API response as JSON and exit")
        .action(async (options, component) => {
          await callComponentGet(
            component as string,
            options as ComponentGetOptions,
          );
        }),
    )
    .command(
      "create",
      createSubCommand(true)
        .description("Create component from JSON/YAML file (idempotent)")
        .arguments("<input-file:string>")
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
        .action(async (options, inputFile) => {
          await callComponentCreate(
            inputFile,
            options as ComponentCreateOptions,
          );
        }),
    )
    .command(
      "update",
      createSubCommand(true)
        .description("Update a component from JSON/YAML file (idempotent)")
        .arguments("<input-file:string>")
        .option(
          "--component <id-or-name:string>",
          "Component ID or name (overrides componentId from file)",
        )
        .option(
          "-c, --change-set <id-or-name:string>",
          "Change set ID or name",
          { required: true },
        )
        .option("--dry-run", "Show diff without applying changes")
        .action(async (options, inputFile) => {
          await callComponentUpdate(
            inputFile as string,
            options as ComponentUpdateOptions,
          );
        }),
    )
    .command(
      "delete",
      createSubCommand(true)
        .description("Delete a component by name or ID")
        .arguments("<component:string>")
        .option(
          "-c, --change-set <id-or-name:string>",
          "Change set ID or name",
          { required: true },
        )
        .option("--dry-run", "Preview deletion without applying changes")
        .action(async (options, component) => {
          await callComponentDelete(
            component as string,
            options as ComponentDeleteOptions,
          );
        }),
    )
    .command(
      "erase",
      createSubCommand(true)
        .description(
          "Erase a component by name or ID",
        )
        .arguments("<component:string>")
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
          await callComponentErase(
            component as string,
            options as ComponentEraseOptions,
          );
        }),
    )
    .command(
      "search",
      createSubCommand(true)
        .description("Search for components using a search query")
        .arguments("<query:string>")
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
    )
    .command(
      "upgrade",
      createSubCommand(true)
        .description("Upgrade component(s) to latest schema version")
        .arguments("[component:string]")
        .option(
          "-c, --change-set <id-or-name:string>",
          "Change set ID or name (creates new change set if not specified)",
        )
        .option(
          "--all",
          "Upgrade all upgradable components (required if no component specified)",
        )
        .option(
          "--schema-category <category:string>",
          "Filter by schema category (e.g., AWS::EC2) when using --all",
        )
        .option(
          "--dry-run",
          "Preview upgrades without applying changes",
        )
        .action(async (options, component) => {
          await callComponentUpgrade(
            component as string | undefined,
            options as ComponentUpgradeOptions,
          );
        }),
    );
}

/**
 * Builds the secret command group with all subcommands.
 *
 * @returns A SubCommand configured for secret operations
 * @internal
 */
function buildSecretCommand() {
  return createSubCommand()
    .description("Manage secrets and credentials")
    .action(function () {
      this.showHelp();
    })
    .command(
      "create",
      createSubCommand(true)
        .description("Create a new secret")
        .arguments("<secret-type:string>")
        .option("--name <name:string>", "Name for the secret instance")
        .option("--description <desc:string>", "Description for the secret")
        .option(
          "-c, --change-set <id-or-name:string>",
          "Change set ID or name (creates new change set if not specified)",
        )
        .option(
          "--use-local-profile",
          "Discover credentials from local environment (e.g., AWS credentials)",
        )
        .option("--interactive", "Prompt for all values interactively")
        .option(
          "--dry-run",
          "Show what would be created without making changes",
        )
        .action(async (options, secretType) => {
          // Parse --field-* options from remaining args
          const fields: Record<string, string> = {};

          // Extract field options (--field-name value pattern)
          // Note: Cliffy doesn't support dynamic options, so we'll handle this in the action

          await callSecretCreate({
            ...options,
            secretType: secretType as string,
            fields,
          } as SecretCreateOptions);
        }),
    )
    .command(
      "update",
      createSubCommand(true)
        .description("Update an existing secret by component name or ID")
        .arguments("<component-name-or-id:string>")
        .option("--name <name:string>", "New name for the secret")
        .option("--description <desc:string>", "New description for the secret")
        .option(
          "-c, --change-set <id-or-name:string>",
          "Change set ID or name (creates new change set if not specified)",
        )
        .option(
          "--use-local-profile",
          "Discover credentials from local environment (e.g., AWS credentials)",
        )
        .option("--interactive", "Prompt for all values interactively")
        .option(
          "--dry-run",
          "Show what would be updated without making changes",
        )
        .action(async (options, componentNameOrId) => {
          // Parse --field-* options from remaining args
          const fields: Record<string, string> = {};

          // Extract field options (--field-name value pattern)
          // Note: Cliffy doesn't support dynamic options, so we'll handle this in the action

          await callSecretUpdate({
            ...options,
            componentNameOrId: componentNameOrId as string,
            fields,
          } as SecretUpdateOptions);
        }),
    );
}

/**
 * Builds the change-set command group with all subcommands.
 *
 * @returns A SubCommand configured for change-set operations
 * @internal
 */
function buildChangeSetCommand() {
  return createSubCommand()
    .description("Manage change sets")
    .action(function () {
      this.showHelp();
    })
    .command(
      "create",
      createSubCommand(true)
        .description("Create a new change set")
        .arguments("<name:string>")
        .option("--open", "Open the change set in the browser after creation")
        .action(async (options, name) => {
          await callChangeSetCreate({
            ...options,
            name: name as string,
          } as ChangeSetCreateOptions);
        }),
    )
    .command(
      "abandon",
      createSubCommand(true)
        .description("Abandon (delete) a change set")
        .arguments("<change-set-id-or-name:string>")
        .action(async (options, changeSetIdOrName) => {
          await callChangeSetAbandon({
            ...options,
            changeSetIdOrName: changeSetIdOrName as string,
          } as ChangeSetAbandonOptions);
        }),
    )
    .command(
      "open",
      createSubCommand(true)
        .description("Open a change set in the browser")
        .arguments("[change-set-id-or-name:string]")
        .action(async (options, changeSetIdOrName) => {
          await callChangeSetOpen({
            ...options,
            changeSetIdOrName: (changeSetIdOrName as string) || "HEAD",
          } as ChangeSetOpenOptions);
        }),
    )
    .command(
      "apply",
      createSubCommand(true)
        .description("Apply a change set to HEAD")
        .arguments("<change-set-id-or-name:string>")
        .option(
          "-d, --detach",
          "Don't wait for actions to complete, return immediately after applying",
        )
        .action(async (options, changeSetIdOrName) => {
          await callChangeSetApply({
            ...options,
            changeSetIdOrName: changeSetIdOrName as string,
          } as ChangeSetApplyOptions);
        }),
    )
    .command(
      "list",
      createSubCommand(true)
        .description("List all change sets")
        .option(
          "-o, --output <format:string>",
          "Output format: info (default), json, or yaml",
          { default: "info" },
        )
        .action(async (options) => {
          await callChangeSetList(options as ChangeSetListOptions);
        }),
    );
}

/**
 * Builds the policy command group with all subcommands.
 *
 * @returns A SubCommand configured for policy operations
 * @internal
 */
function buildPolicyCommand() {
  return createSubCommand()
    .description("Policy management operations")
    .action(function () {
      this.showHelp();
    })
    .command(
      "evaluate",
      createSubCommand(true)
        .description("Evaluate policies against infrastructure components")
        .arguments("<file-path:string>")
        .option(
          "-n, --name <name:string>",
          "Name for the policy evaluation (required unless --all is used)",
        )
        .option(
          "--all",
          "Evaluate all policy files in a directory (only works with directories)",
        )
        .option(
          "-c, --change-set <id:string>",
          "Change set ID or name (defaults to HEAD)",
        )
        .option(
          "-o, --output-folder <name:string>",
          "Folder name to organize results (defaults to timestamp)",
        )
        .option(
          "--no-upload",
          "Skip uploading the policy evaluation results",
        )
        .action(async (options, filePath) => {
          // deno-lint-ignore si-rules/no-deno-env-get
          const claudeAPIKey = Deno.env.get("ANTHROPIC_API_KEY");
          if (!claudeAPIKey) {
            throw new Error(
              "Your Anthropic API Key needs to be set to the `ANTHROPIC_API_KEY` environment variable, or in a `.env` file.",
            );
          }

          await callPolicyEvaluate(
            filePath as string,
            options as PolicyEvaluateOptions,
          );
        }),
    );
}

async function ensureApiConfig(options: GlobalOptions): Promise<void> {
  const ctx = Context.instance();

  const authApiToken = ctx.authApiToken;
  if (authApiToken && isTokenAboutToExpire(authApiToken)) {
    if (ctx.isInteractive) {
      await doLogin(options.authApiUrl);
      await Context.initFromConfig(options);
      return;
    } else {
      ctx.logger.warn(
        "Your auth token is about to expire or has expired, please run `si login` in an interactive terminal to refresh it",
      );
    }
  }

  const isApiTokenAboutToExpire = ctx.apiToken &&
    isTokenAboutToExpire(ctx.apiToken);

  if (ctx.apiToken && !isApiTokenAboutToExpire) {
    ctx.logger.debug("API configured! Good job");
    return;
  }

  if (options.apiToken && isApiTokenAboutToExpire) {
    ctx.logger.error(
      "The api token you set in your environment, or via a command line argument, has expired.",
    );
    Deno.exit(78);
  } else if (
    ctx.apiToken &&
    isApiTokenAboutToExpire &&
    authApiToken &&
    ctx.isInteractive
  ) {
    const currentUserId = getCurrentUser();
    const currentWorkspaceId = getCurrentWorkspace();
    if (currentWorkspaceId && currentUserId) {
      const { workspaceDetails } = getWorkspaceDetails(
        currentUserId,
        currentWorkspaceId,
      );
      if (workspaceDetails) {
        ctx.logger.info("Your workspace token is expired. Creating a new one");
        const authApiClient = new AuthApiClient(
          options.authApiUrl,
          authApiToken,
        );
        const workspaceToken = await authApiClient.createWorkspaceToken(
          currentWorkspaceId,
        );
        writeWorkspace(currentUserId, workspaceDetails, workspaceToken);
        return;
      }
    }
  }

  // If for some reason we cannot regen the automation token (or if none exists),
  // just do the login flow again
  if (ctx.isInteractive) {
    await doLogin(options.authApiUrl);
    await Context.initFromConfig(options);
  } else {
    const msg =
      "No API token configured, or your token is expired/about to expire. Run `si login` in an interactive terminal or set SI_API_KEY and SI_BASE_URL.";
    ctx.logger.error(msg);
    throw new Error(msg);
  }
}

/** Creates a new SubCommand with root path options configured */
function createSubCommand(requireAuth: boolean = false) {
  const command: Command<GlobalOptions> = new Command();

  if (requireAuth) {
    return command.globalAction(async (options) => {
      await ensureApiConfig(options);
    });
  }

  return command;
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
