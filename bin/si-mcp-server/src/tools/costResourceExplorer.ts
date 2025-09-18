import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ChangeSetsApi } from "@systeminit/api-client";
import { listAllComponents } from "./componentList.ts";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { ChangeSet } from "../data/changeSets.ts";

const name = "cost-explorer";
const title = "Generate a cost report for a change set";
const description = `<description>Forecast the resource price by change set and resource. The default forecasting is the monthly cost. If users ask for the weekly cost, you must inform them. Only include resources inside the current change set, if the current change set is empty let users know there’s nothing to forecast. You must inform users the name of the change set you’re forecasting, including HEAD. In case you can not forecast, let users know the reason and what they need to do make your mission successful. Returns the 'success' on successful creation of cost report. On failure, returns error details.</description><usage>Use this tool to forecast the cost of a change set. Show the monthly cost. Ask if users want to see the weekly cost. Ask if users want to see the cost forecasting break by resource.</usage>`;

const CostExplorerInputSchemaRaw = {
  changeSetId: z
    .string()
    .optional()
    .describe(
      "The change set to generate a cost report for; if no change set chosen, the HEAD change set will be used",
    ),
};

const CostExpolerOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z
    .object({
      totalMonthlyUSD: z.any().describe("The total monthly cost in USD"),
      detailedReport: z.any().describe("The detailed report for the analysis"),
    })
    .describe("the information for the cost report"),
};

const CostExlorerOutputSchema = z.object(CostExpolerOutputSchemaRaw);

type CostExplorerResult = z.infer<typeof CostExlorerOutputSchema>["data"];

export function costExplorerTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "costExplorer",
        CostExlorerOutputSchema,
      ),
      inputSchema: CostExplorerInputSchemaRaw,
      outputSchema: CostExpolerOutputSchemaRaw,
    },
    async ({ changeSetId }): Promise<CallToolResult> => {
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

        const componentsWithCode = await listAllComponents(
          apiConfig,
          changeSetId,
          undefined,
          undefined,
          true,
        );

        const codegens: Record<string, unknown> = {};
        for (const componentWithCode of componentsWithCode) {
          if (componentWithCode.codegen) {
            const code = JSON.parse(componentWithCode.codegen.code);
            const resources = code.Resources;
            const rKey = Object.keys(resources)[0];
            const data = resources[rKey];
            codegens[componentWithCode.componentName] = data;
          }
        }

        const cloudformationDoc = {
          AWSTemplateFormatVersion: "2010-09-09",
          Resources: codegens,
        };
        await Deno.writeTextFile(
          "/tmp/template.json",
          JSON.stringify(cloudformationDoc),
        );

        const config = `version: 0.1
projects:
  - name: my-stacks
    path: /tmp/template.json`;
        await Deno.writeTextFile("/tmp/config.yml", config);

        const cmd = new Deno.Command("infracost", {
          args: [
            "breakdown",
            "--config-file",
            "/tmp/config.yml",
            "--format",
            "json",
            "--log-level",
            "debug",
          ],
          env: { ...Deno.env.toObject() },
        });
        const { code, stdout, stderr } = await cmd.output();

        if (code !== 0) throw new Error(new TextDecoder().decode(stderr));
        const output = new TextDecoder().decode(stdout);
        console.warn(output);
        const report = JSON.parse(output);

        const total = report.projects?.reduce(
          (sum: number, p: any) => sum + (p.breakdown?.totalMonthlyCost || 0),
          0,
        );
        // console.warn({ totalMonthlyUSD: total, detailed: report });

        const result: CostExplorerResult = {
          totalMonthlyUSD: total,
          detailedReport: report,
        };

        return successResponse(result);
      });
    },
  );
}
