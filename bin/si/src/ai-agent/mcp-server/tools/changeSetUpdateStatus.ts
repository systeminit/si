import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import { ChangeSetsApi } from "@systeminit/api-client";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { Context } from "../../../context.ts";

const name = "change-set-update-status";
const title = "Apply a change set";
const description = `<description>Apply a change set. Returns 'success' if the status was changed. On failure, returns error details</description><usage>Use this tool to Apply a change set. If the change set requires approval, you should be told that it's waiting for approval on the web application and you should review the changes on that page before merging. You may *never* update the status of the HEAD change set.</usage>`;

const UpdateChangeSetInputSchemaRaw = {
  changeSetId: z.string().describe("the ID of the change set to apply"),
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
    async ({ changeSetId }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        if (!changeSetId) {
          return errorResponse({
            message:
              "Must provide a change set id; ensure you get one from the user!",
          });
        }

        const apiConfig = Context.apiConfig();
        const workspaceId = Context.workspaceId();
        const siApi = new ChangeSetsApi(apiConfig);
        try {
          const response = await siApi.getChangeSet({
            workspaceId,
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
          const response = await siApi.requestApproval({
            workspaceId,
            changeSetId,
          });
          try {
            const newResponse = await siApi.getChangeSet({
              workspaceId,
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
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
