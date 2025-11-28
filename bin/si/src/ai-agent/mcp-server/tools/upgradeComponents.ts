import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import { ComponentsApi, SearchApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const name = "upgrade-components";
const title = "Upgrade a list of components";
const description =
  `<description>Find a list of components to upgrade. You can filter the list of components to upgrade using a schema category, e.g. AWS::EC2. On failure, returns error details. *Always* ensure that components can be upgraded before trying to upgrade them.</description><usage>Use this tool to upgrade a list of components in a change set. You can filter the components to upgrade by passing a schema category. A schema category is in the form provider::service, e.g AWS::EC2. To see all of its information after it has been updated, use the component-get tool.</usage>`;

const UpgradeComponentInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to upgrade the components in; components cannot be upgraded on the HEAD change set.",
    ),
  schemaCategory: z
    .string()
    .optional()
    .describe(
      "An optional schema category, e.g. AWS::EC2, that can be used to filter the set of components to upgrade",
    ),
};

const UpgradeComponentOutputSchemaRaw = {
  status: z
    .enum(["success", "failure"])
    .describe(
      "success when components are successfully upgraded, failure when an error occurs during upgrades",
    ),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({
    success: z.boolean().describe("a successful set of upgrades"),
  }),
};

const UpgradeComponentOutputSchema = z.object(UpgradeComponentOutputSchemaRaw);

type UpgradeComponentResult = z.infer<
  typeof UpgradeComponentOutputSchema
>["data"];

export function upgradeComponentsTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "upgradeComponents",
        UpgradeComponentOutputSchema,
      ),
      inputSchema: UpgradeComponentInputSchemaRaw,
      outputSchema: UpgradeComponentOutputSchemaRaw,
    },
    async ({ changeSetId, schemaCategory }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const siComponentsApi = new ComponentsApi(apiConfig);
        const siSearchApi = new SearchApi(apiConfig);

        try {
          const searchString = [
            "isUpgradable:true",
            schemaCategory && `category:${schemaCategory}`,
          ].filter(Boolean).join(" ");

          const upgradableResp = await siSearchApi.search({
            workspaceId: WORKSPACE_ID!,
            changeSetId: changeSetId,
            q: searchString,
          });

          const result: UpgradeComponentResult = {
            success: true,
          };

          if (upgradableResp.status === 200) {
            const components = upgradableResp.data.components;

            for (const component of components) {
              console.debug(`Starting upgrade of ${component.name}`);
              await siComponentsApi.upgradeComponent({
                workspaceId: WORKSPACE_ID!,
                changeSetId: changeSetId,
                componentId: component.id,
              });
            }
            return successResponse(result);
          }
          return successResponse(result);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
