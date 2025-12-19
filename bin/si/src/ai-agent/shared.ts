/**
 * AI Agent Module - Shared utilities for managing the SI AI Agent (MCP server)
 *
 * This module provides utilities for managing the local SI AI Agent,
 * including configuration management, process operations, and validation.
 *
 * @module
 */

import { ensureDir } from "@std/fs";
import { join } from "@std/path";
import type { Logger } from "../logger.ts";
import { loadAgentContextTemplate } from "../template-loader.ts";

/** SI Agent Context template - lazy loaded to avoid top-level await in tests */
let _cachedTemplate: string | null = null;

async function getAgentContextTemplate(): Promise<string> {
  if (_cachedTemplate) {
    return _cachedTemplate;
  }

  _cachedTemplate = await loadAgentContextTemplate();
  return _cachedTemplate;
}

/** Supported AI coding tools */
export type AiTool = "claude" | "codex" | "opencode" | "cursor";

/** Configuration for the AI Agent */
export interface AiAgentConfig {
  apiToken: string;
  tool: AiTool;
  mcpServerPath?: string;
  baseUrl: string;
}

/** Default configuration values */
export const DEFAULT_CONFIG: Omit<AiAgentConfig, "apiToken" | "baseUrl"> = {
  tool: "claude",
};

/** Tool-specific commands */
export const TOOL_COMMANDS: Record<AiTool, string> = {
  claude: "claude",
  codex: "codex",
  opencode: "opencode",
  cursor: "cursor-agent",
};

/** MCP server configuration structure */
interface McpServerConfig {
  type: string;
  command: string;
  args: string[];
  env: Record<string, string>;
}

interface McpConfig {
  mcpServers: {
    "system-initiative": McpServerConfig;
  };
}

/**
 * Check if an MCP config is using the old Docker-based format
 */
function isOldDockerFormat(config: McpConfig): boolean {
  const serverConfig = config.mcpServers["system-initiative"];
  return (
    serverConfig.command === "docker" &&
    Array.isArray(serverConfig.args) &&
    serverConfig.args.includes("systeminit/si-mcp-server:stable")
  );
}

/**
 * Migrate an old Docker-based MCP config to the new bundled format
 */
function migrateDockerConfig(
  oldConfig: McpConfig,
  newCommand: string,
): McpConfig {
  const oldServerConfig = oldConfig.mcpServers["system-initiative"];

  // Preserve the API token from the old config
  const apiToken = oldServerConfig.env.SI_API_TOKEN;

  return {
    mcpServers: {
      "system-initiative": {
        type: "stdio",
        command: newCommand,
        args: ["ai-agent", "stdio"],
        env: {
          SI_API_TOKEN: apiToken,
        },
      },
    },
  };
}

/** Claude settings configuration structure */
interface ClaudeSettings {
  enabledMcpjsonServers: string[];
  permissions: {
    allow: string[];
    deny: string[];
  };
}

/**
 * Get the path to the AI agent configuration directory
 */
export function getConfigDir(): string {
  // deno-lint-ignore si-rules/no-deno-env-get
  const home = Deno.env.get("HOME") || Deno.env.get("USERPROFILE");
  if (!home) {
    throw new Error("Could not determine home directory");
  }
  return join(home, ".si");
}

/**
 * Get the path to the AI agent configuration file
 */
export function getConfigPath(): string {
  return join(getConfigDir(), "ai-agent.json");
}

/**
 * Load the AI agent configuration from disk
 */
export async function loadConfig(): Promise<AiAgentConfig | null> {
  try {
    const configPath = getConfigPath();
    const content = await Deno.readTextFile(configPath);
    return JSON.parse(content) as AiAgentConfig;
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      return null;
    }
    throw error;
  }
}

/**
 * Save the AI agent configuration to disk
 */
export async function saveConfig(config: AiAgentConfig): Promise<void> {
  const configDir = getConfigDir();
  await ensureDir(configDir);

  const configPath = getConfigPath();
  await Deno.writeTextFile(configPath, JSON.stringify(config, null, 2));

  // Set restrictive permissions (owner read/write only)
  if (Deno.build.os !== "windows") {
    await Deno.chmod(configPath, 0o600);
  }
}

/**
 * Validate that a token is in JWT format (three base64 parts separated by dots)
 */
export function validateToken(token: string): boolean {
  return /^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/.test(token);
}

/**
 * Create the .mcp.json configuration file
 * Points to the bundled MCP server in si binary
 * If an old Docker-based config exists, it will be migrated to the new format
 */
export async function createMcpConfig(
  apiToken: string,
  baseUrl: string,
  targetDir: string,
): Promise<string> {
  // Get the absolute path to the si binary
  // Deno.execPath() returns the path to the currently running executable
  const siBinaryPath = Deno.execPath();

  const mcpPath = join(targetDir, ".mcp.json");

  // Check if .mcp.json already exists
  let mcpConfig: McpConfig;
  try {
    const existingContent = await Deno.readTextFile(mcpPath);
    const existingConfig = JSON.parse(existingContent) as McpConfig;

    // Check if it's using the old Docker format
    if (isOldDockerFormat(existingConfig)) {
      // Migrate to new format, preserving the API token
      mcpConfig = migrateDockerConfig(existingConfig, siBinaryPath);
    } else {
      // Already in new format, just update the API token
      mcpConfig = {
        mcpServers: {
          "system-initiative": {
            type: "stdio",
            command: siBinaryPath,
            args: ["ai-agent", "stdio"],
            env: {
              SI_API_TOKEN: apiToken,
              SI_BASE_URL: baseUrl,
            },
          },
        },
      };
    }
  } catch (error) {
    // File doesn't exist or can't be read, create new config
    if (!(error instanceof Deno.errors.NotFound)) {
      // If it's not a "not found" error, something else went wrong
      throw error;
    }

    // Create new config
    mcpConfig = {
      mcpServers: {
        "system-initiative": {
          type: "stdio",
          command: siBinaryPath,
          args: ["ai-agent", "stdio"],
          env: {
            SI_API_TOKEN: apiToken,
            SI_BASE_URL: baseUrl,
          },
        },
      },
    };
  }

  await Deno.writeTextFile(mcpPath, JSON.stringify(mcpConfig, null, 2));
  return mcpPath;
}

/**
 * Create the .claude/settings.local.json configuration file
 */
export async function createClaudeSettings(targetDir: string): Promise<string> {
  const claudeDir = join(targetDir, ".claude");
  await ensureDir(claudeDir);

  const settings: ClaudeSettings = {
    enabledMcpjsonServers: ["system-initiative"],
    permissions: {
      allow: [
        "mcp__system-initiative__schema-find",
        "mcp__system-initiative__schema-attributes-list",
        "mcp__system-initiative__schema-attributes-documentation",
        "mcp__system-initiative__schema-create-edit-get",
        "mcp__system-initiative__validate-credentials",
        "mcp__system-initiative__change-set-list",
        "mcp__system-initiative__change-set-create",
        "mcp__system-initiative__action-list",
        "mcp__system-initiative__action-update-status",
        "mcp__system-initiative__func-run-get",
        "mcp__system-initiative__func-create-edit-get",
        "mcp__system-initiative__component-list",
        "mcp__system-initiative__component-get",
        "mcp__system-initiative__component-create",
        "mcp__system-initiative__component-update",
        "mcp__system-initiative__component-enqueue-action",
        "mcp__system-initiative__component-discover",
        "mcp__system-initiative__component-restore",
        "mcp__system-initiative__component-import",
        "mcp__system-initiative__generate-si-url",
        "mcp__system-initiative__upgrade-components",
        "mcp__system-initiative__template-generate",
        "mcp__system-initiative__template-list",
        "mcp__system-initiative__template-run",
      ],
      deny: [],
    },
  };

  const settingsPath = join(claudeDir, "settings.local.json");
  await Deno.writeTextFile(settingsPath, JSON.stringify(settings, null, 2));
  return settingsPath;
}

/**
 * Create a context markdown file for an AI tool
 * This provides the AI tool with context about working with SI infrastructure
 */
async function createContextFile(
  targetDir: string,
  filename: string,
): Promise<string> {
  const filePath = join(targetDir, filename);
  const content = await getAgentContextTemplate();
  await Deno.writeTextFile(filePath, content);
  return filePath;
}

/**
 * Create the CLAUDE.md file with System Initiative context
 * This provides Claude Code with context about working with SI infrastructure
 */
export const createClaudeMd = (targetDir: string): Promise<string> =>
  createContextFile(targetDir, "CLAUDE.md");

/**
 * Get the Codex config directory
 */
function getCodexConfigDir(): string {
  // deno-lint-ignore si-rules/no-deno-env-get
  const home = Deno.env.get("HOME") || Deno.env.get("USERPROFILE");
  if (!home) {
    throw new Error("Could not determine home directory");
  }
  return join(home, ".codex");
}

/**
 * Get the Codex config file path
 */
function getCodexConfigPath(): string {
  return join(getCodexConfigDir(), "config.toml");
}

/**
 * Escape a string for TOML format
 */
function escapeTomlString(str: string): string {
  return str
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")
    .replace(/\t/g, "\\t");
}

/**
 * Generate TOML configuration for an MCP server
 */
function generateMcpServerToml(
  serverName: string,
  command: string,
  args: string[],
  env: Record<string, string>,
  envVars: string[],
  enabledTools?: string[],
): string {
  let toml = `\n[mcp_servers.${serverName}]\n`;
  toml += `command = "${escapeTomlString(command)}"\n`;

  // Format args array
  toml += `args = [`;
  if (args.length > 0) {
    toml += args.map((arg) => `"${escapeTomlString(arg)}"`).join(", ");
  }
  toml += `]\n`;

  // Format env object (for hardcoded values)
  if (Object.keys(env).length > 0) {
    toml += `env = { `;
    const envEntries = Object.entries(env).map(
      ([key, value]) => `${key} = "${escapeTomlString(value)}"`,
    );
    toml += envEntries.join(", ");
    toml += ` }\n`;
  }

  // Format env_vars array (for pass-through from shell environment)
  if (envVars.length > 0) {
    toml += `env_vars = [`;
    toml += envVars.map((varName) => `"${varName}"`).join(", ");
    toml += `]\n`;
  }

  // Add enabled_tools if specified
  if (enabledTools && enabledTools.length > 0) {
    toml += `enabled_tools = [`;
    toml += enabledTools
      .map((tool) => `"${escapeTomlString(tool)}"`)
      .join(", ");
    toml += `]\n`;
  }

  return toml;
}

/**
 * Create or update the Codex config.toml file with SI MCP server
 * Codex uses TOML format at ~/.codex/config.toml
 */
export async function createCodexConfig(
  apiToken: string,
  baseUrl: string,
  targetDir?: string,
): Promise<string> {
  const codexConfigDir = getCodexConfigDir();
  await ensureDir(codexConfigDir);

  const configPath = getCodexConfigPath();
  const siBinaryPath = Deno.execPath();

  // List of SI MCP tools to enable (same as Claude permissions)
  const enabledSiTools = [
    "schema-find",
    "schema-attributes-list",
    "schema-attributes-documentation",
    "schema-create-edit-get",
    "validate-credentials",
    "change-set-list",
    "change-set-create",
    "action-list",
    "action-update-status",
    "func-run-get",
    "func-create-edit-get",
    "component-list",
    "component-get",
    "component-create",
    "component-update",
    "component-restore",
    "component-enqueue-action",
    "component-discover",
    "component-import",
    "generate-si-url",
    "upgrade-components",
    "template-generate",
    "template-list",
    "template-run",
  ];

  // Generate the SI MCP server configuration
  // NOTE: Unlike Claude which uses project-level .mcp.json, Codex uses
  // a global ~/.codex/config.toml. To support workspace-specific tokens:
  // 1. Create a project-level .env file with SI_API_TOKEN
  // 2. Users source it before running codex: `source .codex-env && codex`
  // 3. env_vars tells Codex to pass through SI_API_TOKEN from shell environment
  const siMcpConfig = generateMcpServerToml(
    "system-initiative",
    siBinaryPath,
    ["ai-agent", "stdio"], // Correct command: ai-agent stdio, not mcp-server stdio
    // Don't hardcode token - let it come from environment
    // This allows different workspaces to use different tokens
    {},
    ["SI_API_TOKEN", "SI_BASE_URL"], // Tell Codex to pass through this env var from shell
    enabledSiTools, // Explicitly allow these SI tools
  );

  // Write the TOML configuration to the config file
  await Deno.writeTextFile(configPath, siMcpConfig);

  // Create project-level .env file for workspace-specific token
  if (targetDir) {
    try {
      const envPath = join(targetDir, ".codex-env");
      const envContent = `# System Initiative API Token for this workspace
# Source this file before running Codex to use workspace-specific token:
#   source .codex-env && codex
export SI_API_TOKEN="${apiToken}"
export SI_BASE_URL="${baseUrl}"
`;
      await Deno.writeTextFile(envPath, envContent);
    } catch {
      // Silently fail if we can't create .env file
    }
  }

  return configPath;
}

/**
 * Create the AGENTS.md file with System Initiative context for Codex
 * This provides Codex with context about working with SI infrastructure
 * Codex reads AGENTS.md from the project root for project-specific instructions
 */
export const createAgentsMd = (targetDir: string): Promise<string> =>
  createContextFile(targetDir, "AGENTS.md");

/**
 * Create the OPENCODE.md file with System Initiative context for OpenCode.ai
 * This provides OpenCode with context about working with SI infrastructure
 * OpenCode reads context files from the project root
 */
export const createOpenCodeMd = (targetDir: string): Promise<string> =>
  createContextFile(targetDir, "OPENCODE.md");

/**
 * Create the .cursorrules file with System Initiative context for Cursor
 * This provides Cursor with context about working with SI infrastructure
 * Cursor reads .cursorrules from the project root for project-specific instructions
 */
export const createCursorRules = (targetDir: string): Promise<string> =>
  createContextFile(targetDir, ".cursorrules");

/**
 * Create the .cursor/mcp.json configuration file for Cursor
 * Cursor uses a different location than Claude Code (.cursor/mcp.json vs .mcp.json)
 */
export async function createCursorConfig(
  apiToken: string,
  baseUrl: string,
  targetDir: string,
): Promise<string> {
  // Get the absolute path to the si binary
  const siBinaryPath = Deno.execPath();

  // Create .cursor directory
  const cursorDir = join(targetDir, ".cursor");
  await ensureDir(cursorDir);

  const mcpPath = join(cursorDir, "mcp.json");

  // Create the MCP configuration for Cursor
  const mcpConfig = {
    mcpServers: {
      "system-initiative": {
        command: siBinaryPath,
        args: ["ai-agent", "stdio"],
        env: {
          SI_API_TOKEN: apiToken,
          SI_BASE_URL: baseUrl,
        },
      },
    },
  };

  await Deno.writeTextFile(mcpPath, JSON.stringify(mcpConfig, null, 2));
  return mcpPath;
}

/**
 * Create the opencode.jsonc configuration file for OpenCode.ai
 * OpenCode uses a single JSON config file with nested MCP structure
 */
export async function createOpenCodeConfig(
  apiToken: string,
  baseUrl: string,
  targetDir: string,
): Promise<string> {
  const siBinaryPath = Deno.execPath();
  const configPath = join(targetDir, "opencode.jsonc");

  // OpenCode uses a nested structure under "mcp" key
  // Note: OpenCode uses "environment" not "env" for environment variables
  const openCodeConfig = {
    mcp: {
      "system-initiative": {
        type: "local",
        command: [siBinaryPath, "ai-agent", "stdio"],
        environment: {
          SI_API_TOKEN: apiToken,
          SI_BASE_URL: baseUrl,
        },
      },
    },
  };

  await Deno.writeTextFile(configPath, JSON.stringify(openCodeConfig, null, 2));
  return configPath;
}

/**
 * Local process status information
 */
export interface LocalProcessStatus {
  running: boolean;
  pid?: number;
  startTime?: Date;
  command?: string;
}

/**
 * Version check cache to avoid excessive GitHub API calls
 */
interface VersionCache {
  lastCheck: string; // ISO date
  latestVersion: string;
}

/**
 * Get the si-mcp-server installation directory
 */
export function getMcpServerInstallDir(): string {
  const configDir = getConfigDir();
  return join(configDir, "bin");
}

/**
 * Get the path where si-mcp-server binary should be installed
 */
export function getMcpServerInstallPath(): string {
  const binDir = getMcpServerInstallDir();
  const binaryName =
    Deno.build.os === "windows" ? "si-mcp-server.exe" : "si-mcp-server";
  return join(binDir, binaryName);
}

/**
 * Get the version cache file path
 */
function getVersionCachePath(): string {
  return join(getConfigDir(), "mcp-server-version.json");
}

/**
 * Load the version cache
 */
async function loadVersionCache(): Promise<VersionCache | null> {
  try {
    const content = await Deno.readTextFile(getVersionCachePath());
    return JSON.parse(content);
  } catch {
    return null;
  }
}

/**
 * Save the version cache
 */
async function saveVersionCache(cache: VersionCache): Promise<void> {
  await ensureDir(getConfigDir());
  await Deno.writeTextFile(
    getVersionCachePath(),
    JSON.stringify(cache, null, 2),
  );
}

/**
 * Check if we should check for updates (once per day)
 */
async function shouldCheckForUpdates(): Promise<boolean> {
  const cache = await loadVersionCache();
  if (!cache) return true;

  const lastCheck = new Date(cache.lastCheck);
  const now = new Date();
  const dayInMs = 24 * 60 * 60 * 1000;

  return now.getTime() - lastCheck.getTime() > dayInMs;
}

/**
 * Get the latest si-mcp-server version from GitHub releases
 */
export async function getLatestMcpServerVersion(): Promise<string | null> {
  try {
    const response = await fetch(
      "https://api.github.com/repos/systeminit/si/releases/latest",
      {
        headers: {
          Accept: "application/vnd.github.v3+json",
          "User-Agent": "si-cli",
        },
      },
    );

    if (!response.ok) {
      return null;
    }

    const data = await response.json();
    return data.tag_name || null;
  } catch {
    return null;
  }
}

/**
 * Get the current installed si-mcp-server version
 */
export async function getCurrentMcpServerVersion(): Promise<string | null> {
  const binaryPath = getMcpServerInstallPath();

  try {
    const command = new Deno.Command(binaryPath, {
      args: ["--version"],
      stdout: "piped",
      stderr: "null",
    });
    const { success, stdout } = await command.output();

    if (success) {
      const output = new TextDecoder().decode(stdout).trim();
      // Extract version from output (e.g., "si-mcp-server 1.2.3")
      const match = output.match(/(\d+\.\d+\.\d+)/);
      return match ? match[1] : null;
    }
  } catch {
    // Binary doesn't exist or can't execute
  }

  return null;
}

/**
 * Download si-mcp-server binary from GitHub releases
 */
export async function downloadMcpServer(
  version: string,
  logger: Logger,
): Promise<void> {
  logger.info(`Downloading si-mcp-server ${version}...`);

  // Determine platform and architecture
  const platform = Deno.build.os;
  const arch = Deno.build.arch;

  // Map to GitHub release asset names
  let assetName: string;
  if (platform === "darwin" && arch === "aarch64") {
    assetName = "si-mcp-server-aarch64-apple-darwin";
  } else if (platform === "darwin" && arch === "x86_64") {
    assetName = "si-mcp-server-x86_64-apple-darwin";
  } else if (platform === "linux" && arch === "x86_64") {
    assetName = "si-mcp-server-x86_64-unknown-linux-gnu";
  } else if (platform === "windows" && arch === "x86_64") {
    assetName = "si-mcp-server-x86_64-pc-windows-msvc.exe";
  } else {
    throw new Error(
      `Unsupported platform: ${platform}-${arch}. ` +
        `Please download si-mcp-server manually from GitHub releases.`,
    );
  }

  const downloadUrl = `https://github.com/systeminit/si/releases/download/${version}/${assetName}`;

  logger.debug(`Downloading from: ${downloadUrl}`);

  // Download the binary
  const response = await fetch(downloadUrl);
  if (!response.ok) {
    throw new Error(
      `Failed to download: ${response.status} ${response.statusText}`,
    );
  }

  const binaryData = await response.arrayBuffer();

  // Ensure install directory exists
  const installDir = getMcpServerInstallDir();
  await ensureDir(installDir);

  // Write the binary
  const installPath = getMcpServerInstallPath();
  await Deno.writeFile(installPath, new Uint8Array(binaryData));

  // Make it executable (Unix-like systems)
  if (Deno.build.os !== "windows") {
    await Deno.chmod(installPath, 0o755);
  }

  logger.info(`✅ Installed to: ${installPath}`);

  // Update version cache
  await saveVersionCache({
    lastCheck: new Date().toISOString(),
    latestVersion: version,
  });
}

/**
 * Check for updates and download if needed
 * Returns true if a new version was downloaded
 */
export async function checkAndUpdateMcpServer(
  logger: Logger,
): Promise<boolean> {
  // Check if we should check for updates (daily limit)
  if (!(await shouldCheckForUpdates())) {
    logger.debug("Skipping update check (checked recently)");
    return false;
  }

  logger.debug("Checking for si-mcp-server updates...");

  // Get latest version from GitHub
  const latestVersion = await getLatestMcpServerVersion();
  if (!latestVersion) {
    logger.debug("Could not fetch latest version");
    return false;
  }

  // Get current version
  const currentVersion = await getCurrentMcpServerVersion();

  // If not installed or different version, download
  if (!currentVersion || currentVersion !== latestVersion.replace(/^v/, "")) {
    if (!currentVersion) {
      logger.info(`si-mcp-server not found. Downloading ${latestVersion}...`);
    } else {
      logger.info(
        `Updating si-mcp-server: ${currentVersion} → ${latestVersion}`,
      );
    }

    await downloadMcpServer(latestVersion, logger);
    return true;
  }

  // Update cache even if no update needed
  await saveVersionCache({
    lastCheck: new Date().toISOString(),
    latestVersion,
  });

  logger.debug(`si-mcp-server is up to date (${currentVersion})`);
  return false;
}

/**
 * Find the si-mcp-server binary
 * Searches in order (prioritizes explicit user choices):
 * 1. PATH (user explicitly added it - development override)
 * 2. Monorepo location (for development)
 * 3. Same directory as si binary (for manual distribution)
 * 4. Auto-installed location (~/.si/bin/ - production default)
 */
export async function findMcpServerBinary(): Promise<string | null> {
  // Try 1: Check if si-mcp-server is in PATH (highest priority for development)
  try {
    const whichCmd = Deno.build.os === "windows" ? "where" : "which";
    const command = new Deno.Command(whichCmd, {
      args: ["si-mcp-server"],
      stdout: "piped",
      stderr: "null",
    });
    const { success, stdout } = await command.output();
    if (success) {
      const path = new TextDecoder().decode(stdout).trim();
      if (path) {
        return path.split("\n")[0]; // Use first result on Windows
      }
    }
  } catch {
    // Not in PATH, continue
  }

  // Try 2: Monorepo location (for development)
  let currentDir = Deno.cwd();
  const root = Deno.build.os === "windows" ? currentDir.split("\\")[0] : "/";

  while (currentDir !== root) {
    const binaryPath = join(
      currentDir,
      "bin",
      "si-mcp-server",
      "si-mcp-server",
    );
    try {
      const stat = await Deno.stat(binaryPath);
      if (stat.isFile) {
        return binaryPath;
      }
    } catch {
      // Not found, continue
    }

    const parent = join(currentDir, "..");
    if (parent === currentDir) break;
    currentDir = parent;
  }

  // Try 3: Same directory as the current si binary
  try {
    const siPath = Deno.execPath();
    const siDir = siPath.substring(0, siPath.lastIndexOf("/") + 1);
    const binaryName =
      Deno.build.os === "windows" ? "si-mcp-server.exe" : "si-mcp-server";
    const colocatedBinary = join(siDir, binaryName);

    const stat = await Deno.stat(colocatedBinary);
    if (stat.isFile) {
      return colocatedBinary;
    }
  } catch {
    // Not found co-located, continue
  }

  // Try 4: Auto-installed location (production default)
  try {
    const installedPath = getMcpServerInstallPath();
    const stat = await Deno.stat(installedPath);
    if (stat.isFile) {
      return installedPath;
    }
  } catch {
    // Not found, continue
  }

  return null;
}

/**
 * Find the si-mcp-server source (for development with Deno)
 */
export async function findMcpServerSource(): Promise<string | null> {
  let currentDir = Deno.cwd();
  const root = Deno.build.os === "windows" ? currentDir.split("\\")[0] : "/";

  while (currentDir !== root) {
    const mainPath = join(currentDir, "bin", "si-mcp-server", "main.ts");
    try {
      const stat = await Deno.stat(mainPath);
      if (stat.isFile) {
        return mainPath;
      }
    } catch {
      // Not found, continue
    }

    const parent = join(currentDir, "..");
    if (parent === currentDir) break;
    currentDir = parent;
  }

  return null;
}

/**
 * Check if Deno is available
 */
export async function checkDenoAvailable(): Promise<boolean> {
  try {
    const command = new Deno.Command("deno", {
      args: ["--version"],
      stdout: "null",
      stderr: "null",
    });
    const { success } = await command.output();
    return success;
  } catch {
    return false;
  }
}

/**
 * Generate a simple hash for a directory path
 */
function hashDirPath(dirPath: string): string {
  // Normalize the path and convert to a safe filename
  return dirPath
    .toLowerCase()
    .replace(/^\//, "") // Remove leading slash
    .replace(/[^a-z0-9]+/g, "-") // Replace non-alphanumeric with dashes
    .replace(/^-+|-+$/g, "") // Remove leading/trailing dashes
    .slice(-50); // Keep last 50 chars (most specific part)
}

/**
 * Get the path to the PID file for a given directory
 */
export function getPidFilePath(workDir: string): string {
  const configDir = getConfigDir();
  const dirHash = hashDirPath(workDir);
  return join(configDir, `ai-agent-${dirHash}.pid`);
}

/**
 * Save a process PID to a file
 */
export async function savePid(workDir: string, pid: number): Promise<void> {
  const pidFile = getPidFilePath(workDir);
  const configDir = getConfigDir();
  await ensureDir(configDir);
  await Deno.writeTextFile(pidFile, String(pid));
}

/**
 * Load a process PID from a file
 */
export async function loadPid(workDir: string): Promise<number | null> {
  try {
    const pidFile = getPidFilePath(workDir);
    const content = await Deno.readTextFile(pidFile);
    return parseInt(content.trim(), 10);
  } catch {
    return null;
  }
}

/**
 * Remove the PID file
 */
export async function removePid(workDir: string): Promise<void> {
  try {
    const pidFile = getPidFilePath(workDir);
    await Deno.remove(pidFile);
  } catch {
    // Ignore errors if file doesn't exist
  }
}

/**
 * Check if a process is running by PID
 */
export async function isProcessRunning(pid: number): Promise<boolean> {
  try {
    // On Unix-like systems, sending signal 0 checks if process exists
    if (Deno.build.os !== "windows") {
      const command = new Deno.Command("kill", {
        args: ["-0", String(pid)],
        stdout: "null",
        stderr: "null",
      });
      const { success } = await command.output();
      return success;
    } else {
      // On Windows, use tasklist
      const command = new Deno.Command("tasklist", {
        args: ["/FI", `PID eq ${pid}`, "/NH"],
        stdout: "piped",
        stderr: "null",
      });
      const { stdout } = await command.output();
      const output = new TextDecoder().decode(stdout);
      return output.includes(String(pid));
    }
  } catch {
    return false;
  }
}

/**
 * Get the status of a locally running MCP server
 */
export async function getLocalProcessStatus(
  workDir: string,
): Promise<LocalProcessStatus> {
  const pid = await loadPid(workDir);
  if (!pid) {
    return { running: false };
  }

  const running = await isProcessRunning(pid);
  if (!running) {
    // Clean up stale PID file
    await removePid(workDir);
    return { running: false };
  }

  return {
    running: true,
    pid,
  };
}

/**
 * Start the local MCP server process
 * Runs the bundled MCP server via "si mcp-server stdio"
 */
export async function startLocalServer(
  config: AiAgentConfig,
  workDir: string,
  logger: Logger,
): Promise<void> {
  logger.info("Starting SI MCP server (bundled)...");

  // Use the bundled MCP server subcommand
  const command = "si";
  const args = ["mcp-server", "stdio"];

  logger.debug("Running: si mcp-server stdio");

  // Start the process
  const process = new Deno.Command(command, {
    args,
    env: {
      SI_API_TOKEN: config.apiToken,
      // deno-lint-ignore si-rules/no-deno-env-get
      SI_BASE_URL: Deno.env.get("SI_BASE_URL") || "https://api.systeminit.com",
    },
    stdin: "piped",
    stdout: "piped",
    stderr: "piped",
  }).spawn();

  // Save the PID
  await savePid(workDir, process.pid);

  // Give it a moment to start
  await new Promise((resolve) => setTimeout(resolve, 1000));

  // Verify it's still running
  const status = await getLocalProcessStatus(workDir);
  if (!status.running) {
    throw new Error("MCP server process failed to start");
  }

  logger.info(`✅ MCP server started successfully (PID: ${process.pid})`);
}

/**
 * Stop the local MCP server process
 */
export async function stopLocalServer(
  workDir: string,
  logger: Logger,
): Promise<void> {
  const pid = await loadPid(workDir);
  if (!pid) {
    return;
  }

  logger.info(`Stopping MCP server process (PID: ${pid})...`);

  try {
    if (Deno.build.os !== "windows") {
      // On Unix-like systems, use kill
      const command = new Deno.Command("kill", {
        args: [String(pid)],
        stdout: "null",
        stderr: "null",
      });
      await command.output();
    } else {
      // On Windows, use taskkill
      const command = new Deno.Command("taskkill", {
        args: ["/PID", String(pid), "/F"],
        stdout: "null",
        stderr: "null",
      });
      await command.output();
    }

    // Wait a moment for the process to stop
    await new Promise((resolve) => setTimeout(resolve, 500));

    // Clean up PID file
    await removePid(workDir);
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    throw new Error(`Failed to stop process: ${errorMsg}`);
  }
}
