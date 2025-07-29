import { Configuration } from "@systeminit/api-client";
import { logger } from "./logger.ts";
import { jwtDecode } from "jwt-decode";

interface SIJwtPayload {
  workspaceId: string;
  userId: string;
  [key: string]: unknown;
}

const apiToken = Deno.env.get("SI_API_TOKEN");
if (!apiToken) {
  logger.error(
    "SI_API_TOKEN is not defined; re-run the MCP server with your authentication token set in the environment",
  );
  Deno.exit(10);
}
const baseUrl = Deno.env.get("SI_BASE_URL") || "https://api.systeminit.com";
export const apiConfig = new Configuration({
  basePath: baseUrl,
  accessToken: apiToken,
  baseOptions: {
    headers: {
      Authorization: `Bearer ${apiToken}`,
    },
  },
});
const decoded = jwtDecode<SIJwtPayload>(apiToken);
export const WORKSPACE_ID = decoded.workspaceId;
if (!WORKSPACE_ID) {
  logger.error(
    `Tried to extract workspace ID from API Token, but failed. Your token is likely malformed! The decoded token follows:\n\n${
      JSON.stringify(decoded, null, 2)
    }`,
  );
}
