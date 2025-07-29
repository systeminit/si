import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ComponentsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
} from "./commonBehavior.ts";
import { AttributesSchema } from "../data/components.ts";

const name = "component-update";
const title = "Update a component";
const description =
  `<description>Update a component for a given componentId. Update only the attributes that need to be changed; existing attributes will remain. Returns the 'success' on successful update. On failure, returns error details. Always break attribute values down to their end path - *always* prefer /domain/Foo/Bar with a value 'Baz' to setting /domain/Foo to the object '{ Bar: baz }'.</description><usage>Use this tool to update a in a change set. Use the schema-find tool to understand the paths that are available for setting attributes. For array attributes, replace the [array] in the schema path with a 0 indexed array position - ensure all array entries are accounted for in order (no gaps). For [map], do the same with the string key for the map. To see all of its information after it has been updated, use the component-get tool.</usage>`;

const UpdateComponentInputSchemaRaw = {
  changeSetId: z.string().describe(
    "The change set to update the component in; components cannot be created on the HEAD change set",
  ),
  componentId: z.string().describe("the component id to update"),
  attributes: AttributesSchema,
};

const UpdateComponentOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    success: z.boolean().describe("a successful update"),
  }),
};
const UpdateComponentOutputSchema = z.object(
  UpdateComponentOutputSchemaRaw,
);

type UpdateComponentResult = z.infer<typeof UpdateComponentOutputSchema>["data"];

export function componentUpdateTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentCreateResponse",
        UpdateComponentOutputSchema,
      ),
      inputSchema: UpdateComponentInputSchemaRaw,
      outputSchema: UpdateComponentOutputSchemaRaw,
    },
    async ({ changeSetId, attributes, componentId }): Promise<CallToolResult> => {
      const siApi = new ComponentsApi(apiConfig);
      try {
        await siApi.updateComponent({
          workspaceId: WORKSPACE_ID,
          changeSetId: changeSetId,
          componentId,
          updateComponentV1Request: {
            attributes,
          }
        });
        const result: UpdateComponentResult = {
          success: true,
        };

        return successResponse(
          result,
        );
      } catch (error) {
        return errorResponse(error);
      }
    },
  );
}

