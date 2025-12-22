/**
 * Workspace Leave Module - Leave a workspace
 *
 * This module provides functionality to leave a workspace in System Initiative.
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

export interface WorkspaceLeaveOptions {
  workspace: string;
  authApiUrl: string;
}

/**
 * Main entry point for the workspace leave command
 */
export async function callWorkspaceLeave(
  options: WorkspaceLeaveOptions,
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

    const workspaceToLeaveId = await prompt.workspace(
      workspaces,
      options.workspace,
    );

    const workspaceToLeave = workspaces.find(
      (w) => w.id === workspaceToLeaveId,
    );
    if (!workspaceToLeave) {
      ctx.logger.error(`Workspace not found: ${workspaceToLeaveId}`);
      Deno.exit(1);
    }

    const workspaceName = workspaceToLeave.displayName || workspaceToLeaveId;

    // Prevent leaving the current workspace
    const currentWorkspaceId = getCurrentWorkspace();
    if (currentWorkspaceId === workspaceToLeaveId) {
      ctx.logger.error(
        `Cannot leave workspace "${workspaceName}" because it is your current workspace. Please switch to a different workspace first using 'si workspace switch'.`,
      );
      Deno.exit(1);
    }

    const confirmed = await Confirm.prompt({
      message: `Are you sure you want to leave workspace "${workspaceName}"?`,
      default: false,
    });

    if (!confirmed) {
      ctx.logger.info("Workspace leave cancelled.");
      return;
    }

    ctx.logger.info(`Leaving workspace "${workspaceName}"...`);

    const apiConfig = Context.apiConfig();
    const workspaceMgmtApi = new WorkspaceManagementApi(apiConfig);

    try {
      await workspaceMgmtApi.leaveWorkspace({
        workspaceId: workspaceToLeaveId,
      });
    } catch (apiError: unknown) {
      if (hasResponseStatus(apiError)) {
        const { status } = apiError.response;
        if (status === 403) {
          ctx.logger.error(
            `Cannot leave workspace "${workspaceName}". You may be the owner or the last admin. Owners cannot leave their own workspace.`,
          );
          Deno.exit(1);
        } else if (status === 404) {
          ctx.logger.error(
            `Workspace "${workspaceName}" not found or you are not a member of this workspace.`,
          );
          Deno.exit(1);
        }
      }
      throw apiError;
    }

    ctx.logger.info(`Successfully left workspace: ${workspaceName}`);

    removeWorkspace(currentUserId, workspaceToLeaveId);

    ctx.analytics.trackEvent("workspace leave", {
      workspaceName,
    });
  } catch (error) {
    ctx.logger.error(`Failed to leave workspace: ${error}`);
    Deno.exit(1);
  }
}
