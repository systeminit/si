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

const name = "component-delete";
const title = "Delete a component";
const description =
  `<description>Delete a component and the resource for a given componentId. Returns the deletion status on successful deletion. On failure, returns error details.</description><usage>Use this tool to delete a component and its resource in a change set. The component and resource will be marked for deletion and the component removed from the workspace.</usage>`;

const DeleteComponentInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to delete the component from; components cannot be deleted from the HEAD change set",
    ),
  componentId: z.string().describe("the component id to delete"),
};

const DeleteComponentOutputSchemaRaw = {
  status: z
    .enum(["success", "failure"])
    .describe(
      "success when component is successfully marked for deletion, failure when an error occurs during deletion",
    ),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({
    success: z.boolean().describe("a successful deletion"),
  }),
};
const DeleteComponentOutputSchema = z.object(DeleteComponentOutputSchemaRaw);

type DeleteComponentResult = z.infer<
  typeof DeleteComponentOutputSchema
>["data"];

export function componentDeleteTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentDeleteResponse",
        DeleteComponentOutputSchema,
      ),
      inputSchema: DeleteComponentInputSchemaRaw,
      outputSchema: DeleteComponentOutputSchemaRaw,
    },
    async ({ changeSetId, componentId }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const siApi = new ComponentsApi(apiConfig);
        try {
          await siApi.deleteComponent({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            componentId,
          });
          const result: DeleteComponentResult = {
            success: true,
          };

          return successResponse(result);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
