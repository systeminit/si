import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { SchemasApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const name = "func-upsert";
const title = "Create or update a function for an existing schema.";
const description = `
<description>
Create a new function of the given type for a given schema.
</description>
<usage>
  Use this tool to create or update a function. Follow this workflow:
  1. VALIDATION PHASE (REQUIRED):
     - If the user provides functionCode, you MUST first analyze it against the requested
  functionType and check that it is valid TypeScript code.
     - Check the function signature matches the expected type (see examples in functionCode
  parameter)
     - Verify return types match: qualification returns {result, message}, codegen returns
  {format, code}, etc.
     - If code doesn't match the requested type, STOP and inform the user of the mismatch
     - NEVER proceed with tool execution if code validation fails

  2. LOOKUP PHASE:
     - If user only gives schema name, use schema-find tool to get schemaId
     - If user doesn't specify functionType, ask them which type
     - If functionType is "action" but no actionKind given, ask which kind

  3. EXECUTION PHASE:
     - Only after validation passes, call this tool
     - Do not mention schemaVariantId to the user

  CRITICAL: Step 1 is mandatory when functionCode is provided. Review the documentation at
  https://docs.systeminit.com/reference/asset/function to understand each function type's
  requirements before validating user code.
  </usage>`;

const DEFAULT_QUALIFICATION_FUNCTION = `function main() {
    return { result: "success", message: "All good!" };
}`;

const DEFAULT_CODEGEN_FUNCTION = `async function main(component: Input): Promise<Output> {
  return {
    format: "json",
    code: JSON.stringify(component),
  };
}`;

const DEFAULT_MANAGEMENT_FUNCTION = `async function main({ thisComponent, components }: Input): Promise<Output> {
  throw new Error("unimplemented!");
}`;

const DEFAULT_ACTION_FUNCTION = `async function main(component: Input): Promise<Output> {
  throw new Error("unimplemented!");
}`;

const FuncUpsertInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to create a function in; functions cannot be created on HEAD",
    ),
  schemaId: z.string().describe("The schema id the function is for"),
  // funcId: z.string().optional().describe("The id of the function to edit. If none is given, create a new function"),
  name: z.string().describe("The name of the function").min(1),
  description: z.string().describe("A description for the function"),
  functionType: z.enum(["qualification", "codegen", "management", "action"]).describe("the kind of the function"),
  functionCode: z
    .string()
    .describe(
      `
      <important-instructions-you-should-always-follow-first>
      Before submitting code to this tool, please ensure that the function code is valid TypeScript and follows the guidelines provided in the documentation for the given function type.
      If the function code looks like a different function type than the one asked for, STOP and tell the user and ask them what to do.
      It is more important to ensure that the function code is valid than to complete the users instructions quickly.
      </important-instructions-you-should-always-follow-first>
      A typescript function definition. Documentation on functions can be found at https://docs.systeminit.com/reference/asset/function
      <qualification-explanation>
      Qualification functions take an argument, component, which has:
      code, available as a map of code generation results keyed on function name
      domain, which has the domain properties of the component
      resource, which has the resource information
      deleted_at, a string with the time of a deletion
      </qualification-explanation>
      <qualification-good-example>
      async function main(component: Input): Promise<Output> {
        const codeJson = component.code?.["awsIamPolicySimulatorCodeRequest"]
          ?.code as string;

        const args = ["iam", "simulate-custom-policy", "--cli-input-json", codeJson];
        const child = await siExec.waitUntilEnd("aws", args);
        if (child.exitCode !== 0) {
          console.log(child.stdout);
          console.error(child.stderr);
          return {
            result: "failure",
            message: "Policy simulator failed; AWS CLI 2 exited with non zero code",
          };
        }
        let response = JSON.parse(child.stdout);
        console.log("AWS Policy Response\n");
        console.log(JSON.stringify(response, null, 2));
        let result: "success" | "failure" | "warning" = "success";
        let message = "Policy evaluation succeded";
        for (const res of response["EvaluationResults"]) {
          if (res["EvalDecision"] === "implicitDeny") {
            result = "failure";
            message = "Policy evaluation returned a Deny";
          }
        }

        return {
          result,
          message,
        };
      }
      </qualification-good-example>
      `,
    )
    .optional(),
  actionKind: z.enum(["Create", "Destroy", "Refresh", "Update", "Manual"]).optional(),
};

const FuncUpsertOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({}),
};
const FuncUpsertOutputSchema = z.object(FuncUpsertOutputSchemaRaw);
type FuncUpsertOutputData = z.infer<typeof FuncUpsertOutputSchema>["data"];

export function funcUpsertTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "funcCreate",
        FuncUpsertOutputSchema,
      ),
      inputSchema: FuncUpsertInputSchemaRaw,
      outputSchema: FuncUpsertOutputSchemaRaw,
    },
    async ({ changeSetId, functionCode, functionType, schemaId, actionKind, ...createFuncRequest }) => {
      return await withAnalytics(name, async () => {
        const siSchemasApi = new SchemasApi(apiConfig);
        try {
          const response = await siSchemasApi.unlockSchema({
            workspaceId: WORKSPACE_ID,
            changeSetId,
            schemaId,
          });

          const schemaVariantId = response.data.unlockedVariantId;
          const request = {
            workspaceId: WORKSPACE_ID,
            changeSetId,
            schemaId,
            schemaVariantId,
          };

          if (functionType === "qualification") {
            await siSchemasApi.createVariantQualification({
              ...request,
              createVariantQualificationFuncV1Request: {
                ...createFuncRequest,
                code: functionCode ?? DEFAULT_QUALIFICATION_FUNCTION,
              },
            });
          } else if (functionType === "codegen") {
            await siSchemasApi.createVariantCodegen({
              ...request,
              createVariantCodegenFuncV1Request: {
                ...createFuncRequest,
                code: functionCode ?? DEFAULT_CODEGEN_FUNCTION,
              },
            });
          } else if (functionType === "management") {
            await siSchemasApi.createVariantManagement({
              ...request,
              createVariantManagementFuncV1Request: {
                ...createFuncRequest,
                code: functionCode ?? DEFAULT_MANAGEMENT_FUNCTION,
              },
            });
          } else if (functionType === "action") {
            if (!actionKind) {
              return errorResponse({
                message: "Action kind is required for action functions."
              });
            }
            // else if (actionKind !== "Manual") {
            //   TODO: Aaron - protect the user against duplicated action functions
            // }
            await siSchemasApi.createVariantAction({
              ...request,
              createVariantActionFuncV1Request: {
                ...createFuncRequest,
                code: functionCode ?? DEFAULT_ACTION_FUNCTION,
                kind: actionKind,
              },
            });
          } else {
            return errorResponse({
              message: "Currently the agent can only create qualification, codegen, management, and action functions."
            });
          }

          // TODO currently there is no resulting data - RETURN THE FUNC ID
          const data: FuncUpsertOutputData = {};
          return successResponse(data);
        } catch (error) {
          const anyError = error as any;
          if (anyError?.response?.data && JSON.stringify(anyError.response.data).includes("action with kind")) {
            return errorResponse({
              message: "An action of the same kind already exists and only one action of each kind is allowed, except for Manual.",
              hints: "Tell the user that they can't make more than one of this kind of action and ask if they want to make an action of a different kind."
            });
          }
          return errorResponse(error);
        }
      });
    },
  );
}
