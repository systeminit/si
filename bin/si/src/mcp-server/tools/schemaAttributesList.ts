import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import {
  errorResponse,
  findHeadChangeSet,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { ChangeSetsApi, SchemasApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import { buildAttributesStructure } from "../data/schemaAttributes.ts";
import { isValid } from "ulid";

const name = "schema-attributes-list";
const title = "List all the attributes of a schema";
const description =
  `<description>Lists all the attributes of a schema. Returns the schema name and an array of attribute objects that contain the Attribute Name, Path, and if it is Required. On failure, returns error details.</description><usage>Use this tool to discover what attributes (sometimes called properties) are available for a schema.</usage>`;

const ListSchemaAttributesInputSchemaRaw = {
  schemaNameOrId: z.string().describe(
    "The Schema Name or Schema ID to retrieve the attributes for.",
  ),
  changeSetId: z.string().optional().describe(
    "The change set to retreive the schema attributes in. If none is provided, the HEAD change set is used.",
  ),
};

const ListSchemaAttributesOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z
    .object({
      schemaName: z.string().describe("the schema name"),
      attributes: z
        .array(
          z
            .object({
              name: z.string().describe("the attributes name"),
              path: z
                .string()
                .describe(
                  "the absolute path of the attribute, which you can use to look up its documentation",
                ),
              required: z.boolean().describe("if this attribute is required"),
            })
            .describe("an attribute"),
        )
        .describe("array of attributes"),
    })
    .optional()
    .describe("the schema information"),
};
const ListSchemaAttributesOutputSchema = z.object(
  ListSchemaAttributesOutputSchemaRaw,
);
type ListSchemaAttributesOutput = z.infer<
  typeof ListSchemaAttributesOutputSchema
>;

export function schemaAttributesListTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "schemaAttributesListResponse",
        ListSchemaAttributesOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      inputSchema: ListSchemaAttributesInputSchemaRaw,
      outputSchema: ListSchemaAttributesOutputSchemaRaw,
    },
    async ({ schemaNameOrId, changeSetId }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        let responseData: ListSchemaAttributesOutput["data"];

        if (!changeSetId) {
          const changeSetsApi = new ChangeSetsApi(apiConfig);
          const headChangeSet = await findHeadChangeSet(changeSetsApi, false);
          if (headChangeSet.changeSetId) {
            changeSetId = headChangeSet.changeSetId;
          } else {
            return errorResponse(headChangeSet);
          }
        }

        const siSchemasApi = new SchemasApi(apiConfig);
        try {
          let schemaId = "";
          if (!isValid(schemaNameOrId)) {
            try {
              const response = await siSchemasApi.findSchema({
                workspaceId: WORKSPACE_ID,
                changeSetId: changeSetId!,
                schema: schemaNameOrId,
              });
              schemaId = response.data.schemaId;
            } catch (error) {
              const errorMessage = error instanceof Error
                ? error.message
                : String(error);
              return errorResponse({
                message:
                  `Unable to find the schema - check the name and try again. Tell the user we are sorry: ${errorMessage}`,
              });
            }
          } else {
            schemaId = schemaNameOrId;
          }

          const response = await siSchemasApi.getDefaultVariant({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId!,
            schemaId,
          });

          if (response.status === 202) {
            return errorResponse({
              message:
                "The data is not yet available for this request. Try again in a few seconds",
            });
          }

          const attributeDetails = buildAttributesStructure(response.data);
          responseData = {
            schemaName: attributeDetails.schemaName,
            attributes: attributeDetails.attributes,
          };

          return successResponse(
            responseData,
            "For AWS schemas, you can use a web search to find the cloudformation schema",
          );
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
