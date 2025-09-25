import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
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
const title = "Create a new Schema";
const description =
  `<description></description><usage></usage>`;

const SchemaCreateInputSchemaRaw = {
  changeSetId: z.string().describe(
    "The change set to create a schema in; schemas cannot be created on HEAD",
  ),
  name: z.string().describe("The name of the schema").min(1),
};

const SchemaCreateOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({}),
};

const SchemaCreateOutputSchema = z.object(
  SchemaCreateOutputSchemaRaw,
);

type SchemaCreateResult = z.infer<
  typeof SchemaCreateOutputSchema
>["data"];

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
    async (
      { changeSetId, name }: { changeSetId: string; name: string },
    ): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const siSchemasApi = new SchemasApi(apiConfig);

        try {
          const response = await siSchemasApi.createSchema({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            createSchemaV1Request: {
              name,
              code: `function main() {
    const asset = new AssetBuilder();
    return asset.build();
}`
            },
          });
          return successResponse({});
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
