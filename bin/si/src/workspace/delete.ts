/**
 * Workspace Delete Module - Delete a workspace
 *
 * This module provides functionality to delete a workspace in System Initiative.
 * Note: This is a soft delete and workspaces can be recovered.
 *
 * @module
 */

import { WorkspaceManagementApi } from "@systeminit/api-client";
import { Confirm } from "@cliffy/prompt";
import { Context } from "../context.ts";
import {
  getCurrentUser,
  getCurrentWorkspace,
  getUserDetails,
  removeWorkspace,
} from "../cli/config.ts";
import { AuthApiClient } from "../cli/auth.ts";
import * as prompt from "../cli/prompt.ts";
import { hasResponseStatus } from "./shared.ts";

export interface WorkspaceDeleteOptions {
  workspace: string;
  authApiUrl: string;
}

/**
 * Main entry point for the workspace delete command
 */
export async function callWorkspaceDelete(
  options: WorkspaceDeleteOptions,
): Promise<void> {
  const ctx = Context.instance();

  const currentUserId = getCurrentUser();
  if (!currentUserId) {
    ctx.logger.error(
      "Not logged in. Please run 'si login' to authenticate first.",
    );
    Deno.exit(1);
  }

  const { userDetails, token } = getUserDetails(currentUserId);
  if (!userDetails || !token) {
    ctx.logger.error(
      "User configuration corrupted. Please run 'si login' again.",
    );
    Deno.exit(1);
  }

  try {
    const authApiClient = new AuthApiClient(options.authApiUrl, token);
    const workspaces = await authApiClient.getWorkspaces();

    if (workspaces.length === 0) {
      ctx.logger.error("No workspaces available for this user.");
      Deno.exit(1);
    }

    const workspaceToDeleteId = await prompt.workspace(
      workspaces,
      options.workspace,
    );

    const workspaceToDelete = workspaces.find(
      (w) => w.id === workspaceToDeleteId,
    );
    if (!workspaceToDelete) {
      ctx.logger.error(`Workspace not found: ${workspaceToDeleteId}`);
      Deno.exit(1);
    }

    const workspaceName = workspaceToDelete.displayName || workspaceToDeleteId;

    // Prevent deleting the current workspace
    const currentWorkspaceId = getCurrentWorkspace();
    if (currentWorkspaceId === workspaceToDeleteId) {
      ctx.logger.error(
        `Cannot delete workspace "${workspaceName}" because it is your current workspace. Please switch to a different workspace first using 'si workspace switch'.`,
      );
      Deno.exit(1);
    }

    const confirmed = await Confirm.prompt({
      message:
        `Are you sure you want to delete workspace "${workspaceName}"? To recover this operation, you need to contact customer service at support@systeminit.com. This operation will leave any existing resources running.`,
      default: false,
    });

    if (!confirmed) {
      ctx.logger.info("Workspace deletion cancelled.");
      return;
    }

    ctx.logger.info(`Deleting workspace "${workspaceName}"...`);

    const apiConfig = Context.apiConfig();
    const workspaceMgmtApi = new WorkspaceManagementApi(apiConfig);

    try {
      await workspaceMgmtApi.deleteWorkspace({
        workspaceId: workspaceToDeleteId,
      });
    } catch (apiError: unknown) {
      if (hasResponseStatus(apiError)) {
        const { status } = apiError.response;
        if (status === 403) {
          ctx.logger.error(
            `Cannot delete workspace "${workspaceName}". You may not have permission to delete this workspace.`,
          );
          Deno.exit(1);
        } else if (status === 404) {
          ctx.logger.error(
            `Workspace "${workspaceName}" not found or has already been deleted.`,
          );
          Deno.exit(1);
        }
      }
      throw apiError;
    }

    ctx.logger.info(
      `Successfully deleted workspace: ${workspaceName}`,
    );

    removeWorkspace(currentUserId, workspaceToDeleteId);

    ctx.analytics.trackEvent("workspace delete", {
      workspaceName,
    });
  } catch (error) {
    ctx.logger.error(`Failed to delete workspace: ${error}`);
    Deno.exit(1);
  }
}
