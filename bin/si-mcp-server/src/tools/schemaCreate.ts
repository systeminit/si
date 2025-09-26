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

const name = "schema-create";
const title = "Create a new schema";
const description = `
<description>
Create a new schema, which can then be used to create components of that type.
</description>
<usage>
</usage>
`;

const DEFAULT_SCHEMA_DEFINITION_FUNCTION = `function main() {
    const asset = new AssetBuilder();
    return asset.build();
}`;

const SchemaCreateInputSchemaRaw = {
  changeSetId: z
    .string()
    .describe(
      "The change set to create a schema in; schemas cannot be created on HEAD",
    ),
  name: z.string().describe("The name of the schema").min(1),
  description: z.string().optional().describe("The description of the schema"),
  link: z
    .string()
    .optional()
    .describe("A link to documentation about the thing the schema represents."),
  category: z.string().optional().describe("The category of the schema"),
  color: z.string().optional().describe("The color "), // TODO - force the AI agent to only use hex colors, not words!
  definitionFunction: z
    .string()
    .describe(
      `A typescript function, starting with "function main() {", defining the schema's
       properties using an AssetBuilder. Documentation on how to write this function can
       be found at https://docs.systeminit.com/reference/asset/schema.`,
    )
    .default(DEFAULT_SCHEMA_DEFINITION_FUNCTION),
};

const SchemaCreateOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({}),
};
const SchemaCreateOutputSchema = z.object(SchemaCreateOutputSchemaRaw);
type SchemaCreateOutputData = z.infer<typeof SchemaCreateOutputSchema>["data"];

export function schemaCreateTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "schemaCreate",
        SchemaCreateOutputSchema,
      ),
      inputSchema: SchemaCreateInputSchemaRaw,
      outputSchema: SchemaCreateOutputSchemaRaw,
    },
    async ({ changeSetId, definitionFunction, ...createSchemaV1Request }) => {
      return await withAnalytics(name, async () => {
        const siSchemasApi = new SchemasApi(apiConfig);

        try {
          await siSchemasApi.createSchema({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            createSchemaV1Request: {
              ...createSchemaV1Request,
              code: definitionFunction,
            },
          });
          const data: SchemaCreateOutputData = {};
          return successResponse(data);
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
