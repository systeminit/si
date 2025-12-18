/**
 * Template Loader - Loads template files for SI CLI
 * This module handles loading templates from various sources
 * to support both development and production (compiled binary) modes
 */

import { dirname, join } from "@std/path";
import {
  SI_AGENT_CONTEXT_TEMPLATE,
  TEMPLATE_SHELL,
} from "./embedded-templates.ts";

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
    // Using embedded template (expected for compiled binaries)
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
 * Falls back to embedded template for compiled binaries
 */
export async function loadAgentContextTemplate(): Promise<string> {
  return await loadTemplate(
    "SI_Agent_Context.md.tmpl",
    // Fallback: embedded template (always available in compiled binaries)
    SI_AGENT_CONTEXT_TEMPLATE,
  );
}

/**
 * Load the TypeScript template shell file
 * Used by the template generate command
 * Falls back to embedded template for compiled binaries
 */
export async function loadTemplateShell(): Promise<string> {
  return await loadTemplate(
    "template.ts.tmpl",
    // Fallback: embedded template (always available in compiled binaries)
    TEMPLATE_SHELL,
  );
}
