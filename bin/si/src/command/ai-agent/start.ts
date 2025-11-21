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

import type { Context } from "../../context.ts";
import { loadConfig } from "../../ai_agent.ts";

export interface AiAgentStartOptions {
  tool?: string;
}

/**
 * Execute the ai-agent start command
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
    throw new Error("AI agent not initialized");
  }

  const tool = options.tool || config.tool || "claude";

  // Check if tool is available
  const toolCommand = tool === "claude" ? "claude" : tool;
  const toolAvailable = await checkToolAvailable(toolCommand);

  if (!toolAvailable) {
    logger.error(`❌ ${getToolDisplayName(tool)} is not installed!`);
    logger.info(`\nPlease install ${getToolDisplayName(tool)}:`);
    logger.info(getToolInstallUrl(tool));
    throw new Error(`${getToolDisplayName(tool)} not found`);
  }

  // Launch the tool - it will manage the MCP server via .mcp.json
  logger.info(`🚀 Launching ${getToolDisplayName(tool)}...`);
  logger.info("The MCP server will start automatically via .mcp.json\n");

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
    cursor: "Cursor",
    windsurf: "Windsurf",
  };
  return names[tool] || tool;
}

/**
 * Get installation URL for a tool
 */
function getToolInstallUrl(tool: string): string {
  const urls: Record<string, string> = {
    claude: "https://www.anthropic.com/claude-code",
    cursor: "https://cursor.sh/",
    windsurf: "https://codeium.com/windsurf",
  };
  return urls[tool] || "";
}
