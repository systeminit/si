/**
 * Template Loader - Loads template files for SI CLI
 * This module handles loading templates from various sources
 * to support both development and production (compiled binary) modes
 */

import { dirname, join } from "@std/path";

/**
 * Generic template loader that tries multiple path strategies
 */
async function loadTemplate(
  relativePath: string,
  fallbackContent?: string,
): Promise<string> {
  // Try multiple paths to find the template
  const possiblePaths = [
    // When running from source
    new URL(`../data/templates/${relativePath}`, import.meta.url).pathname,
    // Relative to current working directory
    join(Deno.cwd(), "data/templates", relativePath),
    // Relative to binary location
    join(dirname(Deno.execPath()), "data/templates", relativePath),
    // When running from monorepo root
    join(Deno.cwd(), "bin/si/data/templates", relativePath),
  ];

  let lastError: Error | null = null;

  for (const path of possiblePaths) {
    try {
      const content = await Deno.readTextFile(path);
      return content;
    } catch (error) {
      lastError = error as Error;
      // Try next path
    }
  }

  // If fallback content is provided, use it; otherwise throw error
  if (fallbackContent) {
    console.warn(
      `Failed to load template ${relativePath} from any path. Using fallback.`,
    );
    if (lastError) {
      console.warn("Last error:", lastError.message);
    }
    return fallbackContent;
  } else {
    throw new Error(
      `Failed to load template ${relativePath}. Tried paths: ${
        possiblePaths.join(", ")
      }. Last error: ${lastError?.message || "unknown"}`,
    );
  }
}

/**
 * Load the SI Agent Context template
 * Tries multiple strategies to find and load the template file
 */
export async function loadAgentContextTemplate(): Promise<string> {
  return await loadTemplate(
    "SI_Agent_Context.md.tmpl",
    // Fallback: minimal template
    `# System Initiative Assistant Guide

This is a repo for working with System Initiative infrastructure through the MCP server.

## Interacting with System Initiative

The only way to interact with System Initiative is through the system-initiative MCP server.
All infrastructure operations should use the MCP tools.

## Available MCP Tools

Use MCP tools to discover schemas, create components, and manage infrastructure.

For full documentation, see: https://docs.systeminit.com
`,
  );
}

/**
 * Load the TypeScript template shell file
 * Used by the template generate command
 */
export async function loadTemplateShell(): Promise<string> {
  return await loadTemplate("template.ts.tmpl");
}
