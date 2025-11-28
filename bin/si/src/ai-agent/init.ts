/**
 * AI Agent Init Command - Initializes the AI agent configuration
 *
 * This command sets up the AI agent by:
 * - Checking for required dependencies (docker, claude)
 * - Prompting for and validating the API token
 * - Creating the MCP configuration file
 * - Creating Claude settings
 * - Saving the configuration
 *
 * @module
 */

import { Input, Secret } from "@cliffy/prompt";
import type { Context } from "../context.ts";
import {
  createClaudeMd,
  createClaudeSettings,
  createMcpConfig,
  DEFAULT_CONFIG,
  getConfigPath,
  loadConfig,
  saveConfig,
  validateToken,
} from "./shared.ts";

import type { AiTool } from "./shared.ts";

export interface AiAgentInitOptions {
  targetDir?: string;
  apiToken?: string;
  tool?: AiTool;
}

/**
 * Initialize the AI agent configuration
 */
export async function callAiAgentInit(
  ctx: Context,
  options: AiAgentInitOptions = {},
): Promise<void> {
  const logger = ctx.logger;

  logger.info("Initializing SI AI Agent configuration...\n");

  // Get API token
  let apiToken = options.apiToken;

  // Check if there's an existing config
  const existingConfig = await loadConfig();
  if (existingConfig && !apiToken) {
    logger.info("\n🔑 Found existing SI API token in configuration");
    const useExisting = await Input.prompt({
      message: "Use existing token? (y/n)",
      default: "y",
    });

    if (useExisting.toLowerCase() === "y" || useExisting.toLowerCase() === "yes") {
      apiToken = existingConfig.apiToken;
      logger.info("✅ Using existing API token\n");
    }
  }

  // Prompt for token if not provided and not reusing existing
  if (!apiToken) {
    logger.info("\n🔑 System Initiative API Token Required");
    logger.info("To get your API token:");
    logger.info("1. Go to: https://auth.systeminit.com/workspaces");
    logger.info("2. Click the 'gear' icon for your workspace");
    logger.info("3. Select 'API Tokens'");
    logger.info("4. Name it 'claude code' or 'ai-agent'");
    logger.info("5. Generate a new token with 1y expiration");
    logger.info("6. Copy the token from the UI\n");

    while (!apiToken) {
      const token = await Secret.prompt({
        message: "Please paste your API token:",
      });

      if (!token) {
        logger.error("❌ Token cannot be empty");
        continue;
      }

      if (!validateToken(token)) {
        logger.error(
          "❌ Invalid token format. System Initiative tokens are JWTs (three base64 parts separated by dots)",
        );
        continue;
      }

      apiToken = token;
      logger.info("✅ API token validated\n");
    }
  } else if (!validateToken(apiToken)) {
    throw new Error(
      "Invalid token format. System Initiative tokens are JWTs",
    );
  }

  // Determine tool
  const tool = options.tool || existingConfig?.tool || DEFAULT_CONFIG.tool;

  // Create configuration
  const config = {
    apiToken,
    tool,
  };

  // Save configuration
  logger.info("💾 Saving configuration...");
  await saveConfig(config);
  logger.info(`✅ Configuration saved to: ${getConfigPath()}\n`);

  // Determine target directory for MCP and Claude config files
  const targetDir = options.targetDir || Deno.cwd();

  // Create MCP configuration
  logger.info("📄 Creating MCP configuration file...");
  const mcpPath = await createMcpConfig(apiToken, targetDir);
  logger.info(`✅ Created MCP configuration: ${mcpPath}\n`);

  // Create Claude settings only if using Claude
  if (tool === "claude") {
    logger.info("📄 Creating Claude settings configuration...");
    const settingsPath = await createClaudeSettings(targetDir);
    logger.info(`✅ Created Claude settings: ${settingsPath}\n`);

    logger.info("📄 Creating CLAUDE.md context file...");
    const claudeMdPath = await createClaudeMd(targetDir);
    logger.info(`✅ Created CLAUDE.md: ${claudeMdPath}\n`);
  }

  // Success message
  logger.info("🎉 AI Agent initialization complete!");
  logger.info(`\nTool: ${tool}`);
  logger.info("\nNext steps:");
  logger.info("  1. Start the AI agent: si ai-agent start");
  logger.info("     This will start the MCP server and launch your AI tool");
  logger.info("  2. Check status: si ai-agent status");
}
