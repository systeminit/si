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
import { ChangeSetSchema } from "../data/changeSets.ts";
import { Context } from "../../../context.ts";

const name = "change-set-create";
const title = "Create change set";
const description = `<description>Creates a change set. Returns the ID, Name and Status of the change set. On failure, returns error details</description><usage>Use this tool to create a new change set.</usage>`;

const CreateChangeSetInputSchemaRaw = {
  changeSetName: z
    .string()
    .describe(
      "the name of the change set; should be descriptive for the intent of the change.",
    ),
};

const CreateChangeSetOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: ChangeSetSchema.optional().describe("The new change set"),
};
const CreateChangeSetOutputSchema = z.object(CreateChangeSetOutputSchemaRaw);

export function changeSetCreateTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "changeSetCreateResponse",
        CreateChangeSetOutputSchema,
      ),
      inputSchema: CreateChangeSetInputSchemaRaw,
      outputSchema: CreateChangeSetOutputSchemaRaw,
    },
    async ({ changeSetName }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        if (!changeSetName) {
          return errorResponse({
            message:
              "Must provide a change set name; ensure you get one from the user!",
          });
        }
        const apiConfig = Context.apiConfig();
        const workspaceId = Context.workspaceId();
        const siApi = new ChangeSetsApi(apiConfig);
        try {
          const response = await siApi.createChangeSet({
            workspaceId,
            createChangeSetV1Request: { changeSetName },
          });
          return successResponse(response.data.changeSet);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
