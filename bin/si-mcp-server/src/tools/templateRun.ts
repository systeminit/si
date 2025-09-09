import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ComponentsApi, SchemasApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { AttributesSchema } from "../data/components.ts";
import { buildAttributesStructure } from "../data/schemaAttributes.ts";

const name = "template-run";
const title = "Run a Template";
const description =
  `<description>Run a template to create new components based on the template's definition. This tool is used to run templates that were previously generated using template-generate or found using template-list. The template execution will create new components according to the template's definition prefixed with a 'Name Prefix' which can be automatically genereated or chosen by the user.</description><usage>Use this tool to run a template. First, use template-list to discover available templates, and finally use this tool to run the template.</usage>`;

const TemplateRunInputSchemaRaw = {
  changeSetId: z.string().describe(
    "The change set to run the template in",
  ),
  schemaName: z.string().describe(
    "The schema name for the template to create and run",
  ),
  attributes: AttributesSchema.describe(
    "attributes to set on the component before running a template; this *must* include setting a raw value for /domain/Name Prefix which ends with a '-'; for AWS resources, this *must* include setting a subscription (usually from /secrets/AWS Credential on an AWS Credential component) for the AWS Credential, and *must* include setting a subscription (usually from /domain/region on a Region component) for /domain/extra/Region as well",
  ),
  templateName: z.string().describe(
    "The name of the template to create and run",
  ),
};

const TemplateRunOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    managementFuncJobStateId: z.string().describe(
      "The job state ID for the enqueued or executed management function",
    ),
    message: z.string().optional().describe(
      "Optional message from the management function execution",
    ),
  }).describe(
    "Information about the template execution including job state and results",
  ),
};

const TemplateRunOutputSchema = z.object(
  TemplateRunOutputSchemaRaw,
);

type TemplateRunResult = z.infer<
  typeof TemplateRunOutputSchema
>["data"];

export function templateRunTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "templateRun",
        TemplateRunOutputSchema,
      ),
      inputSchema: TemplateRunInputSchemaRaw,
      outputSchema: TemplateRunOutputSchemaRaw,
    },
    async (
      { changeSetId, schemaName, attributes, templateName },
    ): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        try {
          const siComponentsApi = new ComponentsApi(apiConfig);
          const siSchemasApi = new SchemasApi(apiConfig);

          const findSchemaResponse = await siSchemasApi.findSchema({
            workspaceId: WORKSPACE_ID,
            changeSetId,
            schema: schemaName,
          });
          const schemaId = findSchemaResponse.data.schemaId;

          const defaultVariantResponse = await siSchemasApi.getDefaultVariant({
            workspaceId: WORKSPACE_ID,
            changeSetId,
            schemaId,
          });

          if (defaultVariantResponse.status === 202) {
            return errorResponse({
              message:
                "The data is not yet available for this request. Try again in a few seconds",
            });
          }

          const attributesStructure = buildAttributesStructure(
            defaultVariantResponse.data,
          );
          const needsExtraRegion = attributesStructure.attributes.some((a) =>
            a.path === "/domain/extra/Region"
          );
          const needsAwsCredential = attributesStructure.attributes.some((a) =>
            a.path === "/secrets/AWS Credential"
          );
          const needsNamePrefix = attributesStructure.attributes.some((a) =>
            a.path === "/domain/Name Prefix"
          );

          let hasExtraRegion = false;
          let hasAwsCredential = false;
          let hasNamePrefix = false;
          for (const path of Object.keys(attributes)) {
            if (path == "/domain/extra/Region") {
              hasExtraRegion = true;
            }
            if (path == "/secrets/AWS Credential") {
              hasAwsCredential = true;
            }
            if (path == "/domain/Name Prefix") {
              hasNamePrefix = true;
            }
          }

          const missingExtraRegion = needsExtraRegion && !hasExtraRegion;
          const missingAwsCredential = needsAwsCredential && !hasAwsCredential;
          const missingNamePrefix = needsNamePrefix && !hasNamePrefix;

          if (missingExtraRegion || missingAwsCredential || missingNamePrefix) {
            let message =
              "There are missing or malformed required attributes for the template component to be run.";
            if (missingNamePrefix) {
              message =
                `${message} All Templates must have a /domain/Name Prefix set by a raw value.`;
            }
            if (missingExtraRegion || missingAwsCredential) {
              message = `${message} This template contains AWS resources.`;
            }
            if (missingExtraRegion) {
              message =
                `${message} We must have /domain/extra/Region set to a subscription or by a raw value.`;
            }
            if (missingAwsCredential) {
              message =
                `${message} We must have /secrets/AWS Credential set to a subscription.`;
            }
            return errorResponse({
              response: { status: "bad prereq", data: {} },
              message,
            });
          }

          const createComponentResponse = await siComponentsApi.createComponent(
            {
              workspaceId: WORKSPACE_ID,
              changeSetId: changeSetId,
              createComponentV1Request: {
                name: templateName,
                schemaName,
                attributes,
              },
            },
          );

          const managementFunction = { function: "Run Template" };
          const executeResponse = await siComponentsApi
            .executeManagementFunction({
              workspaceId: WORKSPACE_ID,
              changeSetId,
              componentId: createComponentResponse.data.component.id,
              executeManagementFunctionV1Request: {
                managementFunction,
              },
            });

          const result: TemplateRunResult = {
            managementFuncJobStateId:
              executeResponse.data.managementFuncJobStateId,
            message: executeResponse.data.message,
          };

          return successResponse(result);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
