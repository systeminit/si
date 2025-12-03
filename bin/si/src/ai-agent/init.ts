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

import { Input } from "@cliffy/prompt";
import { join } from "@std/path";
import { Context } from "../context.ts";
import {
  createAgentsMd,
  createClaudeMd,
  createClaudeSettings,
  createCodexConfig,
  createMcpConfig,
  createOpenCodeConfig,
  createOpenCodeMd,
  DEFAULT_CONFIG,
  getConfigPath,
  loadConfig,
  saveConfig,
} from "./shared.ts";

import type { AiTool } from "./shared.ts";
import { doLogin } from "../cli/login.ts";

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
  let apiToken = ctx.apiToken;

  // Check if there's an existing config
  const existingConfig = await loadConfig();
  if (existingConfig && !apiToken) {
    logger.info(
      "\n🔑 No API token configured for si, but we found existing SI API token in agent configuration",
    );
    const useExisting = await Input.prompt({
      message: "Use existing token? (y/n)",
      default: "y",
    });

    if (
      useExisting.toLowerCase() === "y" ||
      useExisting.toLowerCase() === "yes"
    ) {
      apiToken = existingConfig.apiToken;
      logger.info("✅ Using existing API token\n");
    }
  }

  // Prompt for token if not provided and not reusing existing
  if (!apiToken) {
    apiToken = await doLogin(ctx.authApiUrl);
    ctx = Context.instance();
  }

  const baseUrl = ctx.baseUrl ?? "https://api.systeminit.com";

  // Determine tool
  const tool = options.tool || existingConfig?.tool || DEFAULT_CONFIG.tool;

  // Create configuration
  const config = {
    apiToken,
    baseUrl,
    tool,
  };

  // Save configuration
  logger.info("💾 Saving configuration...");
  await saveConfig(config);
  logger.info(`✅ Configuration saved to: ${getConfigPath()}\n`);

  // Determine target directory for MCP and Claude config files
  const targetDir = options.targetDir || Deno.cwd();

  // Create tool-specific configurations
  switch (tool) {
    case "claude": {
      // Create MCP configuration for Claude
      logger.info("📄 Creating MCP configuration file...");
      const mcpPath = await createMcpConfig(apiToken, baseUrl, targetDir);
      logger.info(`✅ Created MCP configuration: ${mcpPath}\n`);

      logger.info("📄 Creating Claude settings configuration...");
      const settingsPath = await createClaudeSettings(targetDir);
      logger.info(`✅ Created Claude settings: ${settingsPath}\n`);

      logger.info("📄 Creating CLAUDE.md context file...");
      const claudeMdPath = await createClaudeMd(targetDir);
      logger.info(`✅ Created CLAUDE.md: ${claudeMdPath}\n`);
      break;
    }

    case "codex": {
      logger.info("📄 Creating OpenAI Codex configuration...");
      const codexConfigPath = await createCodexConfig(
        apiToken,
        baseUrl,
        targetDir,
      );
      logger.info(`✅ Created Codex config: ${codexConfigPath}\n`);

      const envPath = join(targetDir, ".codex-env");
      logger.info(`✅ Created workspace-specific env: ${envPath}\n`);

      logger.info("📄 Creating AGENTS.md context file...");
      const agentsMdPath = await createAgentsMd(targetDir);
      logger.info(`✅ Created AGENTS.md: ${agentsMdPath}\n`);

      logger.info("ℹ️  Codex uses global config at ~/.codex/config.toml");
      logger.info(
        "ℹ️  For workspace-specific tokens, source .codex-env before launching:\n",
      );
      logger.info("   source .codex-env && codex\n");
      break;
    }

    case "opencode": {
      logger.info("📄 Creating OpenCode.ai configuration...");
      const openCodeConfigPath = await createOpenCodeConfig(
        apiToken,
        baseUrl,
        targetDir,
      );
      logger.info(`✅ Created OpenCode config: ${openCodeConfigPath}\n`);

      logger.info("📄 Creating OPENCODE.md context file...");
      const openCodeMdPath = await createOpenCodeMd(targetDir);
      logger.info(`✅ Created OPENCODE.md: ${openCodeMdPath}\n`);
      break;
    }

    default:
      logger.warn(
        `Unknown tool: ${tool}. Skipping tool-specific configuration.`,
      );
  }

  // Success message
  logger.info("🎉 AI Agent initialization complete!");
  logger.info(`\nTool: ${tool}`);
  logger.info("\nNext steps:");

  switch (tool) {
    case "codex":
      logger.info("  1. Install Codex CLI if not already installed:");
      logger.info("     brew install codex  OR  npm install -g @openai/codex");
      logger.info("  2. Start Codex: codex");
      logger.info(
        "     The SI MCP server will be available automatically via ~/.codex/config.toml",
      );
      logger.info(
        "  3. Authenticate with your OpenAI/ChatGPT account when prompted",
      );
      break;

    case "opencode":
      logger.info("  1. Install OpenCode.ai if not already installed:");
      logger.info("     brew install opencode-ai/tap/opencode");
      logger.info(
        "     OR curl -fsSL https://raw.githubusercontent.com/opencode-ai/opencode/refs/heads/main/install | bash",
      );
      logger.info("  2. Start OpenCode: opencode");
      logger.info(
        "     The SI MCP server will be available automatically via opencode.jsonc",
      );
      logger.info(
        '  3. Use OpenCode interactively or with prompts (opencode -p "your prompt")',
      );
      break;

    case "claude":
    default:
      logger.info("  1. Start the AI agent: si ai-agent start");
      logger.info(
        "     This will start the MCP server and launch your AI tool",
      );
      logger.info("  2. Check status: si ai-agent status");
      break;
  }
}
