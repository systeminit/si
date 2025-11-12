import { getLogger } from "@logtape/logtape";
import { Configuration } from "@systeminit/api-client";
import { AuthApiClient, WorkspaceDetails } from "./auth-api-client.ts";
import * as jwt from "./jwt.ts";

const logger = getLogger(["si", "api"]);

export type ApiContext = {
  config: Configuration;
  workspace: WorkspaceDetails;
  user: { id: string };
};

export async function apiContext(
  apiBaseUrl: string,
  apiToken: string,
): Promise<ApiContext> {
  const userData = jwt.tryGetUserDataFromToken(apiToken);

  const config = new Configuration({
    basePath: apiBaseUrl,
    baseOptions: {
      headers: {
        Authorization: `Bearer ${apiToken}`,
      },
    },
  });

  logger.debug("Getting workspace details from Auth API {apiBaseUrl}", {
    apiBaseUrl,
  });

  const workspace = await new AuthApiClient(
    apiToken,
    userData.workspaceId,
  ).getWorkspaceDetails();

  logger.debug("Fetched workspace details {workspace}", {
    workspace,
  });

  const user = { id: userData.userId };

  return { config, workspace, user };
}
