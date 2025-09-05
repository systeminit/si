import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ComponentsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import _ from "lodash";

const name = "component-enqueue-action";
const title = "Enqueue an action to run for a component in a change set";
const description =
  `<description>Enqueues an action to run for a component in a change set. Actions other than 'Refresh' will not be run until the change set is applied (refresh will be runn immediately).  Returns success if the action is enqueued. On failure, returns error details</description><usage>*Always* look up the correct action name by using the 'get-component' tool first. Use this tool when the user wants to enqueue an action for a component. 'Create' and 'Update' actions are automatically enqueued, so you don't need to enqueue them explicitly (but you can if the user requests it directly.) 'Refresh' functions are executed automatically rather than enqueued.</usage>`;

const EnqueueActionComponentInputSchemaRaw = {
  changeSetId: z.string().describe(
    "The change set to enqueue the action in",
  ),
  componentId: z.string().describe("the compoonent to enqueue the action for"),
  actionName: z.string().describe(
    "the name of the action to enqueue; can be listed with the component-get tool",
  ),
};

const EnqueueActionComponentOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    success: z.boolean().describe(
      "true if the action is successfully enqueued",
    ),
  }).describe("the component data"),
};
const EnqueueActionComponentOutputSchema = z.object(
  EnqueueActionComponentOutputSchemaRaw,
);

export function componentEnqueueActionTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentEnqueueActionResponse",
        EnqueueActionComponentOutputSchema,
      ),
      inputSchema: EnqueueActionComponentInputSchemaRaw,
      outputSchema: EnqueueActionComponentOutputSchemaRaw,
    },
    async (
      { changeSetId, componentId, actionName },
    ): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const siApi = new ComponentsApi(apiConfig);
        try {
          const response = await siApi.addAction({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            componentId,
            addActionV1Request: {
              action: {
                function: actionName,
              },
            },
          });
          return successResponse(
            response.data,
          );
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
