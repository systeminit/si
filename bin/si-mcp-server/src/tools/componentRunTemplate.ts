import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ComponentsApi, ManagementFuncsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const name = "component-run-template";
const title = "Run a template from a template component";
const description =
  `<description>Find the template schema by name if it does not exist. Execute the Run Template management function on a template component to run/instantiate the template and create new components based on the template definition. This tool is used to run templates that were previously generated using component-generate-template or in the template category. The template execution will create new components according to the template's definition.</description><usage>Use this tool to run a template component by executing its Run Template management function. Provide the schema name of the template component (typically obtained from component-generate-template or schema-find). The tool executes the Run Template management function and wait for completion, returning execution status and results.</usage>`;

const ComponentRunTemplateInputSchemaRaw = {
  changeSetId: z.string().describe(
    "The change set to run the template in",
  ),
  componentId: z.string().describe(
    "The component ID of the template component to run",
  ),
};

const ComponentRunTemplateOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    managementFuncJobStateId: z.string().describe(
      "The job state ID for the executed management function",
    ),
    executionStatus: z.string().describe(
      "The final status of the management function execution",
    ),
    message: z.string().optional().describe(
      "Optional message from the management function execution",
    ),
    funcRunId: z.string().describe(
      "The function run ID from the management function execution"
    ),
  }).describe(
    "Information about the template execution including job state and results",
  ),
};

const ComponentRunTemplateOutputSchema = z.object(
  ComponentRunTemplateOutputSchemaRaw,
);

type ComponentRunTemplateResult = z.infer<
  typeof ComponentRunTemplateOutputSchema
>["data"];

export function componentRunTemplateTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentRunTemplate",
        ComponentRunTemplateOutputSchema,
      ),
      inputSchema: ComponentRunTemplateInputSchemaRaw,
      outputSchema: ComponentRunTemplateOutputSchemaRaw,
    },
    async (
      { changeSetId, componentId, functionName },
    ): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const siComponentsApi = new ComponentsApi(apiConfig);
        const mgmtApi = new ManagementFuncsApi(apiConfig);

        try {
          // Determine the function name to use - templates use the name "Run Template"
          const managementFunction = { function: "Run Template" }; // Standard default for template management functions

          // Execute the management function
          const executeResponse = await siComponentsApi.executeManagementFunction({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            componentId: componentId,
            executeManagementFunctionV1Request: {
              managementFunction,
            },
          });

          // Wait for the management function to complete
          let executionState = "Pending";
          const retrySleepInMs = 1000;
          const retryMaxCount = 120; // 2 minutes max wait time
          let currentCount = 0;

          let finalResult: any = null;

          while (
            (executionState === "Pending" ||
              executionState === "Executing" ||
              executionState === "Operating") &&
            currentCount <= retryMaxCount
          ) {
            if (currentCount !== 0) {
              await new Promise((resolve) => setTimeout(resolve, retrySleepInMs));
            }

            const statusResponse = await mgmtApi.getManagementFuncRunState({
              workspaceId: WORKSPACE_ID,
              changeSetId: changeSetId,
              managementFuncJobStateId: executeResponse.data.managementFuncJobStateId,
            });

            executionState = statusResponse.data.state;
            finalResult = statusResponse.data;
            currentCount++;
          }

          // Check if execution completed
          if (currentCount > retryMaxCount) {
            const result: ComponentRunTemplateResult = {
              managementFuncJobStateId: executeResponse.data.managementFuncJobStateId,
              executionStatus: executionState,
              message: executeResponse.data.message,
              funcRunId: finalResult?.funcRunId || "",
            };

            return successResponse(
              result,
              "The template execution is still in progress; use the funcRunId to find out more",
            );
          } else if (executionState === "Failure") {
            return errorResponse(
              new Error(
                `Template execution failed: ${executeResponse.data.message || "Unknown error"}`,
              ),
            );
          } else {
            const result: ComponentRunTemplateResult = {
              managementFuncJobStateId: executeResponse.data.managementFuncJobStateId,
              executionStatus: executionState,
              message: executeResponse.data.message,
              funcRunId: finalResult?.funcRunId || "",
            };

            return successResponse(result);
          }
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
