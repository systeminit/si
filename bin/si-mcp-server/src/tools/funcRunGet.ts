import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ChangeSetsApi, FuncsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { ChangeSetItem } from "../data/changeSets.ts";
import { decodeBase64 } from "@std/encoding/base64";

const name = "func-run-get";
const title = "Get a function run information";
const description = `<description>Get the information about a function exeuction run. Returns the state of the function run, the componentId and componentName it was for, the schemaName, and the function name, description, kind, arguments, and result - if asked, it will also return the logs and the executed source code. On failure, returns error details</description><usage>Use this tool when the user asks you to work with or troubleshoot a function run - for example, when an action, qualification, or other kind of function as failed.</usage>`;

const GetFuncRunInputSchemaRaw = {
  changeSetId: z
    .string()
    .optional()
    .describe(
      "The change set to look up the schema in; if not provided, HEAD will be used",
    ),
  funcRunId: z.string().describe("the func run id to look up"),
  logs: z
    .boolean()
    .optional()
    .describe(
      "the logs for the function run; logs can be very long, and you should only request them when you are sure you need them for analysis.",
    ),
  code: z
    .boolean()
    .optional()
    .describe(
      "the function body; this can be very long, and you should only request it if you need it for analysis.",
    ),
  args: z
    .boolean()
    .optional()
    .describe(
      "the arguments to the function; these can be very long, and you should only request them if you need them for deep analysis.",
    ),
  result: z
    .boolean()
    .optional()
    .describe(
      "the results of the function; this can be very long, and you should only request it if you need it for analysis.",
    ),
};

const GetFuncRunOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z
    .object({
      funcRunId: z.string().describe("the func run id"),
      funcRunState: z
        .enum([
          "Created",
          "Dispatched",
          "Killed",
          "Running",
          "PostProcessing",
          "Failure",
          "Success",
        ])
        .describe(
          "the state of this function execution run. 'Created' is the initial state, but not yet dispatched to the job system. 'Dispatched' means it is send to the job system. 'Killed' means it has been manually stopped during execution. 'Running' means it is currently executing. 'PostProcessing' means the system is taking the results of the function and appying them to System Initiative. 'Failure' means the function execution has failed. 'Success' means the function has run succesfully (but it does not neccessarily mean that it was successful from the user perspective; it only means the function executed without error.",
        ),
      componentId: z
        .string()
        .describe("the component id this function run was for"),
      componentName: z
        .string()
        .describe("the component name this function run was for"),
      schemaName: z
        .string()
        .describe("the schema name of the component this function was for"),
      functionName: z.string().describe("the name of the function"),
      functionKind: z
        .enum([
          "Action",
          "Attribute",
          "Authentication",
          "CodeGeneration",
          "Intrinsic",
          "Qualification",
          "SchemaVariantDefinition",
          "Unknown",
          "Management",
        ])
        .describe(
          "'Action' means an action function on a component; 'Attribute' means an attribute function on a components attribute; 'CodeGeneration' means a code generation function on a component; 'Intrinsic' means it is not a javascript function, but instead implemented directly by SI; 'Qualification' means a qualification that a component is valid; 'SchemaVariantDefinition' means the typescript that defines the schema for a component; 'Unknown' means a function type that is not yet known to the system; 'Management' means a management function on a component (like import or discover.)",
        ),
      args: z
        .string()
        .optional()
        .describe(
          "A JSON string representing the arguments passed as input to this function",
        ),
      resultValue: z
        .string()
        .optional()
        .describe(
          "A JSON string representing the return value of the function",
        ),
      logs: z
        .string()
        .optional()
        .describe(
          "A string of logs produced by the function; only included if the logs argument to the tool is true",
        ),
      code: z
        .string()
        .optional()
        .describe(
          "The source code executed for this func run; useful for troubleshooting. Only included if the code argument to the tool is true",
        ),
    })
    .describe("the func run data"),
};
const GetFuncRunOutputSchema = z.object(GetFuncRunOutputSchemaRaw);

type FuncRunResult = z.infer<typeof GetFuncRunOutputSchema>["data"];

export function funcRunGetTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "funcRunGetResponse",
        GetFuncRunOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      inputSchema: GetFuncRunInputSchemaRaw,
      outputSchema: GetFuncRunOutputSchemaRaw,
    },
    async ({
      changeSetId,
      funcRunId,
      logs,
      code,
      args: showArguments,
      result: showResult,
    }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        if (!changeSetId) {
          const changeSetsApi = new ChangeSetsApi(apiConfig);
          try {
            const changeSetList = await changeSetsApi.listChangeSets({
              workspaceId: WORKSPACE_ID,
            });
            const head = (
              changeSetList.data.changeSets as ChangeSetItem[]
            ).find((cs) => cs.isHead);
            if (!head) {
              return errorResponse({
                message:
                  "No HEAD change set found; this is a bug! Tell the user we are sorry.",
              });
            }
            changeSetId = head.id;
          } catch (error) {
            const errorMessage =
              error instanceof Error ? error.message : String(error);
            return errorResponse({
              message: `No change set id was provided, and we could not find HEAD; this is a bug! Tell the user we are sorry: ${errorMessage}`,
            });
          }
        }

        const siApi = new FuncsApi(apiConfig);
        try {
          const response = await siApi.getFuncRun({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            funcRunId: funcRunId,
          });
          const result: FuncRunResult = {
            funcRunId: response.data.funcRun.id,
            funcRunState: response.data.funcRun
              .state as FuncRunResult["funcRunState"],
            componentId: response.data.funcRun.componentId,
            componentName: response.data.funcRun.componentName,
            schemaName: response.data.funcRun.schemaName,
            functionName:
              response.data.funcRun.functionDisplayName ||
              response.data.funcRun.functionName,
            functionKind: response.data.funcRun
              .functionKind as FuncRunResult["functionKind"],
          };
          if (showArguments && response.data.funcRun.functionArgs) {
            result.args = JSON.stringify(response.data.funcRun.functionArgs);
          }
          if (showResult && response.data.funcRun.resultValue) {
            result.resultValue = JSON.stringify(
              response.data.funcRun.resultValue,
            );
          }

          if (logs && response.data.funcRun.logs?.logs) {
            let logOutput = "";
            for (const logLine of response.data.funcRun.logs.logs) {
              logOutput += (logLine as { message: string }).message;
            }
            result.logs = logOutput;
          }
          if (code && response.data.funcRun.functionCodeBase64) {
            const codeString = decodeBase64(
              response.data.funcRun.functionCodeBase64,
            );
            const textDecoder = new TextDecoder();
            result.code = textDecoder.decode(codeString);
          }

          return successResponse(result);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
