import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import { ChangeSetsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { ChangeSetSchema } from "../data/changeSets.ts";

const name = "change-set-list";
const title = "List change sets";
const description =
  `<description>Lists change sets. Returns the ID, Name and Status of the change set. On failure, returns error details</description><usage>Use this tool when you need to know what change sets are available in the workspace.</usage>`;
const hints =
  "Apply, Force-Apply, and Abandon change sets using their 'id' and the change-set-status-update tool.";

const ListChangeSetsOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.array(ChangeSetSchema).optional().describe("The list of change sets"),
};
const ListChangeSetsOutputSchema = z.object(
  ListChangeSetsOutputSchemaRaw,
);

export function changeSetListTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "changeSetListResponse",
        ListChangeSetsOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      outputSchema: ListChangeSetsOutputSchemaRaw,
    },
    async (): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const siApi = new ChangeSetsApi(apiConfig);
        try {
          const response = await siApi.listChangeSets({
            workspaceId: WORKSPACE_ID!,
          });
          return successResponse(
            response.data.changeSets,
            hints,
          );
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
