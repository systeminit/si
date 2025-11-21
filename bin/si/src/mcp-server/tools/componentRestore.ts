import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import { ComponentsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
} from "./commonBehavior.ts";

const name = "component-restore";
const title = "Restore a component that is marked for deletion";
const description =
  `<description>Restores a component that is marked for deletion in a change set. Returns success if the component is restored. On failure, returns error details</description><usage>Use this tool when the user wants to restore a component that they marked for deletion. You can use the component-get tool to check if a component is set toDelete. A component will only be set toDelete if it exists on the HEAD change set and has an associated resource or has a dependent component. This will restore the component immediately and all of it's data and subscriptions. This will not work if the component has been erased!</usage>`;

const RestoreComponentInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to restore the component from; components that were erased cannot be restored",
    ),
  componentId: z.string().describe("the component id to restore"),
};

const RestoreComponentOutputSchemaRaw = {
  status: z
    .enum(["success", "failure"])
    .describe(
      "success when component is successfully restored, failure when an error occurs during restoration",
    ),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({
    success: z.boolean().describe("a successful restore"),
  }),
};
const RestoreComponentOutputSchema = z.object(RestoreComponentOutputSchemaRaw);

type RestoreComponentResult = z.infer<
  typeof RestoreComponentOutputSchema
>["data"];

export function componentRestoreTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentRestoreResponse",
        RestoreComponentOutputSchema,
      ),
      inputSchema: RestoreComponentInputSchemaRaw,
      outputSchema: RestoreComponentOutputSchemaRaw,
    },
    async ({ changeSetId, componentId }): Promise<CallToolResult> => {
      const siApi = new ComponentsApi(apiConfig);
      try {
        await siApi.restoreComponent({
          workspaceId: WORKSPACE_ID,
          changeSetId: changeSetId,
          componentId,
        });
        const result: RestoreComponentResult = {
          success: true,
        };

        return successResponse(result);
      } catch (error) {
        return errorResponse(error);
      }
    },
  );
}
