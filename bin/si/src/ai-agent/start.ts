/**
 * AI Agent Start Command - Launches Claude Code
 *
 * This command simply launches Claude Code, which will automatically:
 * - Read .mcp.json in the current directory
 * - Spawn "si mcp-server stdio" (bundled MCP server)
 * - Manage the MCP server lifecycle
 *
 * When you exit Claude, the MCP server stops automatically.
 *
 * @module
 */

import type { Context } from "../context.ts";
import { loadConfig } from "./shared.ts";

export interface AiAgentStartOptions {
  tool?: string;
}

/**
 * Start the AI agent by launching the configured tool
 */
export async function callAiAgentStart(
  ctx: Context,
  options: AiAgentStartOptions = {},
): Promise<void> {
  const logger = ctx.logger;

  logger.info("Starting SI AI Agent...\n");

  // Load configuration
  const config = await loadConfig();
  if (!config) {
    logger.error("❌ No configuration found!");
    logger.info("Please run: si ai-agent init");
    Deno.exit(1);
  }

  const tool = options.tool || config.tool || "claude";

  // Validate tool
  const validTools = ["claude", "codex", "opencode", "cursor", "windsurf"];
  if (!validTools.includes(tool)) {
    logger.error(
      `Unknown tool: ${tool}. Valid tools are: ${validTools.join(", ")}`,
    );
    Deno.exit(1);
  }

  // Get the CLI command for the tool
  const toolCommand = getToolCommand(tool);
  const toolAvailable = await checkToolAvailable(toolCommand);

  if (!toolAvailable) {
    logger.error(`❌ ${getToolDisplayName(tool)} CLI is not installed!`);
    logger.info(`\nTo use ${getToolDisplayName(tool)} with the SI MCP server:`);

    if (tool === "cursor") {
      logger.info("  1. Install the Cursor CLI:");
      logger.info("     See: https://cursor.com/docs/cli/overview");
      logger.info(
        "  2. Or launch Cursor manually and it will use the .cursor/mcp.json configuration",
      );
      if (Deno.build.os === "darwin") {
        logger.info('     macOS: open -a "Cursor"');
      } else if (Deno.build.os === "windows") {
        logger.info("     Windows: Start Cursor from the Start menu");
      } else {
        logger.info("     Linux: Launch Cursor from your applications menu");
      }
    } else {
      logger.info(`  Install from: ${getToolInstallUrl(tool)}`);
    }

    Deno.exit(1);
  }

  ctx.analytics.trackEvent("ai-agent start", {
    tool,
  });

  // Launch the tool - it will manage the MCP server via config file
  logger.info(`🚀 Launching ${getToolDisplayName(tool)}...`);

  // Tool-specific configuration file info
  const configInfo = getToolConfigInfo(tool);
  logger.info(`The MCP server will start automatically via ${configInfo}\n`);

  try {
    const toolProcess = new Deno.Command(toolCommand, {
      stdin: "inherit",
      stdout: "inherit",
      stderr: "inherit",
    }).spawn();

    // Wait for tool to exit
    const status = await toolProcess.status;

    logger.info(`\n${getToolDisplayName(tool)} exited.`);

    if (!status.success) {
      Deno.exit(status.code);
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    logger.error(`Failed to launch ${getToolDisplayName(tool)}: ${errorMsg}`);
    throw error;
  }
}

/**
 * Get the CLI command for a tool
 */
function getToolCommand(tool: string): string {
  const commands: Record<string, string> = {
    claude: "claude",
    codex: "codex",
    opencode: "opencode",
    cursor: "cursor-agent",
    windsurf: "windsurf",
  };
  return commands[tool];
}

/**
 * Check if a tool command is available
 */
async function checkToolAvailable(command: string): Promise<boolean> {
  try {
    const cmd = new Deno.Command(command, {
      args: ["--version"],
      stdout: "null",
      stderr: "null",
    });
    const { success } = await cmd.output();
    return success;
  } catch {
    return false;
  }
}

/**
 * Get display name for a tool
 */
function getToolDisplayName(tool: string): string {
  const names: Record<string, string> = {
    claude: "Claude Code",
    codex: "OpenAI Codex",
    opencode: "OpenCode.ai",
    cursor: "Cursor",
    windsurf: "Windsurf",
  };
  return names[tool];
}

/**
 * Get installation URL for a tool
 */
function getToolInstallUrl(tool: string): string {
  const urls: Record<string, string> = {
    claude: "https://www.anthropic.com/claude-code",
    codex: "https://developers.openai.com/codex/cli/",
    opencode: "https://opencode.ai/",
    cursor: "https://cursor.sh/",
    windsurf: "https://codeium.com/windsurf",
  };
  return urls[tool];
}

/**
 * Get configuration file info for a tool
 */
function getToolConfigInfo(tool: string): string {
  const configInfo: Record<string, string> = {
    claude: ".mcp.json",
    codex: "~/.codex/config.toml",
    opencode: "opencode.jsonc",
    cursor: ".cursor/mcp.json",
    windsurf: ".mcp.json",
  };
  return configInfo[tool];
}
