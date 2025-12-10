import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import { ActionsApi } from "@systeminit/api-client";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { Context } from "../../../context.ts";

const name = "action-update-status";
const title = "Update the status of an action";
const description = `<description>Update the status of an action. Returns 'success' if the status was changed. On failure, returns error details</description><usage>Use this tool to set an Action to 'on-hold' (ensuring it does not execute, nor will any dependent actions; puts it in the 'OnHold' status), to take it off hold 'off-hold' (puts it in the 'Queued' status), 'retry' to try the action again, etiher after it has failed or been put on hold, or 'remove' to remove if from the list of actions for this change set.</usage>`;

const UpdateActionInputSchemaRaw = {
  newStatus: z
    .enum(["on-hold", "off-hold", "retry", "remove"])
    .describe(
      "'on-hold' will set the action state to OnHold, ensuring it won't run. 'off-hold' will move the action back to the queue (taking it from OnHold to Queued). 'retry' will schedule an action to try again, either after it has failed or has been put on hold (effectively taking it 'off-hold' as well). 'remove' will remove the action from the queue for this change set.",
    ),
  changeSetId: z
    .string()
    .describe("the ID of the change set to maniplate the action queue of"),
  actionId: z.string().describe("the ID of the action to change the state of"),
};

const UpdateActionOutputSchemaRaw = {
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
const UpdateActionOutputSchema = z.object(UpdateActionOutputSchemaRaw);

export function actionUpdateTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "actionUpdateResponse",
        UpdateActionOutputSchema,
      ),
      annotations: {
        destructiveHint: true,
      },
      inputSchema: UpdateActionInputSchemaRaw,
      outputSchema: UpdateActionOutputSchemaRaw,
    },
    async ({ changeSetId, newStatus, actionId }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        if (!changeSetId) {
          return errorResponse({
            message:
              "Must provide a change set id; ensure you get one from the user!",
          });
        }
        if (!newStatus) {
          return errorResponse({
            message:
              "Must provide a new status for the action; ensure you get one from the user!",
          });
        }

        const apiConfig = Context.apiConfig();
        const workspaceId = Context.workspaceId();
        const siApi = new ActionsApi(apiConfig);
        try {
          // Confirm the change set you want to manipulate isn't HEAD
          if (newStatus == "on-hold") {
            const response = await siApi.putOnHold({
              workspaceId,
              changeSetId,
              actionId,
            });
            return successResponse(response.data);
          } else if (newStatus == "off-hold" || newStatus == "retry") {
            const response = await siApi.retryAction({
              workspaceId,
              changeSetId,
              actionId,
            });
            return successResponse(response.data);
          } else if (newStatus == "remove") {
            const response = await siApi.cancelAction({
              workspaceId,
              changeSetId,
              actionId,
            });
            return successResponse(response.data);
          } else {
            return errorResponse({
              message: `Invalid status '${newStatus}'. Must be one of: on-hold, off-hold, retry, remove`,
            });
          }
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
