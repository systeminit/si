/**
 * Workspace Member List Module - List members of a workspace
 *
 * This module provides functionality to list all members and their roles in the current workspace.
 *
 * @module
 */

import { WorkspaceManagementApi } from "@systeminit/api-client";
import { Context } from "../context.ts";
import {
  getCurrentUser,
  getCurrentWorkspace,
  getUserDetails,
  getWorkspaceDetails,
} from "../cli/config.ts";
import { hasResponseStatus } from "./shared.ts";

export interface WorkspaceMemberListOptions {
  authApiUrl: string;
}

/**
 * Main entry point for the workspace member list command
 */
export async function callWorkspaceMemberList(
  _options: WorkspaceMemberListOptions,
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

  // Get current workspace
  const currentWorkspaceId = getCurrentWorkspace();
  if (!currentWorkspaceId) {
    ctx.logger.error(
      "No workspace selected. Please run 'si workspace switch' to select a workspace first.",
    );
    Deno.exit(1);
  }

  const { workspaceDetails } = getWorkspaceDetails(
    currentUserId,
    currentWorkspaceId,
  );
  if (!workspaceDetails) {
    ctx.logger.error(
      `Workspace configuration not found. Please run 'si workspace switch' to reconfigure.`,
    );
    Deno.exit(1);
  }

  const workspaceName = workspaceDetails.displayName || currentWorkspaceId;

  ctx.logger.info(`\nMembers of workspace "${workspaceName}":\n`);

  const apiConfig = Context.apiConfig();
  const workspaceMgmtApi = new WorkspaceManagementApi(apiConfig);

  try {
    const response = await workspaceMgmtApi.listMembers({
      workspaceId: currentWorkspaceId,
    });

    const members = response.data;

    if (!members || members.length === 0) {
      ctx.logger.info("No members found in this workspace.");
      return;
    }

    // Sort members by role (owner first, then approver, then collaborator)
    const roleOrder = { "owner": 0, "approver": 1, "collaborator": 2 };
    const sortedMembers = [...members].sort((a, b) => {
      const roleA = roleOrder[a.role.toLowerCase() as keyof typeof roleOrder] ?? 999;
      const roleB = roleOrder[b.role.toLowerCase() as keyof typeof roleOrder] ?? 999;
      if (roleA !== roleB) return roleA - roleB;
      return a.email.localeCompare(b.email);
    });

    // Find max lengths for formatting
    const maxEmailLength = Math.max(...sortedMembers.map((m) => m.email.length));
    const maxNicknameLength = Math.max(...sortedMembers.map((m) => m.nickname?.length || 0));
    const maxRoleLength = Math.max(...sortedMembers.map((m) => m.role.length));

    // Print header
    ctx.logger.info(
      `${"Email".padEnd(maxEmailLength)}  ${"Nickname".padEnd(maxNicknameLength)}  ${"Role".padEnd(maxRoleLength)}`,
    );
    ctx.logger.info(
      `${"-".repeat(maxEmailLength)}  ${"-".repeat(maxNicknameLength)}  ${"-".repeat(maxRoleLength)}`,
    );

    // Print members
    for (const member of sortedMembers) {
      const email = member.email.padEnd(maxEmailLength);
      const nickname = (member.nickname || "").padEnd(maxNicknameLength);
      const role = member.role.padEnd(maxRoleLength);
      ctx.logger.info(`${email}  ${nickname}  ${role}`);
    }

    ctx.logger.info(`\nTotal members: ${members.length}`);

    ctx.analytics.trackEvent("workspace member list", {
      workspaceName,
      memberCount: members.length,
    });
  } catch (apiError: unknown) {
    if (hasResponseStatus(apiError)) {
      const { status } = apiError.response;
      if (status === 403) {
        ctx.logger.error(
          `Permission denied. You don't have permission to list members of workspace "${workspaceName}".`,
        );
        Deno.exit(1);
      } else if (status === 404) {
        ctx.logger.error(
          `Workspace "${workspaceName}" not found or you are not a member.`,
        );
        Deno.exit(1);
      }
    }
    ctx.logger.error(`Failed to list workspace members: ${apiError}`);
    Deno.exit(1);
  }
}
