import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import {
  DebugFuncsApi,
  type DebugFuncsApiExecDebugFuncRequest,
  type DebugFuncsApiGetDebugFuncStateRequest,
} from "@systeminit/api-client";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";

const name = "exec-debug-func";
const title = "Run a debug func inside a change set";
const description =
  `<description>Runs a one-off function inside a change set intended to troubleshoot problems building infrastructure in the cloud. Returns arbitrary json data that should provide information about problems encountered when building infrastructure. Requires a component that has credentials and other settings necessary for making API calls within the cloud environment. The component SHOULD be connected to READ ONLY CREDENTIALS to prevent making changes to infrastructure. Optionally accepts arbitrary JSON debugInput with any additional context required for troubleshooting the problem.</description>`;

const debugFuncDescription =
  "<description>A typescript debug function. The function should *ALWAYS* have the form of `async function debug({ component, debugInput }) { /* debug code here */ }`. The function name should *ALWAYS* be `debug`</description>";

const execDebugFuncInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe("The id of the change set in which this function should be run"),
  componentId: z
    .string()
    .describe(
      "The id of the component that is connected to the necessary credentials and other context for making the API calls needed for debugging.",
    ),
  debugInput: z
    .any()
    .optional()
    .describe(
      "Arbitrary JSON object input that will be passed to the debug function",
    ),
  debugFunc: z.string().describe(debugFuncDescription),
};

const execDebugFuncInputSchema = z.object(execDebugFuncInputSchemaRaw);

type DebugFuncInputSchema = z.infer<typeof execDebugFuncInputSchema>;

const execDebugFuncOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is `failure`, the error message will contain information about what went wrong",
    ),
  output: z.any().describe("The debug output produced by the debug function"),
  funcRunId: z
    .string()
    .optional()
    .describe(
      "The func run id of the function, to be passed to the func-run-get tool if more details are necessary, such as the function logs.",
    ),
};

const execDebugFuncOutputSchema = z.object(execDebugFuncOutputSchemaRaw);

const POLL_WAIT_START_MS = 100;
const MAX_BASE_WAIT_MS = 5000;
const MAX_JITTERED_WAIT_MS = 15000;
const MAX_POLLS = 1000;
const JITTER = 0.3; // 30% jitter

const SUCCESS = "Success";
const FAILURE = "Failure";

// Honk, shoo, mimimimimi
const zzz = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export function execDebugFunc(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "execDebugFuncResponse",
        execDebugFuncOutputSchema,
      ),
      inputSchema: execDebugFuncInputSchemaRaw,
      outputSchema: execDebugFuncOutputSchemaRaw,
    },
    async ({
      changeSetId,
      componentId,
      debugInput,
      debugFunc,
    }: DebugFuncInputSchema): Promise<CallToolResult> =>
      await withAnalytics(name, async () => {
        const debugFuncPattern = /async\s+function\s+debug/;
        if (!debugFuncPattern.test(debugFunc)) {
          return errorResponse({
            message:
              "Debug function must have the form 'async function debug({ component, debugInput }) { ... }'",
          });
        }

        const debugFuncsApi = new DebugFuncsApi(apiConfig);
        const execFuncRequest: DebugFuncsApiExecDebugFuncRequest = {
          workspaceId: WORKSPACE_ID!,
          changeSetId,
          execDebugFuncV1Request: {
            code: debugFunc,
            name: `debug-func-${componentId}-${changeSetId}`,
            handler: "debug",
            componentId,
            debugInput,
          },
        };

        // Wait for the function to have a chance to run
        await zzz(POLL_WAIT_START_MS);

        try {
          const execResponse = await debugFuncsApi.execDebugFunc(
            execFuncRequest,
          );
          const debugFuncJobStateId = execResponse.data.debugFuncJobStateId;

          let pollCount = 0;
          let currentWaitMs = POLL_WAIT_START_MS;
          let funcRunId: string | null | undefined;

          while (pollCount < MAX_POLLS) {
            const jobStateRequest: DebugFuncsApiGetDebugFuncStateRequest = {
              workspaceId: WORKSPACE_ID!,
              changeSetId,
              debugFuncJobStateId,
            };

            const { data } = await debugFuncsApi.getDebugFuncState(
              jobStateRequest,
            );
            funcRunId = data.funcRunId;

            if (data.state === SUCCESS || data.state === FAILURE) {
              const errorMessage = data.state === FAILURE
                ? data.failure || "Debug function execution failed"
                : undefined;
              return successResponse({
                status: data.state,
                output: data.result,
                errorMessage,
                funcRunId,
              });
            }

            pollCount++;

            if (pollCount >= MAX_POLLS) {
              const message = funcRunId
                ? `Function execution timed out but you can check for results or logs with the FuncRunId: ${funcRunId}`
                : "Function execution timed out";
              return errorResponse({
                message,
              });
            }

            const jitter = Math.random() * JITTER;
            const waitMs = Math.min(
              currentWaitMs * (1 + jitter),
              MAX_JITTERED_WAIT_MS,
            );
            await zzz(waitMs);

            currentWaitMs = Math.min(currentWaitMs * 2, MAX_BASE_WAIT_MS);
          }

          return errorResponse({
            message: `Function execution timed out after ${MAX_POLLS} polls`,
          });
        } catch (error) {
          return errorResponse({
            message: `Failed to execute debug function: ${error}`,
          });
        }
      }),
  );
}
