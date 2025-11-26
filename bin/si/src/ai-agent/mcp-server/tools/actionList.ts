import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import {
  ActionsApi,
  ChangeSetsApi,
  ComponentsApi,
  SchemasApi,
} from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  findHeadChangeSet,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { type ActionList, ActionSchema } from "../data/actions.ts";

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
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
          });
          const actionList: Array<ActionList> = [];
          for (const action of response.data.actions) {
            // Get the name and schemaName of the component (if there is one)
            let compName: string | undefined = undefined;
            let compSchemaName: string | undefined = undefined;
            if (action.componentId) {
              try {
                const compApi = new ComponentsApi(apiConfig);
                const comp = await compApi.getComponent({
                  workspaceId: WORKSPACE_ID,
                  changeSetId,
                  componentId: action.componentId,
                });
                compName = comp.data.component.name;
                const compSchemaId = comp.data.component.schemaId;
                try {
                  const schemaApi = new SchemasApi(apiConfig);
                  const schema = await schemaApi.getSchema({
                    workspaceId: WORKSPACE_ID,
                    changeSetId,
                    schemaId: compSchemaId,
                  });
                  compSchemaName = schema.data.name;
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
