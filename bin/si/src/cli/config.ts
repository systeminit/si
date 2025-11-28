/**
 * CLI configuration utilities
 *
 * This module provides utilities for extracting configuration from environment variables.
 */

import * as jwt from "./jwt.ts";

export interface Config {
  apiUrl: string;
  apiToken: string;
  workspaceId: string;
}

export function extractConfig(): Config {
  // Get configuration from environment variables
  const apiUrl = Deno.env.get("SI_API_URL") || "https://api.systeminit.com";
  const apiToken = Deno.env.get("SI_API_TOKEN");

  if (!apiToken) {
    throw new Error("Error: SI_API_TOKEN environment variable is required.");
  }

  const userData = jwt.getUserDataFromToken(apiToken);
  if (!userData) {
    throw new Error(
      "Failed to extract user data from API token. Token may be empty or invalid.",
    );
  }

  return {
    apiUrl,
    apiToken,
    workspaceId: userData.workspaceId,
  };
}
