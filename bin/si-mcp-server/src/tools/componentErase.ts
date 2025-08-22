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

const name = "component-erase";
const title = "Erase a component";
const description = `<description>Erase a component, removing it completely from System Initiative without enqueuing a delete action or ensuring that downstream components aren't impacted by the removal. This can leave upstream provider resources orphaned (no longer managed by System Initiative) and can negatively impact downstream components if not used carefully. Returns true when successfully erased. On failure, returns error details. This cannot be undone within the change set!</description><usage>Use this tool to remove a component from System Initiative in a change set. You can use this tool to immediately cleanup any components from the model without needing to enqueue a delete action. The component will be immediately removed from the change set and the real world resource will be left intact. If you erase a component that has subscriptions to downstream components, the subscriptions will be removed and the values will no longer be propagated. Only use this tool if it's acceptable to sever the connection with the real world resource, or you no longer need data to propagate to downstream components, otherwise you should prefer to use Delete. This cannot be undone within this change set!</usage>`;

const EraseComponentInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to erase the component from; components cannot be erased from the HEAD change set",
    ),
  componentId: z.string().describe("the component id to erase"),
};

const EraseComponentOutputSchemaRaw = {
  status: z
    .enum(["success", "failure"])
    .describe(
      "success when component is successfully erased, failure when an error occurs during erase",
    ),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({
    success: z.boolean().describe("a successful erase"),
  }),
};
const EraseComponentOutputSchema = z.object(EraseComponentOutputSchemaRaw);

type EraseComponentResult = z.infer<typeof EraseComponentOutputSchema>["data"];

export function componentEraseTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentEraseResponse",
        EraseComponentOutputSchema,
      ),
      inputSchema: EraseComponentInputSchemaRaw,
      outputSchema: EraseComponentOutputSchemaRaw,
    },
    async ({ changeSetId, componentId }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
      const siApi = new ComponentsApi(apiConfig);
      try {
        await siApi.eraseComponent({
          workspaceId: WORKSPACE_ID,
          changeSetId: changeSetId,
          componentId,
        });
        const result: EraseComponentResult = {
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
