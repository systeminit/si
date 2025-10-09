import { unknownValueToErrorMessage } from "./helpers.ts";

export const SCHEMA_FILE_FORMAT_VERSION = 0;

export interface Config {
  apiUrl: string;
  apiToken: string;
  workspaceId: string;
}

function decodeJWT(token: string): Record<string, unknown> {
  try {
    const parts = token.split(".");
    if (parts.length !== 3) {
      throw new Error("Invalid JWT format");
    }
    const payload = parts[1];
    const decoded = atob(payload.replace(/-/g, "+").replace(/_/g, "/"));
    return JSON.parse(decoded);
  } catch (error) {
    throw new Error(`Failed to decode JWT: ${unknownValueToErrorMessage(error)}`);
  }
}

export function extractConfig(): Config {
  // Get configuration from environment variables
  const apiUrl = Deno.env.get("SI_API_URL") || "https://api.systeminit.com";
  const apiToken = Deno.env.get("SI_API_TOKEN");

  if (!apiToken) {
    throw new Error("Error: SI_API_TOKEN environment variable is required.");
  }

  // Extract workspaceId from JWT if not provided
  let workspaceId = Deno.env.get("SI_WORKSPACE_ID");
  if (!workspaceId) {
    const payload = decodeJWT(apiToken);
    workspaceId = payload.workspaceId as string;
    if (!workspaceId) {
      throw new Error("workspaceId not found in JWT payload");
    }
  }

  return {
    apiUrl,
    apiToken,
    workspaceId,
  };
}