import { Configuration } from "@systeminit/api-client";
import { logger } from "./logger.ts";
import { jwtDecode } from "jwt-decode";
import { analytics } from "./analytics.ts";

interface SIJwtPayload {
  workspaceId: string;
  userId: string;
  [key: string]: unknown;
}

// Only initialize if not in test mode (Deno.test is available during test runs)
const isTestMode = typeof Deno.test !== "undefined";

const apiToken = isTestMode ? undefined : Deno.env.get("SI_API_TOKEN");
if (!apiToken && !isTestMode) {
  logger.error(
    "SI_API_TOKEN is not defined; re-run the MCP server with your authentication token set in the environment",
  );
  Deno.exit(10);
}
const baseUrl = Deno.env.get("SI_BASE_URL") || "https://api.systeminit.com";
export const apiConfig = apiToken ? new Configuration({
  basePath: baseUrl,
  accessToken: apiToken,
  baseOptions: {
    headers: {
      Authorization: `Bearer ${apiToken}`,
    },
  },
}) : undefined as unknown as Configuration;

const decoded = apiToken ? jwtDecode<SIJwtPayload>(apiToken) : undefined;
export const WORKSPACE_ID: string | undefined = decoded?.workspaceId;
export const USER_ID: string | undefined = decoded?.userId;
if (!isTestMode) {
  if (!WORKSPACE_ID) {
    logger.error(
      `Tried to extract workspace ID from API Token, but failed. Your token is likely malformed! The decoded token follows:\n\n${
        JSON.stringify(decoded, null, 2)
      }`,
    );
  }
  if (USER_ID) {
    await analytics.identifyUser();
  }
}
