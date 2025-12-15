/**
 * Workspace Create Module - Create new workspaces
 *
 * This module provides functionality to create new workspaces in System Initiative.
 *
 * @module
 */

import { WorkspaceManagementApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import {
  getApiUrlForInstance,
  getCurrentUser,
  getUserDetails,
  setCurrentWorkspace,
  writeWorkspace,
} from "../cli/config.ts";
import { AuthApiClient, type WorkspaceDetails } from "../cli/auth.ts";

export interface WorkspaceCreateOptions {
  name: string;
  description?: string;
  instanceUrl?: string;
  authApiUrl: string;
}

/**
 * Main entry point for the workspace create command
 */
export async function callWorkspaceCreate(
  options: WorkspaceCreateOptions,
): Promise<void> {
  const ctx = Context.instance();

  ctx.logger.info(
    `Creating workspace "${options.name}"...`,
  );

  try {
    const apiConfig = Context.apiConfig();
    const workspaceMgmtApi = new WorkspaceManagementApi(apiConfig);

    const response = await workspaceMgmtApi.createWorkspace({
      createWorkspaceRequest: {
        displayName: options.name,
        description: options.description || "",
        instanceUrl: options.instanceUrl || "https://app.systeminit.com",
      },
    });

    const apiWorkspace = response.data;

    if (!apiWorkspace.instanceUrl) {
      throw new Error("Workspace created but missing instanceUrl");
    }

    ctx.logger.info(
      `Workspace created successfully: ${apiWorkspace.displayName}`,
    );
    ctx.logger.info(`Workspace ID: ${apiWorkspace.id}`);
    ctx.logger.info(`Instance URL: ${apiWorkspace.instanceUrl}`);

    // Switch to the newly created workspace
    const currentUserId = getCurrentUser();
    if (!currentUserId) {
      ctx.logger.warn("Cannot switch to workspace: not logged in");
      return;
    }

    const { token } = getUserDetails(currentUserId);
    if (!token) {
      ctx.logger.warn("Cannot switch to workspace: no auth token found");
      return;
    }

    ctx.logger.info(`Generating workspace access token...`);
    const authApiClient = new AuthApiClient(options.authApiUrl, token);
    const workspaceToken = await authApiClient.createWorkspaceToken(
      apiWorkspace.id,
    );

    // Convert API Workspace to WorkspaceDetails
    const workspace: WorkspaceDetails = {
      id: apiWorkspace.id,
      displayName: apiWorkspace.displayName,
      instanceUrl: apiWorkspace.instanceUrl, // validated above
      instanceEnvType: apiWorkspace.instanceEnvType,
      baseUrl: getApiUrlForInstance(apiWorkspace.instanceUrl), // validated above
    };

    // Save workspace details and token
    writeWorkspace(currentUserId, workspace, workspaceToken);
    setCurrentWorkspace(workspace.id);

    ctx.logger.info(`Switched to new workspace: ${workspace.displayName}`);
    
    ctx.analytics.trackEvent("workspace create", {
      workspaceName: workspace.displayName
    });
  } catch (error) {
    ctx.logger.error(`Failed to create workspace: ${error}`);
    Deno.exit(1);
  }
}
