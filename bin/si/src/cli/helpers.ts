import {
  ChangeSetsApi,
  type ChangeSetViewV1,
  type Configuration,
} from "@systeminit/api-client";
import { AuthApiClient, type WorkspaceDetails } from "./auth.ts";
import { Context } from "../context.ts";
import { getWorkspaceDetails as getStoredWorkspaceDetails } from "./config.ts";

export type BaseCliContext = {
  ctx: Context;
};

export type AuthenticatedCliContext = BaseCliContext & {
  apiConfiguration: Configuration;
  workspace: WorkspaceDetails;
};

export async function getWorkspaceDetails(
  workspaceId: string,
): Promise<WorkspaceDetails> {
  const ctx = Context.instance();
  const userId = Context.userId();
  Context.apiConfig();

  const { workspaceDetails: maybeWorkspaceDetails } = getStoredWorkspaceDetails(
    userId,
    workspaceId,
  );
  if (maybeWorkspaceDetails) {
    return maybeWorkspaceDetails;
  }

  const authApiUrl = ctx.authApiUrl;
  const apiToken = ctx.authApiToken ?? ctx.apiToken;

  const authApi = new AuthApiClient(authApiUrl, apiToken);
  const workspace = await authApi.getWorkspaceDetails(workspaceId);

  return workspace;
}

/**
 * Get the HEAD changeset ID for a workspace.
 *
 * @returns The HEAD changeset ID
 * @throws Error if HEAD changeset cannot be found or API call fails
 */
export async function getHeadChangeSetId(): Promise<string> {
  const ctx = Context.instance();
  const apiConfig = Context.apiConfig();
  const workspaceId = Context.workspaceId();

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
