/**
 * Template Loader - Loads template files for SI CLI
 * This module handles loading templates from various sources
 * to support both development and production (compiled binary) modes
 */

import { dirname, join } from "@std/path";
import {
  PROVIDER_AWS_TEMPLATE,
  PROVIDER_AZURE_TEMPLATE,
  PROVIDER_COMMON_TEMPLATE,
  PROVIDER_DIGITALOCEAN_TEMPLATE,
  PROVIDER_HETZNER_TEMPLATE,
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
 * Load the TypeScript template shell file
 * Used by the template generate command
 * Falls back to embedded template for compiled binaries
 */
export function loadTemplateShell(): Promise<string> {
  return loadTemplate(
    "template.ts.tmpl",
    // Fallback: embedded template (always available in compiled binaries)
    TEMPLATE_SHELL,
  );
}

/**
 * Get the embedded fallback template for a provider
 */
function getEmbeddedProviderTemplate(provider: string): string | undefined {
  const templates: Record<string, string> = {
    common: PROVIDER_COMMON_TEMPLATE,
    aws: PROVIDER_AWS_TEMPLATE,
    azure: PROVIDER_AZURE_TEMPLATE,
    hetzner: PROVIDER_HETZNER_TEMPLATE,
    digitalocean: PROVIDER_DIGITALOCEAN_TEMPLATE,
    // google: PROVIDER_GOOGLE_TEMPLATE,
  };
  return templates[provider];
}

/**
 * Load a provider-specific template file
 * @param provider - Provider name (aws, azure, hetzner, digitalocean, google, common)
 * @returns Promise resolving to the template content
 */
export async function loadProviderTemplate(provider: string): Promise<string> {
  const filename = `providers/${provider}.md.tmpl`;

  // Try multiple paths to find the template
  const possiblePaths = [
    // When running from source
    new URL(`../data/templates/${filename}`, import.meta.url).pathname,
    // Relative to current working directory
    join(Deno.cwd(), "data/templates", filename),
    // Relative to binary location
    join(dirname(Deno.execPath()), "data/templates", filename),
    // When running from monorepo root
    join(Deno.cwd(), "bin/si/data/templates", filename),
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

  // Fall back to embedded template (for compiled binaries)
  const embeddedTemplate = getEmbeddedProviderTemplate(provider);
  if (embeddedTemplate) {
    return embeddedTemplate;
  }

  throw new Error(
    `Failed to load provider template ${filename}. Tried paths: ${
      possiblePaths.join(", ")
    }. Last error: ${
      lastError?.message || "unknown"
    }. No embedded fallback available for provider: ${provider}`,
  );
}

/**
 * Load all provider templates
 * @returns Promise resolving to a map of provider names to template content
 */
export async function loadAllProviderTemplates(): Promise<
  Record<string, string>
> {
  const providers = [
    "common",
    "aws",
    "azure",
    "hetzner",
    "digitalocean",
    // "google",
  ];
  const templates: Record<string, string> = {};

  for (const provider of providers) {
    templates[provider] = await loadProviderTemplate(provider);
  }

  return templates;
}
