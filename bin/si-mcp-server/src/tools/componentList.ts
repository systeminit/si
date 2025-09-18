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
  withAnalytics,
} from "./commonBehavior.ts";
import { ChangeSet } from "../data/changeSets.ts";

const name = "component-list";
const title = "List components";
const description = `<description>Lists all components. Returns an array of components with componentId, component name, and the schema name. On failure, returns error details</description><usage>Use this tool to understand what components are present in a change set in the workspace, and to find their componentId or schemaName in order to work with them.</usage>`;

const ListComponentsInputSchemaRaw = {
  changeSetId: z
    .string()
    .optional()
    .describe(
      "The change set to look up components in; if not provided, HEAD will be used",
    ),
  filters: z
    .object({
      logic: z
        .enum(["AND", "OR"])
        .optional()
        .describe("Logic operator between filter groups (default: AND)"),
      filterGroups: z
        .array(
          z
            .object({
              responseField: z
                .enum(["componentName", "componentId", "schemaName"])
                .describe("the response field to filter on"),
              logic: z
                .enum(["AND", "OR"])
                .optional()
                .describe(
                  "Logic operator between regular expressions within this filter group (default: OR)",
                ),
              regularExpressions: z
                .array(
                  z.string().describe("a javascript regular expression string"),
                )
                .describe(
                  "an array of javascript compatible regular expression strings",
                ),
            })
            .describe(
              "a filter group, consisting of a responseField to filter and an array of regularExpressions",
            ),
        )
        .describe("an array of filter groups"),
    })
    .optional()
    .describe(
      "filtering configuration with configurable AND/OR logic both between filter groups and within each group's regular expressions",
    ),
};

const ListComponentsOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z
    .array(
      z
        .object({
          componentId: z.string().describe("the component id"),
          componentName: z.string().describe("the component name"),
          schemaName: z.string().describe("the schema name for the component"),
          codegen: z
            .any()
            .optional()
            .describe("the codegen for the cloudformation for the resource"),
        })
        .describe("an individual component"),
    )
    .describe("the list of components"),
};
const ListComponentsOutputSchema = z.object(ListComponentsOutputSchemaRaw);
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
    async ({ changeSetId, filters }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
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
              message: `No change set id was provided, and we could not find HEAD; this is a bug! Tell the user we are sorry: ${
                error instanceof Error ? error.message : String(error)
              }`,
            });
          }
        }
        try {
          const response = await listAllComponents(apiConfig, changeSetId);
          const filteredResponse = applyFilters(response, filters);
          return successResponse(filteredResponse);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}

export function applyFilters(
  components: Array<ListComponents["data"][number]>,
  filters?: {
    logic?: "AND" | "OR";
    filterGroups: Array<{
      responseField: "componentName" | "componentId" | "schemaName";
      logic?: "AND" | "OR";
      regularExpressions: string[];
    }>;
  },
): Array<ListComponents["data"][number]> {
  if (!filters || !filters.filterGroups || filters.filterGroups.length === 0) {
    return components;
  }

  const betweenGroupsLogic = filters.logic || "AND";

  return components.filter((component) => {
    const groupResults = filters.filterGroups.map((filterGroup) => {
      const fieldValue = component[filterGroup.responseField];
      const withinGroupLogic = filterGroup.logic || "OR";

      const regexResults = filterGroup.regularExpressions.map((regexStr) => {
        try {
          const regex = new RegExp(regexStr);
          return regex.test(fieldValue);
        } catch (error) {
          // If regex is invalid, skip this regex
          console.warn(`Invalid regex pattern: ${regexStr}`, error);
          return false;
        }
      });

      // Apply logic within the filter group
      if (withinGroupLogic === "AND") {
        return regexResults.every((result) => result);
      } else {
        return regexResults.some((result) => result);
      }
    });

    // Apply logic between filter groups
    if (betweenGroupsLogic === "AND") {
      return groupResults.every((result) => result);
    } else {
      return groupResults.some((result) => result);
    }
  });
}

export async function listAllComponents(
  apiConfig: Configuration,
  changeSetId: string,
  cursor?: string,
  componentList?: Array<ListComponents["data"][number]>,
  withCodegen?: boolean,
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
      ...(withCodegen && { includeCodegen: true }),
    };
  } else {
    args = {
      workspaceId: WORKSPACE_ID,
      changeSetId: changeSetId,
      limit: "300",
      ...(withCodegen && { includeCodegen: true }),
    };
  }

  const response = await siApi.listComponents(args);
  for (const component of response.data.componentDetails) {
    componentList.push({
      componentId: component.componentId,
      componentName: component.name,
      schemaName: component.schemaName,
      codegen: component.codegen,
    });
  }
  if (response.data.nextCursor) {
    componentList = await listAllComponents(
      apiConfig,
      changeSetId,
      response.data.nextCursor,
      componentList,
      withCodegen,
    );
  }
  return componentList;
}
