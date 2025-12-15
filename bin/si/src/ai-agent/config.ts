/**
 * AI Agent Config Command - Displays and manages AI agent configuration
 *
 * This command allows users to:
 * - View current configuration (with masked token)
 * - Update API token
 * - Update docker image/tag
 * - Show configuration file location
 *
 * @module
 */

import { Secret } from "@cliffy/prompt";
import type { Context } from "../context.ts";
import type { AiTool } from "./shared.ts";
import {
  getConfigPath,
  loadConfig,
  saveConfig,
  validateToken,
} from "./shared.ts";

export interface AiAgentConfigOptions {
  show?: boolean;
  updateToken?: boolean;
  tool?: string;
}

/**
 * Mask an API token for display (show first 10 and last 4 characters)
 */
function maskToken(token: string): string {
  if (token.length <= 20) {
    return "***";
  }
  return `${token.slice(0, 10)}...${token.slice(-4)}`;
}

/**
 * View or update AI agent configuration
 */
export async function callAiAgentConfig(
  ctx: Context,
  options: AiAgentConfigOptions = {},
): Promise<void> {
  const logger = ctx.logger;

  // Load configuration
  const config = await loadConfig();
  if (!config) {
    logger.error("❌ No configuration found!");
    logger.info("Please run: si ai-agent init");
    throw new Error("AI agent not initialized");
  }

  // If no options provided, show config by default
  if (!options.updateToken && !options.tool) {
    options.show = true;
  }

  // Show current configuration
  if (options.show) {
    logger.info("SI AI Agent Configuration\n");
    logger.info("=========================\n");
    logger.info(`Configuration file: ${getConfigPath()}`);
    logger.info(`API Token: ${maskToken(config.apiToken)}`);
    logger.info(`Tool: ${config.tool}\n`);

    if (!options.updateToken && !options.tool) {
      logger.info("To update configuration:");
      logger.info("  Update token: si ai-agent config --update-token");
      logger.info("  Update tool: si ai-agent config --tool <tool>");
    }
  }

  let configChanged = false;

  // Update API token
  if (options.updateToken) {
    logger.info("\n🔑 Update API Token\n");

    const newToken = await Secret.prompt({
      message: "Please paste your new API token:",
    });

    if (!newToken) {
      logger.error("❌ Token cannot be empty");
      throw new Error("Token cannot be empty");
    }

    if (!validateToken(newToken)) {
      logger.error(
        "❌ Invalid token format. System Initiative tokens are JWTs",
      );
      throw new Error("Invalid token format");
    }

    config.apiToken = newToken;
    configChanged = true;
    logger.info("✅ API token updated");
  }

  // Update tool
  if (options.tool) {
    logger.info(`\nUpdating tool to: ${options.tool}`);
    config.tool = options.tool as AiTool;
    configChanged = true;
    logger.info("✅ Tool updated");
  }

  // Save configuration if changed
  if (configChanged) {
    await saveConfig(config);
    logger.info("\n💾 Configuration saved");
    logger.info("\nNote: If the AI agent is running, restart it to apply changes:");
    logger.info("  si ai-agent stop");
    logger.info("  si ai-agent start");
  }

  // Track AI agent config command
  ctx.analytics.trackEvent("ai-agent config", {
    show: options.show ?? false,
    updateToken: options.updateToken ?? false,
    updateTool: !!options.tool,
    tool: config.tool,
  });
}
