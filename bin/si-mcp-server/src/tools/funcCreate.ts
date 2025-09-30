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

const name = "func-create";
const title = "Create a new function for a schema.";
const description = `
<description>
Create a new function of the given type for a given schema.
</description>
<usage>
Use this tool to create a new function. If the user only gives a schema name, use the schema-find tool to find the schemaId. If the user does not say which type of function they want to make, ask them which they want. If the user does not specify the actionKind for an action function, ask them which kind they want. Do not mention any schemaVariantId to the user.
</usage>
`;

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

const FuncCreateInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to create a function in; functions cannot be created on HEAD",
    ),
  schemaId: z.string().describe("The schema id the function is for"),
  name: z.string().describe("The name of the function").min(1),
  description: z.string().describe("A description for the function"),
  functionType: z.enum(["qualification", "codegen", "management", "action"]).describe("the kind of the function"),
  functionCode: z
    .string()
    .describe(
      `A typescript function definition. Documentation on functions can be found at https://docs.systeminit.com/reference/asset/function`,
    )
    .optional(),
  actionKind: z.enum(["Create", "Destroy", "Refresh", "Update", "Manual"]).optional(),
};

const FuncCreateOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({}),
};
const FuncCreateOutputSchema = z.object(FuncCreateOutputSchemaRaw);
type FuncCreateOutputData = z.infer<typeof FuncCreateOutputSchema>["data"];

export function funcCreateTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "funcCreate",
        FuncCreateOutputSchema,
      ),
      inputSchema: FuncCreateInputSchemaRaw,
      outputSchema: FuncCreateOutputSchemaRaw,
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

          // currently there is no resulting data
          const data: FuncCreateOutputData = {};
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
