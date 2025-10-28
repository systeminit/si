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
  findHeadChangeSet,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const name = "component-list";
const title = "List components";
const description = `
  <important>
      *DO NOT USE THIS TOOL FOR TEMPLATE GENERATION:*
      - For template generation, use the template-generate tool's built-in search functionality
      - Only use component-list when you need to retrieve a comprehensive list of ALL components or apply
    complex regex filtering that isn't supported by search syntax
      </important>
      <description>Lists all components. Returns an array of components with componentId, component name, and
    the schema name. On failure, returns error details</description>
      <usage>Use this tool ONLY when you need to:
      1. Get a complete inventory of all components in a change set
      2. Apply complex regex-based filtering not available in search syntax
      3. Perform operations that require the full component list

      DO NOT use this tool for finding components to include in templates - use template-generate's search
    instead.
      </usage>  `;

const ListComponentsInputSchemaRaw = {
  changeSetId: z.string().optional().describe(
    "The change set to look up components in; if not provided, HEAD will be used",
  ),
  filters: z.object({
    logic: z.enum(["AND", "OR"]).optional().describe(
      "Logic operator between filter groups (default: AND)",
    ),
    filterGroups: z.array(
      z.object({
        responseField: z.enum(["componentName", "componentId", "schemaName"])
          .describe("the response field to filter on"),
        logic: z.enum(["AND", "OR"]).optional().describe(
          "Logic operator between regular expressions within this filter group (default: OR)",
        ),
        regularExpressions: z.array(
          z.string().describe("a javascript regular expression string"),
        ).describe(
          "an array of javascript compatible regular expression strings",
        ),
      }).describe(
        "a filter group, consisting of a responseField to filter and an array of regularExpressions",
      ),
    ).describe("an array of filter groups"),
  }).optional().describe(
    "filtering configuration with configurable AND/OR logic both between filter groups and within each group's regular expressions",
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
    async ({ changeSetId, filters }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        if (!changeSetId) {
          const changeSetsApi = new ChangeSetsApi(apiConfig);
          const headChangeSet = await findHeadChangeSet(changeSetsApi, false);
          if (headChangeSet.changeSetId) {
            changeSetId = headChangeSet.changeSetId;
          } else {
            return errorResponse(headChangeSet);
          }
        }
        try {
          const response = await listAllComponents(apiConfig, changeSetId);
          const filteredResponse = applyFilters(response, filters);
          return successResponse(
            filteredResponse,
          );
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
