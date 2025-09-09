import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ChangeSetsApi, SchemasApi } from "@systeminit/api-client";
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
  `<description>List template schemas in a change set. Templates are special schemas in the 'Templates' category that can be used to generate multiple components. Returns an array of template schemas with their IDs, names, categories, and installation status. On failure, returns error details.</description><usage>Use this tool to list all available templates that can be ran using the template-run tool. Templates are schemas in the 'Templates' category that can be ran to generate components.</usage>`;

const TemplateListInputSchemaRaw = {
  changeSetId: z.string().optional().describe(
    "The change set to list template schemas in; if not provided, HEAD will be used",
  ),
};

const TemplateListOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.array(
    z.object({
      schemaId: z.string().describe("the template schema id"),
      schemaName: z.string().describe("the template schema name"),
      category: z.string().nullable().optional().describe(
        "the template category",
      ),
      installed: z.boolean().describe("whether the template is installed"),
    }).describe("an individual template schema"),
  ).describe("the list of template schemas"),
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
    async ({ changeSetId }): Promise<CallToolResult> => {
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

          const response = await schemasApi.searchSchemas({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            searchSchemasV1Request: {
              category: "Templates",
            },
          });

          // Map the response to our output format
          const templates: TemplateListResult = response.data.schemas.map((
            schema,
          ) => ({
            schemaId: schema.schemaId,
            schemaName: schema.schemaName,
            category: schema.category,
            installed: schema.installed,
          }));

          return successResponse(templates);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
