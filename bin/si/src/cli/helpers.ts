import { Configuration } from "@systeminit/api-client";
import { AuthApiClient, type WorkspaceDetails } from "../auth-api-client.ts";
import { extractConfig } from "../config.ts";
import type { Context } from "../context.ts";

/// From the environment variables, extract the configuration needed to run auth commands
export async function initializeCliContextWithAuth(
  baseCtx: BaseCliContext,
): Promise<AuthenticatedCliContext> {
  const { ctx } = baseCtx;
  const { apiUrl, apiToken, workspaceId } = extractConfig();

  ctx.logger.debug(
    `Initializing CLI context with auth, pointing at ${apiUrl}, workspace ${workspaceId}`,
  );

  const apiConfiguration = new Configuration({
    basePath: apiUrl,
    accessToken: apiToken,
    baseOptions: {
      headers: {
        Authorization: `Bearer ${apiToken}`,
      },
    },
  });

  const workspace = await getWorkspaceDetails(apiToken, workspaceId);

  return { apiConfiguration, workspace, ctx: ctx };
}

export type BaseCliContext = {
  ctx: Context;
};

export type AuthenticatedCliContext = BaseCliContext & {
  apiConfiguration: Configuration;
  workspace: WorkspaceDetails;
};

export async function getWorkspaceDetails(
  apiToken: string,
  workspaceId?: string,
) {
  if (!workspaceId) {
    throw new Error("Workspace ID is required");
  }

  const authApi = new AuthApiClient(apiToken, workspaceId);

  const workspace = await authApi.getWorkspaceDetails();

  return workspace;
}
