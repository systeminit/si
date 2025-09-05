import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { SchemasApi, ChangeSetsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { ChangeSet } from "../data/changeSets.ts";

const name = "template-list";
const title = "List template schemas";
const description =
  `<description>Lists all template schemas in a change set. Templates are special schemas that can be used to generate multiple component instances. Returns an array of template schemas with their IDs, names, categories, and installation status. On failure, returns error details.</description><usage>Use this tool to discover available templates that can be run using the component-run-template tool. Templates are schemas that have management functions for generating component instances. Use the returned schemaId with component-create to instantiate template components, or look for existing template component instances using component-list.</usage>`;

const TemplateListInputSchemaRaw = {
  changeSetId: z.string().optional().describe(
    "The change set to look up template schemas in; if not provided, HEAD will be used",
  ),
  limit: z.string().optional().describe(
    "Maximum number of results to return (default: 50, max: 300)",
  ),
  cursor: z.string().optional().describe(
    "Cursor for pagination (SchemaId of the last item from previous page)",
  ),
  includeAllSchemas: z.boolean().optional().describe(
    "If true, include all schemas; if false (default), only include schemas that are likely templates (those with management functions or specific naming patterns)",
  ),
};

const TemplateListOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    templates: z.array(
      z.object({
        schemaId: z.string().describe("the template schema id"),
        schemaName: z.string().describe("the template schema name"),
        category: z.string().nullable().optional().describe("the template category"),
        installed: z.boolean().describe("whether the template is installed"),
        isTemplate: z.boolean().describe("whether this schema is identified as a template"),
      }).describe("an individual template schema"),
    ).describe("the list of template schemas"),
    nextCursor: z.string().nullable().optional().describe(
      "cursor for next page of results, null if no more pages",
    ),
    totalCount: z.number().describe("total number of templates returned"),
  }).describe("template listing results with pagination info"),
};

const TemplateListOutputSchema = z.object(
  TemplateListOutputSchemaRaw,
);

type TemplateListResult = z.infer<
  typeof TemplateListOutputSchema
>["data"];

export function templateListTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "templateList",
        TemplateListOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      inputSchema: TemplateListInputSchemaRaw,
      outputSchema: TemplateListOutputSchemaRaw,
    },
    async ({ changeSetId, limit, cursor, includeAllSchemas }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        if (!changeSetId) {
          const changeSetsApi = new ChangeSetsApi(apiConfig);
          try {
            const changeSetList = await changeSetsApi.listChangeSets({
              workspaceId: WORKSPACE_ID,
            });
            const changeSets = changeSetList.data.changeSets as ChangeSet[];
            const head = changeSets.find((cs) => cs.isHead);
            if (!head) {
              return errorResponse({
                message: "Could not find HEAD change set",
              });
            }
            changeSetId = head.id;
          } catch (error) {
            return errorResponse({
              message:
                `No change set id was provided, and we could not find HEAD; this is a bug! Tell the user we are sorry: ${
                  error instanceof Error ? error.message : String(error)
                }`,
            });
          }
        }

        try {
          const schemasApi = new SchemasApi(apiConfig);
          const response = await schemasApi.listSchemas({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            limit: limit,
            cursor: cursor,
          });

          // Filter schemas to identify templates
          const schemas = response.data.schemas;
          const shouldIncludeAll = includeAllSchemas || false;

          // Map all schemas and determine which are likely templates
          const templates = schemas
            .map((schema) => ({
              schemaId: schema.schemaId,
              schemaName: schema.schemaName,
              category: schema.category,
              installed: schema.installed,
              isTemplate: schema.category,
            }))
            // If includeAllSchemas is false, only return schemas identified as templates
            .filter((template) => shouldIncludeAll || template.isTemplate);

          const result: TemplateListResult = {
            templates,
            nextCursor: response.data.nextCursor,
            totalCount: templates.length,
          };

          return successResponse(result);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
