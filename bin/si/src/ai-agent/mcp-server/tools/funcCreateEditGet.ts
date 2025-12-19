import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod-v3";
import { validateFunctionCode } from "../validators/funcValidator.ts";
import { FuncsApi, SchemasApi } from "@systeminit/api-client";
import { Context } from "../../../context.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const toolName = "func-create-edit-get";
const title =
  "Create, update, or get information about a function for an existing schema.";
const description = `
<description>
Create, update, or get information about an existing function for an existing schema following the usage workflow.
</description>
<usage-workflow>
  *ALWAYS* follow this workflow:
  1. VALIDATION PHASE (REQUIRED):
     - If the user provides functionCode, you *MUST* first analyze it against the requested
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
     - Do not mention locking/unlocking schemas to the user
     - This tool cannot be used on the HEAD change set
  CRITICAL: Step 1 is mandatory when functionCode is provided.
  <validation-checklist>
    Before calling this tool, confirm:
    [ ] I have analyzed the functionCode syntax
    [ ] I have verified the function signature matches the functionType
    [ ] I have verified the return type matches the functionType
    [ ] If validation failed, I have stopped and informed the user
  </validation-checklist>
</usage-workflow>`;

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

// Reduced token usage: Reference documentation instead of embedding massive examples
// Previous: ~31KB (~15K tokens) | New: ~2KB (~500 tokens) | Savings: 96.7%
const functionCodeDescribe = [
  `<description>
    A TypeScript function definition. Complete documentation with examples: https://docs.systeminit.com/reference/function
    *Always* follow the workflow outlined in the <usage-workflow>.
  </description>`,
  `<validation-rules>
  Before submitting code to this tool, ensure:
  - Valid TypeScript syntax
  - Correct function signature for the requested type
  - Proper return type matches function type (see below)
  If the function code looks like a different function type than requested, STOP and inform the user.
  It is more important to ensure function code is valid than to complete instructions quickly.
  Always follow the workflow in the usage instructions!
  </validation-rules>`,
  `<function-types>
  Function types and their required return signatures:

  1. qualification: Validates component properties
     Input: component (has: code, domain, resource, deleted_at)
     Returns: {result: "success" | "failure" | "warning", message: string}

  2. codegen: Generates configuration code
     Input: component (has: domain, resource, deleted_at)
     Returns: {format: "json", code: string}

  3. management: Creates/configures multiple components
     Input: {thisComponent, components, currentView}
     Returns: {status: "ok", ops: {create: {...}}}

  4. action: Interacts with external systems (Create/Destroy/Refresh/Update/Manual)
     Input: component (has: properties with domain, resource, code, si)
     Returns: {status: "ok" | "error", payload?: any, message?: string, resourceId?: string}

  See https://docs.systeminit.com/reference/function for complete examples and patterns.
  Available APIs: siExec.waitUntilEnd() for CLI commands, Fetch API for HTTP, Lodash (_), requestStorage
  </function-types>`,
];

const funcCreateEditGetInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to create, update, get information about a function in; functions cannot be manipulated on HEAD",
    ),
  schemaId: z.string().describe("The schema id the function is for."),
  funcId: z
    .string()
    .optional()
    .describe(
      "The id of the function to edit or get information about. If none is given, create a new function.",
    ),
  name: z
    .string()
    .min(1)
    .optional()
    .describe(
      "The name of the function. Required for creating a new function.",
    ),
  description: z.string().optional().describe("A description for the function"),
  functionType: z
    .enum(["qualification", "codegen", "management", "action"])
    .optional()
    .describe(
      "The type of the function. Required for creating a new function.",
    ),
  functionCode: z.string().optional().describe(functionCodeDescribe.join(" ")),
  actionKind: z
    .enum(["Create", "Destroy", "Refresh", "Update", "Manual"])
    .optional()
    .describe(
      "The kind of action function. Only required for new functions of the action type.",
    ),
};

const funcCreateEditGetOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({
    funcId: z.string().describe("the function id"),
    name: z.string().describe("the function name"),
    functionCode: z.string().describe("the function code"),
  }),
};
const funcCreateEditGetOutputSchema = z.object(
  funcCreateEditGetOutputSchemaRaw,
);
type FuncCreateEditGetOutputData = z.infer<
  typeof funcCreateEditGetOutputSchema
>["data"];

export function funcCreateEditGetTool(server: McpServer) {
  server.registerTool(
    toolName,
    {
      title,
      description: generateDescription(
        description,
        "funcCreateEditGet",
        funcCreateEditGetOutputSchema,
      ),
      inputSchema: funcCreateEditGetInputSchemaRaw,
      outputSchema: funcCreateEditGetOutputSchemaRaw,
    },
    async ({
      changeSetId,
      schemaId,
      funcId,
      functionCode,
      functionType,
      actionKind,
      name,
      description,
    }) => {
      return await withAnalytics(toolName, async () => {
        if (functionType) {
          const validationIssues = validateFunctionCode(
            functionType,
            functionCode,
            actionKind,
          );
          if (validationIssues.length > 0) {
            return errorResponse({
              message: "Function code failed validation. Upsert aborted.",
              hints: validationIssues.map((i) => `• ${i.message}`).join("\n"),
            });
          }
        } else if (!funcId) {
          return errorResponse(
            {
              message:
                "Function type is required when creating a new function.",
            },
            "Provide a function type in the request body.",
          );
        }
        const apiConfig = Context.apiConfig();
        const workspaceId = Context.workspaceId();
        const siSchemasApi = new SchemasApi(apiConfig);
        const siFuncsApi = new FuncsApi(apiConfig);

        let hints, touchedFuncId, touchedFuncCode, touchedName: string;

        try {
          // Work with overlay functions

          // first ensure that the schema for this function is installed
          await siSchemasApi.installSchema({
            workspaceId: workspaceId,
            changeSetId,
            schemaId,
          });

          // then get the default variant
          const responseGetDefaultVariant =
            await siSchemasApi.getDefaultVariant({
              workspaceId: workspaceId,
              changeSetId,
              schemaId,
            });
          const isBuiltIn =
            responseGetDefaultVariant.data.installedFromUpstream;

          if (isBuiltIn) {
            if (funcId) {
              // EDIT
              // Fetch the existing function
              const responseGetFunc = await siFuncsApi.getFunc({
                workspaceId: workspaceId,
                changeSetId,
                funcId,
              });

              // Check if it's locked = overlay functions are unlocked if not yet applied
              if (responseGetFunc.data.isLocked) {
                return errorResponse({
                  message: "Cannot edit locked functions on builtin schemas.",
                });
              }

              // If no updates provuded, just return current information
              if (
                functionCode === undefined &&
                description === undefined &&
                name === undefined
              ) {
                return successResponse({
                  funcId: funcId,
                  name: responseGetFunc.data.displayName,
                  functionCode: responseGetFunc.data.code,
                });
              }

              // Edit the overlay function directly (no unlock required)
              const updateFuncV1Request = {
                code: functionCode ?? responseGetFunc.data.code,
                description: description ?? responseGetFunc.data.description,
                displayName: name ?? responseGetFunc.data.displayName,
              };

              await siFuncsApi.updateFunc({
                workspaceId: workspaceId,
                changeSetId,
                funcId,
                updateFuncV1Request,
              });

              return successResponse(
                {
                  funcId: funcId,
                  name:
                    responseGetFunc.data.displayName ||
                    responseGetFunc.data.name,
                  functionCode: updateFuncV1Request.code,
                },
                "Updated overlay function. Changes will be preserved on schema upgrades.",
              );
            } else {
              // CREATE

              if (!name) {
                return errorResponse({
                  message: "Name is required for creating action functions.",
                });
              }

              // Get the default schema variant ID
              const schemaVariantId = responseGetDefaultVariant.data.variantId;

              if (functionType === "action") {
                if (!actionKind) {
                  return errorResponse({
                    message:
                      "Action kind is required for creating action functions.",
                  });
                } else if (actionKind !== "Manual") {
                  let canMakeAction = true;

                  responseGetDefaultVariant.data.variantFuncs.forEach(
                    (func) => {
                      if (
                        func.funcKind.kind === "action" &&
                        func.funcKind.actionKind === actionKind
                      ) {
                        canMakeAction = false;
                      }
                    },
                  );
                  if (!canMakeAction) {
                    return errorResponse(
                      {
                        message:
                          "An action of the same kind already exists and only one of each kind is allowed, except for Manual action functions.",
                      },
                      "Tell the user that they can't make more of one of this kind of action and ask if they want to create a new Manual action.",
                    );
                  }
                }

                // Create an action function
                const code = functionCode ?? DEFAULT_ACTION_FUNCTION;
                const responseCreate = await siSchemasApi.createVariantAction({
                  workspaceId: workspaceId,
                  changeSetId,
                  schemaId,
                  schemaVariantId,
                  createVariantActionFuncV1Request: {
                    name,
                    description,
                    code,
                    kind: actionKind!,
                  },
                });

                return successResponse(
                  {
                    funcId: responseCreate.data.funcId,
                    name: name,
                    functionCode: code,
                  },
                  "Created overlay action function on a builtin schema. Changes will be preserved when the schema is upgraded.",
                );
              } else if (functionType === "management") {
                // Create a management function
                const code = functionCode ?? DEFAULT_MANAGEMENT_FUNCTION;
                const responseCreate =
                  await siSchemasApi.createVariantManagement({
                    workspaceId: workspaceId,
                    changeSetId,
                    schemaId,
                    schemaVariantId,
                    createVariantManagementFuncV1Request: {
                      name,
                      description,
                      code,
                    },
                  });

                return successResponse(
                  {
                    funcId: responseCreate.data.funcId,
                    name: name,
                    functionCode: code,
                  },
                  "Created overlay management function on a builtin schema. Changes will be preserved when the schema is upgraded.",
                );
              } else if (functionType === "codegen") {
                // Create a codegen function
                const code = functionCode ?? DEFAULT_CODEGEN_FUNCTION;
                const responseCreate = await siSchemasApi.createVariantCodegen({
                  workspaceId: workspaceId,
                  changeSetId,
                  schemaId,
                  schemaVariantId,
                  createVariantCodegenFuncV1Request: {
                    name,
                    description,
                    code,
                  },
                });

                return successResponse(
                  {
                    funcId: responseCreate.data.funcId,
                    name: name,
                    functionCode: code,
                  },
                  "Created overlay codegen function on a builtin schema. Changes will be preserved when the schema is upgraded.",
                );
              } else if (functionType === "qualification") {
                // Create a qualification function
                const code = functionCode ?? DEFAULT_QUALIFICATION_FUNCTION;
                const responseCreate =
                  await siSchemasApi.createVariantQualification({
                    workspaceId: workspaceId,
                    changeSetId,
                    schemaId,
                    schemaVariantId,
                    createVariantQualificationFuncV1Request: {
                      name,
                      description,
                      code,
                    },
                  });

                return successResponse(
                  {
                    funcId: responseCreate.data.funcId,
                    name: name,
                    functionCode: code,
                  },
                  "Created overlay qualification function on a builtin schema. Changes will be preserved when the schema is upgraded.",
                );
              }
            }
          } // None Overlay functions

          if (funcId) {
            // update an existing function or get information about it

            // first fetch existing data about the function
            const responseGetFunc = await siFuncsApi.getFunc({
              workspaceId: workspaceId,
              changeSetId,
              funcId,
            });

            // ensure that the schema is unlocked
            const responseUnlockSchema = await siSchemasApi.unlockSchema({
              workspaceId: workspaceId,
              changeSetId,
              schemaId,
            });

            // next make sure that the function is unlocked
            const responseUnlockFunc = await siFuncsApi.unlockFunc({
              workspaceId: workspaceId,
              changeSetId,
              funcId,
              unlockFuncV1Request: {
                schemaVariantId: responseUnlockSchema.data.unlockedVariantId,
              },
            });

            // fill the update request body with our new data or existing data if it didn't change
            const updateFuncV1Request = {
              code: functionCode ?? responseGetFunc.data.code,
              description: description ?? responseGetFunc.data.description,
              displayName: name ?? responseGetFunc.data.displayName,
            };

            // populate data to return from the tool
            touchedFuncId = responseUnlockFunc.data.unlockedFuncId;
            touchedFuncCode = updateFuncV1Request.code;
            touchedName = updateFuncV1Request.displayName as string;

            // information gathering complete, now only move onto updating if we have new data
            if (
              functionCode !== undefined ||
              description !== undefined ||
              name !== undefined
            ) {
              // finally hit the luminork API endpoint with the update
              await siFuncsApi.updateFunc({
                workspaceId,
                changeSetId,
                funcId: touchedFuncId,
                updateFuncV1Request,
              });
            }
          } else {
            // create a new function

            // first run checks to make sure we have the required data
            if (!schemaId) {
              return errorResponse({
                message:
                  "You cannot create a function without specifying the schemaId of the schema you want the function created on.",
              });
            }
            if (!name) {
              return errorResponse({
                message:
                  "You cannot create a function without giving it a name.",
              });
            }
            if (!functionType) {
              return errorResponse({
                message:
                  "You cannot create a function without specifying which type of function you want to create. This tool can create qualification, codegen, management, and action functions.",
              });
            }

            // then unlock the schema we will be creating a function on
            const responseUnlockSchema = await siSchemasApi.unlockSchema({
              workspaceId,
              changeSetId,
              schemaId,
            });
            const schemaVariantId = responseUnlockSchema.data.unlockedVariantId;

            // prepare the function create parameters
            const baseParams = {
              workspaceId,
              changeSetId,
              schemaId,
              schemaVariantId,
            };

            // get variant data for the variant we're working on
            const responseGetVariant = await siSchemasApi.getVariant({
              ...baseParams,
            });

            // issue a warning if the schema this function is being created on is a builtin
            if (responseGetVariant.data.installedFromUpstream) {
              hints =
                "Warn the user that because this function was created on a schema created by System Initiative that they will lose their customizations (like this function) if they upgrade the schema. Repeat this warning any time the user edits a function on a builtin schema.";
            }

            // use the correct funciton create endpoint based on the type of function
            let responseCreate, code;
            if (functionType === "qualification") {
              code = functionCode ?? DEFAULT_QUALIFICATION_FUNCTION;
              responseCreate = await siSchemasApi.createVariantQualification({
                ...baseParams,
                createVariantQualificationFuncV1Request: {
                  name,
                  description,
                  code,
                },
              });
            } else if (functionType === "codegen") {
              code = functionCode ?? DEFAULT_CODEGEN_FUNCTION;
              responseCreate = await siSchemasApi.createVariantCodegen({
                ...baseParams,
                createVariantCodegenFuncV1Request: {
                  name,
                  description,
                  code,
                },
              });
            } else if (functionType === "management") {
              code = functionCode ?? DEFAULT_MANAGEMENT_FUNCTION;
              responseCreate = await siSchemasApi.createVariantManagement({
                ...baseParams,
                createVariantManagementFuncV1Request: {
                  name,
                  description,
                  code,
                },
              });
            } else if (functionType === "action") {
              if (!actionKind) {
                return errorResponse({
                  message: "Action kind is required for action functions.",
                });
              } else if (actionKind !== "Manual") {
                // Before attempting to create this action, check if an action of the same type already exists.
                let canMakeAction = true;
                // deno-lint-ignore no-explicit-any
                responseGetVariant.data.variantFuncs.forEach((func: any) => {
                  if (
                    func.funcKind.kind === "action" &&
                    func.funcKind.actionKind === actionKind
                  ) {
                    canMakeAction = false;
                  }
                });

                if (!canMakeAction) {
                  return errorResponse(
                    {
                      message:
                        "An action of the same kind already exists and only one action of each kind is allowed, except for Manual.",
                    },
                    "Existing actions cannot be edited by this tool. *Do not* offer the option to edit the existing action function. Tell the user that they can't make more than one of this kind of action and ask if they want to make a manual action.",
                  );
                }
              }
              code = functionCode ?? DEFAULT_ACTION_FUNCTION;
              responseCreate = await siSchemasApi.createVariantAction({
                ...baseParams,
                createVariantActionFuncV1Request: {
                  name,
                  description,
                  code,
                  kind: actionKind,
                },
              });
            } else {
              return errorResponse({
                message:
                  "Currently the agent can only create qualification, codegen, management, and action functions.",
              });
            }
            // populate data to return from the tool
            touchedFuncId = responseCreate.data.funcId;
            touchedFuncCode = code;
            touchedName = name;
          }

          const data: FuncCreateEditGetOutputData = {
            funcId: touchedFuncId,
            name: touchedName,
            functionCode: touchedFuncCode,
          };
          return successResponse(data, hints);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
