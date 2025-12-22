import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import {
  ActionsApi,
  ChangeSetsApi,
  ComponentsApi,
  SchemasApi,
} from "@systeminit/api-client";
import type {
  GetComponentV1Response,
  GetSchemaV1Response,
} from "@systeminit/api-client";
import {
  errorResponse,
  findHeadChangeSet,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { type ActionList, ActionSchema } from "../data/actions.ts";
import { cache, generateCacheKey } from "../cache.ts";
import { Context } from "../../../context.ts";

const name = "action-list";
const title = "List actions";
const description =
  `<description>Lists all actions. Returns an array of actions with actionId, componentId, Name, kind, and state. On failure, returns error details</description><usage>Use this tool when the user asks what actions are present.</usage>`;

const ListActionsInputSchemaRaw = {
  changeSetId: z
    .string()
    .optional()
    .describe(
      "The change set to look up actions in; if not provided, HEAD will be used",
    ),
};

const ListActionsOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.array(ActionSchema).optional().describe("The list of actions"),
};
const ListActionsOutputSchema = z.object(ListActionsOutputSchemaRaw);

export function actionListTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "actionsListResponse",
        ListActionsOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      inputSchema: ListActionsInputSchemaRaw,
      outputSchema: ListActionsOutputSchemaRaw,
    },
    async ({ changeSetId }): Promise<CallToolResult> => {
      const apiConfig = Context.apiConfig();
      const workspaceId = Context.workspaceId();

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
        const siApi = new ActionsApi(apiConfig);
        try {
          const response = await siApi.getActions({
            workspaceId,
            changeSetId: changeSetId,
          });
          const actionList: Array<ActionList> = [];
          for (const action of response.data.actions) {
            // Get the name and schemaName of the component (if there is one)
            let compName: string | undefined = undefined;
            let compSchemaName: string | undefined = undefined;
            if (action.componentId) {
              try {
                // Try cache first for component
                const componentCacheKey = generateCacheKey(
                  "component",
                  action.componentId,
                  changeSetId,
                );
                let componentData = cache.get<GetComponentV1Response>(
                  componentCacheKey,
                  changeSetId,
                );

                if (!componentData) {
                  // Cache miss - fetch component from API
                  const compApi = new ComponentsApi(apiConfig);
                  const comp = await compApi.getComponent({
                    workspaceId,
                    changeSetId,
                    componentId: action.componentId,
                  });
                  componentData = comp.data;
                  // Cache the result
                  cache.set(componentCacheKey, componentData, changeSetId);
                }

                compName = componentData.component.name;
                const compSchemaId = componentData.component.schemaId;

                try {
                  // Try cache first for schema
                  const schemaCacheKey = generateCacheKey(
                    "schema",
                    compSchemaId,
                    changeSetId,
                  );
                  let schemaData = cache.get<GetSchemaV1Response>(
                    schemaCacheKey,
                    changeSetId,
                  );

                  if (!schemaData) {
                    // Cache miss - fetch schema from API
                    const schemaApi = new SchemasApi(apiConfig);
                    const schema = await schemaApi.getSchema({
                      workspaceId,
                      changeSetId,
                      schemaId: compSchemaId,
                    });
                    schemaData = schema.data;
                    // Cache the result
                    cache.set(schemaCacheKey, schemaData, changeSetId);
                  }

                  compSchemaName = schemaData.name;
                } catch (error) {
                  return errorResponse({
                    message:
                      `Error getting schema for extra details while listing actions; bug! ${error}`,
                  });
                }
              } catch (error) {
                return errorResponse({
                  message:
                    `Error getting component for extra details while listing actions; bug! ${error}`,
                });
              }
            }
            actionList.push({
              actionId: action.id,
              componentId: action.componentId,
              componentName: compName,
              schemaName: compSchemaName,
              name: action.name,
              kind: action.kind,
              funcRunId: action.funcRunId,
              state: action.state as ActionList["state"],
            });
          }
          return successResponse(
            actionList,
            "'Failed' actions can be retried, and you can learn more about the function execution ysing the func-run-get tool with the funcRunId",
          );
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
