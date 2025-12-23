/**
 * Workspace Member Management Module - Invite and update workspace members
 *
 * This module provides functionality to manage workspace members in System Initiative.
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
  getWorkspaceDetails,
} from "../cli/config.ts";
import { hasResponseStatus } from "./shared.ts";

export interface WorkspaceInviteOptions {
  email?: string;
  approvers?: string;
  authApiUrl: string;
}

type RoleType = "collaborator" | "approver";
type OperationType = "invite" | "update";

interface InviteWithRole {
  email: string;
  role: RoleType;
}

/**
 * Basic email validation
 */
function isValidEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

/**
 * Parse comma-separated emails and validate them
 */
function parseEmails(emailString: string): string[] {
  return emailString
    .split(",")
    .map((e) => e.trim())
    .filter((e) => e.length > 0);
}

/**
 * Main entry point for the workspace member management command
 */
export async function callWorkspaceInvite(
  options: WorkspaceInviteOptions,
): Promise<void> {
  const ctx = Context.instance();

  // Collect all invitations with their roles
  const invitations: InviteWithRole[] = [];

  // Handle single email (backwards compatibility) - defaults to collaborator
  if (options.email) {
    invitations.push({ email: options.email, role: "collaborator" });
  }

  // Handle role-based email lists
  if (options.approvers) {
    const emails = parseEmails(options.approvers);
    invitations.push(
      ...emails.map((email) => ({ email, role: "approver" as RoleType })),
    );
  }

  // Validate we have at least one invitation
  if (invitations.length === 0) {
    ctx.logger.error(
      "No emails provided. Provide an email address or use --approvers to specify approver emails.",
    );
    Deno.exit(1);
  }

  // Validate all emails
  const invalidEmails = invitations
    .map((inv) => inv.email)
    .filter((email) => !isValidEmail(email));
  if (invalidEmails.length > 0) {
    ctx.logger.error(
      `Invalid email format: ${invalidEmails.join(", ")}`,
    );
    Deno.exit(1);
  }

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

  // Show summary of invitations
  ctx.logger.info(
    `\nInviting/updating ${invitations.length} user(s) in workspace "${workspaceName}":`,
  );
  for (const inv of invitations) {
    ctx.logger.info(`  - ${inv.email} as ${inv.role}`);
  }

  // Confirm invitation
  const confirmed = await Confirm.prompt({
    message: `\nProceed with these changes?`,
    default: true,
  });

  if (!confirmed) {
    ctx.logger.info("Operations cancelled.");
    return;
  }

  const apiConfig = Context.apiConfig();
  const workspaceMgmtApi = new WorkspaceManagementApi(apiConfig);

  // Get current member list to check for existing members
  let currentMembers: Array<{ email: string; userId: string; role: string }> =
    [];
  try {
    const membersResponse = await workspaceMgmtApi.listMembers({
      workspaceId: currentWorkspaceId,
    });
    currentMembers = membersResponse.data;
  } catch (_error) {
    ctx.logger.warn("Could not fetch current members list");
  }

  const results: {
    email: string;
    operation: OperationType;
    success: boolean;
    error?: string;
  }[] = [];

  // Process invitations/updates
  for (const invitation of invitations) {
    try {
      // Check if user is already a member
      const existingMember = currentMembers.find(
        (m) => m.email.toLowerCase() === invitation.email.toLowerCase(),
      );

      if (existingMember) {
        // User already exists - check if we need to update their role
        const currentRole = existingMember.role.toLowerCase();
        const targetRole = invitation.role.toLowerCase();

        if (currentRole !== targetRole) {
          ctx.logger.info(
            `${invitation.email} is already a member with role ${existingMember.role}. Updating to ${invitation.role}...`,
          );

          // API expects uppercase role names: EDITOR or APPROVER
          const apiRoleName = invitation.role === "approver"
            ? "APPROVER"
            : "EDITOR";

          await workspaceMgmtApi.updateMemberRole({
            workspaceId: currentWorkspaceId,
            updateMemberRoleRequest: {
              userId: existingMember.userId,
              role: apiRoleName,
            },
          });

          ctx.logger.info(
            `✓ Updated ${invitation.email} role to ${invitation.role}`,
          );
          results.push({
            email: invitation.email,
            operation: "update",
            success: true,
          });
        } else {
          ctx.logger.info(
            `${invitation.email} is already a ${existingMember.role} - no change needed`,
          );
          results.push({
            email: invitation.email,
            operation: "update",
            success: true,
          });
        }
        continue;
      }

      // User doesn't exist - invite them
      ctx.logger.info(`Inviting ${invitation.email}...`);

      const memberResponse = await workspaceMgmtApi.inviteMember({
        workspaceId: currentWorkspaceId,
        inviteMemberRequest: {
          email: invitation.email,
        },
      });

      // If role is "approver", try to update their role immediately
      if (invitation.role === "approver" && memberResponse.data) {
        const invitedMember = memberResponse.data.find(
          (m) => m.email.toLowerCase() === invitation.email.toLowerCase(),
        );

        if (invitedMember && invitedMember.userId) {
          try {
            ctx.logger.info(
              `Setting role for ${invitation.email} to ${invitation.role}...`,
            );

            await workspaceMgmtApi.updateMemberRole({
              workspaceId: currentWorkspaceId,
              updateMemberRoleRequest: {
                userId: invitedMember.userId,
                role: "APPROVER", // API expects uppercase role name
              },
            });

            ctx.logger.info(
              `✓ Successfully set ${invitation.email} as ${invitation.role}`,
            );
          } catch (_roleError) {
            ctx.logger.warn(
              `Note: Could not update role for ${invitation.email}. ` +
                `They may need to accept the invitation first. ` +
                `You can update their role later using: si workspace members manage --approvers ${invitation.email}`,
            );
          }
        }
      }

      results.push({
        email: invitation.email,
        operation: "invite",
        success: true,
      });

      ctx.logger.info(`✓ Successfully invited ${invitation.email}`);
    } catch (apiError: unknown) {
      let errorMessage = "Unknown error";

      if (hasResponseStatus(apiError)) {
        const { status } = apiError.response;
        if (status === 403) {
          errorMessage = "Permission denied";
        } else if (status === 404) {
          errorMessage = "Workspace not found";
        } else if (status === 409) {
          errorMessage = "Already a member or has pending invitation";
        } else if (status === 400) {
          errorMessage = "Invalid request";
        }
      }

      results.push({
        email: invitation.email,
        operation: "invite",
        success: false,
        error: errorMessage,
      });

      ctx.logger.error(
        `✗ Failed to invite ${invitation.email}: ${errorMessage}`,
      );
    }
  }

  // Summary
  const successful = results.filter((r) => r.success).length;
  const failed = results.filter((r) => !r.success).length;

  ctx.logger.info(`\nOperation summary:`);
  ctx.logger.info(`  Successful: ${successful}`);
  if (failed > 0) {
    ctx.logger.info(`  Failed: ${failed}`);
  }

  ctx.analytics.trackEvent("workspace member management", {
    workspaceName,
    total: results.length,
    successful,
    failed,
    invitations: invitations.length,
  });

  // Exit with error if any operations failed
  if (failed > 0) {
    Deno.exit(1);
  }
}
