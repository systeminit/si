import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ComponentsApi } from "@systeminit/api-client";
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
  `<description>Find a list of components to upgrade. On failure, returns error details. *Always* ensure that components can be upgraded before trying to upgrade them.</description><usage>Use this tool to upgrade a list of components in a change set. To see all of its information after it has been updated, use the component-get tool.</usage>`;

const UpgradeComponentInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to upgrade the components in; components cannot be upgraded on the HEAD change set.",
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
    async ({ changeSetId }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const siComponentsApi = new ComponentsApi(apiConfig);

        try {
          const upgradableResp = await siComponentsApi.searchComponents({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            searchComponentsV1Request: {
              upgradable: true,
            },
          });

          // Coercing the API response as a List of strings as it's picked up weird in the API!
          const SearchComponentsV1ResponseSchema = z.object({
            components: z.array(z.string()),
          });

          const result: UpgradeComponentResult = {
            success: true,
          };

          if (upgradableResp.status === 200) {
            const parsed = SearchComponentsV1ResponseSchema.parse(
              upgradableResp.data,
            );
            const components = parsed.components;

            for (const componentId of components) {
              console.debug(`Starting upgrade of ${componentId}`);
              await siComponentsApi.upgradeComponent({
                workspaceId: WORKSPACE_ID,
                changeSetId: changeSetId,
                componentId,
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
