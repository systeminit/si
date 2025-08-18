import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ChangeSetsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
} from "./commonBehavior.ts";

const name = "change-set-update-status";
const title = "Update the status of a change set";
const description = `<description>Update the status of a change set. Returns 'success' if the status was changed. On failure, returns error details</description><usage>Use this tool to Abandon (or 'delete') a change set, Apply a change set, or Force Apply the change set (ignoring all approvals). You may *never* update the status of the HEAD change set.</usage>`;

const UpdateChangeSetInputSchemaRaw = {
  newStatus: z
    .enum(["abandon", "apply", "force-apply"])
    .describe(
      "'abandon' will abandon this change set, effectively deleting it. 'apply' will apply the change set and follow any workspace approval requirements. 'force-apply' will apply the change set *ignoring* all approval requirements",
    ),
  changeSetId: z
    .string()
    .describe("the ID of the change set to update the status of"),
};

const UpdateChangeSetOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z
    .object({ success: z.boolean().describe("will always be true") })
    .describe("will be true if the request succeeds"),
};
const UpdateChangeSetOutputSchema = z.object(UpdateChangeSetOutputSchemaRaw);

export function changeSetUpdateTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "changeSetUpdateResponse",
        UpdateChangeSetOutputSchema,
      ),
      annotations: {
        destructiveHint: true,
      },
      inputSchema: UpdateChangeSetInputSchemaRaw,
      outputSchema: UpdateChangeSetOutputSchemaRaw,
    },
    async ({ changeSetId, newStatus }): Promise<CallToolResult> => {
      if (!changeSetId) {
        return errorResponse({
          message:
            "Must provide a change set id; ensure you get one from the user!",
        });
      }
      if (!newStatus) {
        return errorResponse({
          message:
            "Must provide a new status for the change set; ensure you get one from the user!",
        });
      }

      const siApi = new ChangeSetsApi(apiConfig);
      try {
        const response = await siApi.getChangeSet({
          workspaceId: WORKSPACE_ID,
          changeSetId,
        });
        if (response.data.changeSet.isHead) {
          return errorResponse({
            message:
              "You may not change the status of the HEAD change set. Inform the user that HEAD is immutable, and they should not try and change its status. Call them a cheeky monkey.",
          });
        }
      } catch (error) {
        return errorResponse(error);
      }

      try {
        // Confirm the change set you want to manipulate isn't HEAD
        if (newStatus == "abandon") {
          const response = await siApi.abandonChangeSet({
            workspaceId: WORKSPACE_ID,
            changeSetId,
          });
          return successResponse(response.data);
        } else if (newStatus == "apply") {
          const response = await siApi.requestApproval({
            workspaceId: WORKSPACE_ID,
            changeSetId,
          });
          try {
            const newResponse = await siApi.getChangeSet({
              workspaceId: WORKSPACE_ID,
              changeSetId,
            });
            if (newResponse.data.changeSet.status == "NeedsApproval") {
              return successResponse(
                response.data,
                "Tell the user that while the request was successful, the Change Set requires review and explicit Approval.",
              );
            } else {
              return successResponse(response.data);
            }
          } catch (error) {
            return errorResponse(error);
          }
        } else if (newStatus == "force-apply") {
          const response = await siApi.forceApply({
            workspaceId: WORKSPACE_ID,
            changeSetId,
          });
          return successResponse(response.data);
        } else {
          return errorResponse({
            message: `Invalid status '${newStatus}'. Must be one of: abandon, apply or force-apply`,
          });
        }
      } catch (error) {
        return errorResponse(error);
      }
    },
  );
}
