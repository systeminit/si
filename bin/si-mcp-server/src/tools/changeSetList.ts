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
import { ChangeSetSchema } from "../data/changeSets.ts";
import { generateChangeSetListHints } from "../utils/deepLinks.ts";

const name = "change-set-list";
const title = "List change sets";
const description =
  `<description>Lists change sets. Returns the ID, Name and Status of the change set. On failure, returns error details</description><usage>Use this tool when you need to know what change sets are available in the workspace.</usage>`;
const hints =
  "Apply, Force-Apply, and Abandon change sets using their 'id' and the change-set-status-update tool.";

const ListChangeSetsOutputSchema = z.object({
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.array(ChangeSetSchema).optional().describe("The list of change sets"),
});

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
      outputSchema: ListChangeSetsOutputSchema.shape,
    },
    async (): Promise<CallToolResult> => {
      const siApi = new ChangeSetsApi(apiConfig);
      try {
        const response = await siApi.listChangeSets({
          workspaceId: WORKSPACE_ID,
        });
        const changeSetsWithLinks = generateChangeSetListHints(
          response.data.changeSets.map((cs: { id: string; name: string }) => ({ id: cs.id, name: cs.name }))
        );
        
        return successResponse(
          response.data.changeSets,
          `${hints}\n\n${changeSetsWithLinks}`,
        );
      } catch (error) {
        return errorResponse(error);
      }
    },
  );
}
