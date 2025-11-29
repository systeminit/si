/**
 * SI Client Module - System Initiative API Client Configuration
 *
 * This module provides a configured System Initiative API client that
 * automatically extracts workspace and user IDs from the SI_API_TOKEN
 * JWT token. The token must be set as an environment variable.
 *
 * @module
 */

import {
  ChangeSetsApi,
  type ChangeSetViewV1,
  Configuration,
} from "@systeminit/api-client";
import { jwtDecode } from "jwt-decode";
import { Context } from "./context.ts";

/**
 * JWT payload structure for System Initiative API tokens.
 */
interface SIJwtPayload {
  workspaceId: string;
  userId: string;
  [key: string]: unknown;
}

/**
 * Validates and retrieves the SI_API_TOKEN from environment.
 *
 * @returns The API token
 * @throws Exits the process if token is not found
 */
function _getApiToken(): string {
  const apiToken = Deno.env.get("SI_API_TOKEN");
  if (!apiToken) {
    console.error(
      "SI_API_TOKEN is not defined; re-run with your authentication token set in the environment",
    );
    Deno.exit(10);
  }
  return apiToken;
}

/**
 * Gets the System Initiative API base URL from environment.
 *
 * @returns The base URL (defaults to https://api.systeminit.com)
 */
function getBaseUrl(): string {
  return Deno.env.get("SI_BASE_URL") || "https://api.systeminit.com";
}

/**
 * Decodes the JWT token and extracts the payload.
 *
 * @param token - The JWT token to decode
 * @returns The decoded JWT payload
 */
function decodeToken(token: string): SIJwtPayload | null {
  try {
    return jwtDecode<SIJwtPayload>(token);
  } catch (error) {
    console.error(
      `Failed to decode SI_API_TOKEN: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
    // Don't exit - just return null and let caller handle it
    return null;
  }
}

/**
 * Validates the decoded token contains required fields.
 *
 * @param decoded - The decoded JWT payload
 */
function validateToken(decoded: SIJwtPayload): void {
  if (!decoded.workspaceId) {
    console.error(
      `Tried to extract workspace ID from API Token, but failed. Your token is likely malformed! The decoded token follows:\n\n${
        JSON.stringify(decoded, null, 2)
      }`,
    );
    Deno.exit(10);
  }
}

// Initialize the SI API client (only if token is available)
let apiConfig: Configuration | undefined;
let WORKSPACE_ID: string | undefined;
let USER_ID: string | undefined;

const apiToken = Deno.env.get("SI_API_TOKEN");
if (apiToken) {
  const baseUrl = getBaseUrl();
  const decoded = decodeToken(apiToken);

  // Only initialize if token decoded successfully
  if (decoded) {
    validateToken(decoded);

    apiConfig = new Configuration({
      basePath: baseUrl,
      accessToken: apiToken,
      baseOptions: {
        headers: {
          Authorization: `Bearer ${apiToken}`,
        },
      },
    });

    WORKSPACE_ID = decoded.workspaceId;
    USER_ID = decoded.userId;
  }
}

/**
 * Configured API client for System Initiative.
 */
export { apiConfig };

/**
 * The workspace ID extracted from the API token.
 */
export { WORKSPACE_ID };

/**
 * The user ID extracted from the API token.
 */
export { USER_ID };

/**
 * Get the HEAD changeset ID for a workspace.
 *
 * @param apiConfig - API configuration
 * @param workspaceId - Workspace ID
 * @returns The HEAD changeset ID
 * @throws Error if HEAD changeset cannot be found or API call fails
 */
export async function getHeadChangeSetId(
  apiConfig: Configuration,
  workspaceId: string,
): Promise<string> {
  const ctx = Context.instance();

  const changeSetsApi = new ChangeSetsApi(apiConfig);
  const response = await changeSetsApi.listChangeSets({ workspaceId });

  // Find the HEAD changeset
  const changeSets = response.data.changeSets as ChangeSetViewV1[];
  const headChangeSet = changeSets.find((cs) => cs.isHead);

  if (headChangeSet) {
    ctx.logger.debug(`Found HEAD changeset: {id} ({name})`, {
      id: headChangeSet.id,
      name: headChangeSet.name,
    });
    return headChangeSet.id;
  } else {
    throw new Error("No HEAD changeset found in workspace");
  }
}
