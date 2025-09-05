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

const name = "component-generate-template";
const title = "Generate a template from a list a components";
const description =
  `<description>Generate a template for a given list of componentIds and a name for the template. If the list is empty, do not use this tool. Only include componentIds explicitly listed. The name can either be explicitly given or generated based on the names of the components themselves. The name must be relevant to the components chosen. Returns the 'success' on successful creation of a template. On failure, returns error details.</description><usage>Use this tool to generate a template from at least one component in a change set. Use only the component-list tool to understand the components that are available to be included in a template. For the template name, generate it based on the component names, schema names, and their conceptual relevancy to one another. To see all of its information after the template has been generated, use the schema-find tool.</usage>`;

const ComponentGenerateTemplateInputSchemaRaw = {
  changeSetId: z.string().describe(
    "The change set to generate a template in; templates cannot be generated on the HEAD change set",
  ),
  componentIds: z.array(
    z.string().describe(
      "the component id to be included in the generated template",
    ),
  ).describe(
    "the list of component ids to be included in the generated template",
  ),
  templateName: z.string().describe(
    "the name of the template that will be generated",
  ),
  secrets: z.boolean().optional().describe(
    "include secret-defining components in the template to be generated; useful for excluding common base components and not tying in credential setup into templates; the user may want to make this decision themselves",
  ),
  region: z.boolean().optional().describe(
    "include components that define the region for your infrastructure in the template to be generated; useful for excluding common base components; the user may want to make this decision themselves",
  ),
};

const ComponentGenerateTemplateOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    schemaId: z.string().describe("the schema id of the generated template"),
    schemaVariantId: z.string().describe(
      "the schema variant id of the generated template",
    ),
    funcId: z.string().describe(
      "the func id for running the generated template",
    ),
    schema: z.object({
      // FIXME(nick,aaron): re-use this type from the "find-schema" tool rather than hard copy/paste.
      schemaId: z.string().describe("the schema id for the generated template"),
      schemaName: z.string().describe(
        "the name of the schema for the generated template",
      ),
      description: z
        .string()
        .optional()
        .describe(
          "a description of the schema for the generated template, frequently containing documentation",
        ),
      link: z
        .string()
        .url()
        .optional()
        .describe(
          "an external URL that contains documentation about what this schema for the generated template is modeling; this will likely be null because we just generated it",
        ),
    })
      .optional()
      .describe("the information for the schema for the generated template"),
  }).describe(
    "the information for the generated template including all ids relevant for future tasks; the ids are not as important to the user, but there is not need to obfuscate them either",
  ),
};

const ComponentGenerateTemplateOutputSchema = z.object(
  ComponentGenerateTemplateOutputSchemaRaw,
);

type ComponentGenerateTemplateResult = z.infer<
  typeof ComponentGenerateTemplateOutputSchema
>["data"];

export function componentGenerateTemplateTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentGenerateTemplate",
        ComponentGenerateTemplateOutputSchema,
      ),
      inputSchema: ComponentGenerateTemplateInputSchemaRaw,
      outputSchema: ComponentGenerateTemplateOutputSchemaRaw,
    },
    async (
      { changeSetId, componentIds, templateName },
    ): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const siComponentsApi = new ComponentsApi(apiConfig);
        const siSchemasApi = new SchemasApi(apiConfig);

        try {
          const response = await siComponentsApi.generateTemplate({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            generateTemplateV1Request: {
              componentIds,
              assetName: templateName,
              funcName: `Generate ${templateName}`,
            },
          });

          const schemaResponse = await siSchemasApi.findSchema({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            schemaId: response.data.schemaId,
          });

          const result: ComponentGenerateTemplateResult = {
            schemaId: response.data.schemaId,
            schemaVariantId: response.data.schemaVariantId,
            funcId: response.data.funcId,
            schema: schemaResponse.data,
          };

          return successResponse(
            result,
          );
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
