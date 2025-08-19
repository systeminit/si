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
import { generateChangeSetDeepLink } from "../utils/deepLinks.ts";
import { createResponseSchema } from "../data/base.ts";

const name = "change-set-create";
const title = "Create change set";
const description =
  `<description>Creates a change set. Returns the ID, Name and Status of the change set. On failure, returns error details</description><usage>Use this tool to create a new change set.</usage>`;

const CreateChangeSetInputSchemaRaw = {
  changeSetName: z.string().describe(
    "the name of the change set; should be descriptive for the intent of the change.",
  ),
};

const CreateChangeSetOutputSchema = createResponseSchema({
  id: z.string().describe("Change Set ID"),
  isHead: z.boolean().describe("True if the change set is HEAD; false if not"),
  name: z.string().describe("The name of the Change Set"),
  status: z.enum(["Abandoned", "Applied", "Approved", "Failed", "NeedsApproval", "Open", "Rejected"]).describe("The status of the change set. 'Abandoned' means it is no longer accessible. 'Applied' means it has been applied to HEAD. 'Approved' means any neccessary approvals have been applied. 'Failed' means a snapshot migrations has failed. 'NeedsApproval' means applying to HEAD is desired, but approvals are required first. 'Open' means it is available for users to modify. 'Rejected' means a request to apply with approval was rejected."),
});

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
      outputSchema: CreateChangeSetOutputSchema.shape,
    },
    async ({ changeSetName }): Promise<CallToolResult> => {
      if (!changeSetName) {
        return errorResponse({
          message:
            "Must provide a change set name; ensure you get one from the user!",
        });
      }
      const siApi = new ChangeSetsApi(apiConfig);
      try {
        const response = await siApi.createChangeSet({
          workspaceId: WORKSPACE_ID,
          createChangeSetV1Request: { changeSetName },
        });
        const deepLink = generateChangeSetDeepLink(response.data.changeSet.id);
        return successResponse(
          response.data.changeSet,
          undefined,
          deepLink,
        );
      } catch (error) {
        return errorResponse(error);
      }
    },
  );
}
