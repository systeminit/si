import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ComponentsApi, SchemasApi, SearchApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const name = "template-generate";
const title =
  "Generate a template from user provided or searched for componentIds";
const description = `
  <important>
  Do not use the component-list tool, *always* prefer this tools search functionality.
  </important>
  <description>Generate a template for a user provided list of componentIds or componentIds returned via this tools search functionality. Only include provided componentIds or results from the search in this tool. The name can either be explicitly given or generated based on the names of the components themselves. The name must be relevant to the components chosen. Returns the 'success' on successful creation of a template. On failure, returns error details.</description>
  <usage>Use this tool to generate a template from provided componentIds or componentIds returned from search results in a change set. If using search, reference the syntax here for how to perform searches for one or multiple components -> 'https://docs.systeminit.com/explanation/search-syntax'. For the template name, generate it based on the component names, schema names, and their conceptual relevancy to one another. To see all of its information after the template has been generated, use the schema-find tool.
  </usage>
  `;

const TemplateGenerateInputSchemaRaw = {
  changeSetId: z.string().describe(
    "The change set to generate a template in; templates cannot be generated on the HEAD change set",
  ),
  componentIds: z.array(
    z.string().describe(
      "the component id to be included in the generated template",
    ),
  ).optional().describe(
    "the list of component ids to be included in the generated template, if none provided the tool will search",
  ),
  searchQuery: z.string().optional().describe(
    `
    <description>
    when componentIds are not provided this search query will be used to search for componentIds to include in the template
    </description>
    <documentation>
    Search Syntax

    Component name: Search for prod to find components with prod in their name.
    Schema name: Search for Instance to find EC2 instances.
    Combine them: Search for prod Instance to find EC2 instances with prod in their name!
    When you need more than mere words can convey, you can use more advanced search features like attribute searches and boolean logic.

    Attribute Search Syntax
    To search inside components, you can use attribute searches. InstanceType:, for example, will search for instances with that type. Specific syntax for attribute searches:

    Basic Syntax: InstanceType:m8g.medium will search for m8g.medium instances.

    Alternatives: InstanceType:m8g.medium|m8g.small will search for m8g.medium or m8g.large instances.

    Wildcards: InstanceType:m8g.* will search for all m8g instances regardless of size.

    Wildcards can be placed anywhere in the value: InstanceType:m*.large will match m8g.large, m7g.large and even m7i-flex.large.

    Tip: While building your infrastructure, you may want to find things where you did not specify an attribute. For example, !AvailabilityZone:* will bring back instances where you did not specify an AvailabilityZone, so you can add one!

    Exact Matches: Runtime:"python3.11" will match only the python3.11 runtime on a lambda function, but not python3.

    You can use quotes (") to pin down your search and match an exact value. If you don't use quotes, things that start with the value you specify are matched.

    Quotes will also allow you to use spaces in your search: Description:"Production Access".

    Attribute Paths: LaunchTemplate/Version:1 will match instances with LaunchTemplate version 1.

    Sometimes an attribute has a generic name, and you need to specify more of its path. LaunchTemplate/Version:1 is useful because it will not bring in every other AWS resource with a random Version field set to 1.

    Schema: schema:AWS::EC2::Instance, or schema:Instance, will find all EC2 instances.

    All of these features can be mixed and matched: InstanceType:m8g.*|"mac1.metal" will find m8g instances as well as mac1.metal instances.

    Boolean Logic
    Sometimes you need more precise logic than just "find things matching A, B and C." For this, we support full boolean logic, with nesting:

    Negation: !InstanceType:m8g.large will match all instances that are not m8g.large.
    Alternatives: Instance | Image will match all instances and images.
    Grouping: (prod Instance) | (dev Image) will match Instances in prod, and images with "dev" in the name.
    "And" (narrowing a search) is done by putting spaces between things. & is supported but redundant: prod Instance and prod & Instance do the same thing.

    </documentation>

    `,
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

const TemplateGenerateOutputSchemaRaw = {
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

const TemplateGenerateOutputSchema = z.object(
  TemplateGenerateOutputSchemaRaw,
);

type TemplateGenerateResult = z.infer<
  typeof TemplateGenerateOutputSchema
>["data"];

export function templateGenerateTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "templateGenerate",
        TemplateGenerateOutputSchema,
      ),
      inputSchema: TemplateGenerateInputSchemaRaw,
      outputSchema: TemplateGenerateOutputSchemaRaw,
    },
    async (
      { changeSetId, componentIds, templateName, searchQuery },
    ): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const siComponentsApi = new ComponentsApi(apiConfig);
        const siSchemasApi = new SchemasApi(apiConfig);
        const siSearchApi = new SearchApi(apiConfig);

        try {
          let templateComponentIds: string[];
          const hasListOfIds = componentIds && componentIds.length > 0;

          if (hasListOfIds && !searchQuery) {
            templateComponentIds = componentIds;
          } else if (!hasListOfIds && searchQuery) {
            const searchResponse = await siSearchApi.search({
              workspaceId: WORKSPACE_ID,
              changeSetId: changeSetId,
              q: searchQuery,
            });

            templateComponentIds = searchResponse.data.components.map((
              component,
            ) => component.id);

            if (templateComponentIds.length < 1) {
              return errorResponse(
                "No components found for the provided search query.",
              );
            }
          } else if (hasListOfIds && searchQuery) {
            return errorResponse(
              "You cannot use both a list of componentIds and a search query.",
            );
          } else {
            return errorResponse(
              "You must provide either a list of componentIds or a search query.",
            );
          }

          const response = await siComponentsApi.generateTemplate({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            generateTemplateV1Request: {
              componentIds: templateComponentIds,
              assetName: templateName,
              funcName: `Generate ${templateName}`,
            },
          });

          const schemaResponse = await siSchemasApi.findSchema({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            schemaId: response.data.schemaId,
          });

          const result: TemplateGenerateResult = {
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
