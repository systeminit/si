import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import {
  ChangeSetsApi,
  ComponentsApi,
  ComponentsApiListComponentsRequest,
  Configuration,
} from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
} from "./commonBehavior.ts";
import { ChangeSet } from "../data/changeSets.ts";

const name = "component-list";
const title = "List components";
const description =
  `<description>Lists all components. Returns an array of components with componentId, component name, and the schema name. On failure, returns error details</description><usage>Use this tool to understand what components are present in a change set in the workspace, and to find their componentId or schemaName in order to work with them.</usage>`;

const ListComponentsInputSchemaRaw = {
  changeSetId: z.string().optional().describe(
    "The change set to look up components in; if not provided, HEAD will be used",
  ),
};

const ListComponentsOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.array(
    z.object({
      componentId: z.string().describe("the component id"),
      componentName: z.string().describe("the component name"),
      schemaName: z.string().describe("the schema name for the component"),
    }).describe("an individual component"),
  )
    .describe("the list of components"),
};
const ListComponentsOutputSchema = z.object(
  ListComponentsOutputSchemaRaw,
);
type ListComponents = z.infer<typeof ListComponentsOutputSchema>;

export function componentListTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentListResponse",
        ListComponentsOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      inputSchema: ListComponentsInputSchemaRaw,
      outputSchema: ListComponentsOutputSchemaRaw,
    },
    async ({ changeSetId }): Promise<CallToolResult> => {
      if (!changeSetId) {
        const changeSetsApi = new ChangeSetsApi(apiConfig);
        try {
          const changeSetList = await changeSetsApi.listChangeSets({
            workspaceId: WORKSPACE_ID,
          });
          const changeSets = changeSetList.data.changeSets as ChangeSet[];
          const head = changeSets.find((cs) => cs.isHead);
          if (!head) {
            return errorResponse({
              message: "Could not find HEAD change set",
            });
          }
          changeSetId = head.id;
        } catch (error) {
          return errorResponse({
            message:
              `No change set id was provided, and we could not find HEAD; this is a bug! Tell the user we are sorry: ${
                error instanceof Error ? error.message : String(error)
              }`,
          });
        }
      }
      try {
        const response = await listAllComponents(apiConfig, changeSetId);
        return successResponse(
          response,
        );
      } catch (error) {
        return errorResponse(error);
      }
    },
  );
}

async function listAllComponents(
  apiConfig: Configuration,
  changeSetId: string,
  cursor?: string,
  componentList?: Array<ListComponents["data"][number]>,
): Promise<Array<ListComponents["data"][number]>> {
  if (!componentList) {
    componentList = [];
  }
  const siApi = new ComponentsApi(apiConfig);
  let args: ComponentsApiListComponentsRequest;
  if (cursor) {
    args = {
      workspaceId: WORKSPACE_ID,
      changeSetId: changeSetId,
      limit: "300",
      cursor,
    };
  } else {
    args = {
      workspaceId: WORKSPACE_ID,
      changeSetId: changeSetId,
      limit: "300",
    };
  }

  const response = await siApi.listComponents(args);
  for (const component of response.data.componentDetails) {
    componentList.push({
      componentId: component.componentId,
      componentName: component.name,
      schemaName: component.schemaName,
    });
  }
  if (response.data.nextCursor) {
    componentList = await listAllComponents(
      apiConfig,
      changeSetId,
      response.data.nextCursor,
      componentList,
    );
  }
  return componentList;
}
