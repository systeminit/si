import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import {
  ActionsApi,
  ComponentsApi,
  FuncsApi,
  ManagementFuncsApi,
  SchemasApi,
} from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { AttributesSchema } from "../data/components.ts";
import { validateSchemaPrereqs } from "../data/schemaHints.ts";

const name = "component-discover";
const title =
  "Discover all the resources for a given schema, creating components for them all.";
const description =
  `<description>Discover all the resources for a given schema name, and create components for them in a change set. This tool will delete any components it uses to be able to refine the requirements of the discover process.</description><usage>Use this tool to bring an all the existing resources for a given Schema into System Initiative. For example, if the user asks to discover AWS::EC2::VPC's, then this tool will find all of the AWS::EC2::VPC's in the given region and account. After discovering components, you should ask the user if they want you to update the attributes of the discovered components to use subscriptions to any existing components attributes - for example, a discovered AWS::EC2::Subnet would be updated to have a subscription to the /resource_value/VpcId of the AWS::EC2::VPC that matches the imported VpcId attribute of the subnet.</usage>`;

const DiscoverComponentInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to discover the resources in; resources cannot be discovered on the HEAD change set",
    ),
  schemaName: z
    .string()
    .describe("the schema name of the resources to discover"),
  attributes: AttributesSchema.describe(
    "attributes of the schema that is being discovered can be used to filter what is discovered - for example, setting the /domain/VpcId attribute of an AWS::EC2::Subnet would discover all subnets whose /domain/VpcId matches that attributes value. *Always* set a /domain/extra/Region subscription and /secrets/AWS Credential subscription - these are required!",
  ),
};

const DiscoverComponentOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z
    .object({
      componentId: z.string().describe("the component id"),
      componentName: z.string().describe("the components name"),
      schemaName: z.string().describe("the schema for the component"),
      funcRunId: z
        .string()
        .nullable()
        .optional()
        .describe(
          "the function run id for this management function; useful for debugging failure",
        ),
    })
    .describe("the template component created to discover resources"),
};
const DiscoverComponentOutputSchema = z.object(
  DiscoverComponentOutputSchemaRaw,
);

type DiscoverComponentResult = z.infer<
  typeof DiscoverComponentOutputSchema
>["data"];

export function componentDiscoverTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentDiscoverResponse",
        DiscoverComponentOutputSchema,
      ),
      inputSchema: DiscoverComponentInputSchemaRaw,
      outputSchema: DiscoverComponentOutputSchemaRaw,
    },
    async ({
      changeSetId,
      schemaName,
      attributes,
    }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const prereqError = validateSchemaPrereqs(schemaName, attributes);
        if (prereqError) {
          return prereqError;
        }

        const siApi = new ComponentsApi(apiConfig);
        const siSchemasApi = new SchemasApi(apiConfig);
        const siFuncsApi = new FuncsApi(apiConfig);
        try {
          const findSchemaResponse = await siSchemasApi.findSchema({
            workspaceId: WORKSPACE_ID!,
            changeSetId: changeSetId,
            schema: schemaName,
          });
          const schemaId = findSchemaResponse.data.schemaId;

          const discoverTemplateResponse = await siApi.createComponent({
            workspaceId: WORKSPACE_ID!,
            changeSetId: changeSetId,
            createComponentV1Request: {
              name: `Discover ${schemaName} - Temporary`,
              schemaName,
              attributes,
            },
          });
          const discoverTemplateResult: Record<string, string> = {
            componentId: discoverTemplateResponse.data.component.id,
            componentName: discoverTemplateResponse.data.component.name,
            schemaName: schemaName,
          };

          // Now get the variantFuncs so we can decide on the discovery function
          let defaultVariantResponse = await siSchemasApi.getDefaultVariant({
            workspaceId: WORKSPACE_ID!,
            changeSetId: changeSetId,
            schemaId,
          });

          if (!defaultVariantResponse.data.variantFuncs) {
            // Variant data is still building, try again in a moment
            let attempts = 1;
            while (defaultVariantResponse.status === 202 && attempts < 6) {
              // Wait three seconds
              await new Promise(r => setTimeout(r, attempts * 1000));

              // Try again
              attempts++;
              defaultVariantResponse = await siSchemasApi.getDefaultVariant({
                workspaceId: WORKSPACE_ID!,
                changeSetId: changeSetId,
                schemaId,
              });
            }

            // Make sure we have the proper data before moving on!
            if (defaultVariantResponse.status === 202) {
              return errorResponse({
                message: "After 5 attempts, the workspace graph is still generating default variant data.",
              });
            } else if (!defaultVariantResponse.data.variantFuncs) {
              return errorResponse({
                message: "Error getting default variant function data",
              });
            }
          }

          const variantFunctions = defaultVariantResponse.data.variantFuncs;

          const discoverFunc = variantFunctions.find(
            (func) =>
              func.funcKind.kind === "management" &&
              func.funcKind.managementFuncKind === "discover",
          );

          if (!discoverFunc) {
            return errorResponse({
              message: `The schema ${schemaName} doesn't support discovery`,
            });
          }

          const discoverFuncResponse = await siFuncsApi.getFunc({
            workspaceId: WORKSPACE_ID!,
            changeSetId: changeSetId,
            funcId: discoverFunc.id,
          });

          const funcName = discoverFuncResponse.data.name;

          // Lets dequeue any actions created for this component
          const actionsApi = new ActionsApi(apiConfig);
          const queuedDiscoveryComponentActions = await actionsApi.getActions({
            workspaceId: WORKSPACE_ID!,
            changeSetId: changeSetId,
          });
          for (const action of queuedDiscoveryComponentActions.data.actions) {
            await actionsApi.cancelAction({
              workspaceId: WORKSPACE_ID!,
              changeSetId: changeSetId,
              actionId: action.id,
            });
          }

          try {
            const discoverResponse = await siApi.executeManagementFunction({
              workspaceId: WORKSPACE_ID!,
              changeSetId,
              componentId: discoverTemplateResult["componentId"],
              executeManagementFunctionV1Request: {
                managementFunction: { function: funcName },
              },
            });

            let discoverState = "Pending";
            const retrySleepInMs = 1000;
            const retryMaxCount = 260;
            let currentCount = 0;

            const mgmtApi = new ManagementFuncsApi(apiConfig);
            while (
              (discoverState == "Pending" ||
                discoverState == "Executing" ||
                discoverState == "Operating") &&
              currentCount <= retryMaxCount
            ) {
              if (currentCount != 0) {
                sleep(retrySleepInMs);
              }
              try {
                const status = await mgmtApi.getManagementFuncRunState({
                  workspaceId: WORKSPACE_ID!,
                  changeSetId,
                  managementFuncJobStateId:
                    discoverResponse.data.managementFuncJobStateId,
                });
                discoverState = status.data.state;
                if (status.data.funcRunId) {
                  discoverTemplateResult["funcRunId"] = status.data.funcRunId;
                }
                currentCount += 1;
              } catch (error) {
                return errorResponse({
                  message: `error fetching management function state: ${
                    JSON.stringify(
                      error,
                      null,
                      2,
                    )
                  }`,
                });
              }
            }
            if (currentCount > retryMaxCount) {
              return successResponse(
                discoverTemplateResult,
                "The discover function is still in progress; use the funcRunId to find out more",
              );
            } else if (discoverState == "Failure") {
              return errorResponse({
                response: {
                  status: "failed",
                  data: discoverTemplateResult,
                },
                message:
                  `failed to discover ${schemaName} resources; see funcRunId ${
                    discoverTemplateResult["funcRunId"]
                  } with the func-run-get tool for more information`,
              });
            } else {
              // Let's cleanup the discovery component now that the management function is successful
              await siApi.deleteComponent({
                workspaceId: WORKSPACE_ID!,
                changeSetId: changeSetId,
                componentId: discoverTemplateResult["componentId"],
              });

              return successResponse(discoverTemplateResult);
            }
          } catch (error) {
            return errorResponse(error);
          }
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
