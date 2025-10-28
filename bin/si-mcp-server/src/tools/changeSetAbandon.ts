import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ChangeSetsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const name = "change-set-abandon";
const title = "Abandon a change set";
const description =
  `<description>Abandon a change set. Returns 'success' if the status was changed. On failure, returns error details</description><usage>Use this tool to Abandon a change set. You may *never* abandon the HEAD change set.</usage>`;

const AbandonChangeSetInputSchemaRaw = {
  changeSetId: z.string().describe("the ID of the change set to abandon"),
};

const AbandonChangeSetOutputSchemaRaw = {
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
const AbandonChangeSetOutputSchema = z.object(AbandonChangeSetOutputSchemaRaw);

export function changeSetAbandonTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "changeSetAbandonResponse",
        AbandonChangeSetOutputSchema,
      ),
      annotations: {
        destructiveHint: true,
      },
      inputSchema: AbandonChangeSetInputSchemaRaw,
      outputSchema: AbandonChangeSetOutputSchemaRaw,
    },
    async ({ changeSetId }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        if (!changeSetId) {
          return errorResponse({
            message:
              "Must provide a change set id; ensure you get one from the user!",
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
                "You may not abandon the HEAD change set. Inform the user that HEAD is immutable, and they should not try and abandon it. Call them a cheeky monkey.",
            });
          }
        } catch (error) {
          return errorResponse(error);
        }

        try {
          const response = await siApi.abandonChangeSet({
            workspaceId: WORKSPACE_ID,
            changeSetId,
          });
          return successResponse(response.data);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
