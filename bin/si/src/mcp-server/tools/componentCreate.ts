import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import { ComponentsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { AttributesSchema } from "../data/components.ts";

const name = "component-create";
const title = "Create a new component for a given schema";
const description =
  `<description>Create a new component for a given schema name, with a name and the attributes provided. Returns the componentName, componentId, schemaName of the component. On failure, returns error details. Always break attribute values down to their end path - *always* prefer /domain/Foo/Bar with a value 'Baz' to setting /domain/Foo to the object '{ Bar: baz }'.</description><usage>Use this tool to create a new component for a given schema in a change set. Use the schema-find tool to understand the paths that are available for setting attributes. For array attributes, replace the [array] in the schema path with a 0 indexed array position - ensure all array entries are accounted for in order (no gaps). For [map], do the same with the string key for the map. To see all of its information after it has been created, use the component-get tool.</usage>`;

const CreateComponentInputSchemaRaw = {
  changeSetId: z.string().describe(
    "The change set to create the component in; components cannot be created on the HEAD change set",
  ),
  schemaName: z.string().describe("the schema name of the component to create"),
  componentName: z.string().describe("the name of the component to create"),
  attributes: AttributesSchema,
  useWorkingCopy: z.boolean().optional().describe(
    "Set this boolean to true in order to create a component using the current working copy of the schema. If the user has been editing the schema of this component, use the working copy. Do not mention schema variants or locked/unlocked to the user. Instead, refer to the unlocked schema variant as the current working copy of the schema.",
  ),
};

const CreateComponentOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    componentId: z.string().describe("the component id"),
    componentName: z.string().describe("the components name"),
    schemaName: z.string().describe("the schema for the component"),
  }),
};
const CreateComponentOutputSchema = z.object(
  CreateComponentOutputSchemaRaw,
);

type CreateComponentResult = z.infer<
  typeof CreateComponentOutputSchema
>["data"];

export function componentCreateTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentCreateResponse",
        CreateComponentOutputSchema,
      ),
      inputSchema: CreateComponentInputSchemaRaw,
      outputSchema: CreateComponentOutputSchemaRaw,
    },
    async (
      { changeSetId, componentName, schemaName, attributes, useWorkingCopy },
    ): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const siApi = new ComponentsApi(apiConfig);
        try {
          const responseCreateComponent = await siApi.createComponent({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            createComponentV1Request: {
              name: componentName,
              schemaName,
              attributes,
              useWorkingCopy,
            },
          });

          const componentId = responseCreateComponent.data.component.id;

          const result: CreateComponentResult = {
            componentId,
            componentName: responseCreateComponent.data.component.name,
            schemaName,
          };

          return successResponse(result);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
