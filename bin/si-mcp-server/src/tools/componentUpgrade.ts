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
import _ from "lodash";

const name = "component-upgrade";
const title = "Upgrade a component in a change set";
const description = `<description>Upgrades a component in a change set. Returns success if component is upgraded. On failure, returns error details</description><usage>*Always* check that a component can be upgraded by using the 'get-component' tool first. Use this tool when the user wants to process the upgrade a component.</usage>`;

const UpgradeComponentInputSchemaRaw = {
  changeSetId: z.string().describe("The change set to upgrade the component"),
  componentId: z.string().describe("the component to upgrade"),
};

const UpgradeComponentOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({
    success: z.boolean().describe("a successful upgrade"),
  }),
};
const UpgradeComponentOutputSchema = z.object(UpgradeComponentOutputSchemaRaw);

type UpgradeComponentResult = z.infer<
  typeof UpgradeComponentOutputSchema
>["data"];

export function componentUpgradeTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentUpgradeResponse",
        UpgradeComponentOutputSchema,
      ),
      inputSchema: UpgradeComponentInputSchemaRaw,
      outputSchema: UpgradeComponentOutputSchemaRaw,
    },
    async ({ changeSetId, componentId }): Promise<CallToolResult> => {
      const siApi = new ComponentsApi(apiConfig);
      try {
        await siApi.upgradeComponent({
          workspaceId: WORKSPACE_ID,
          changeSetId: changeSetId,
          componentId,
        });
        const result: UpgradeComponentResult = {
          success: true,
        };

        return successResponse(result);
      } catch (error) {
        return errorResponse(error);
      }
    },
  );
}
