import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import {
  errorResponse,
  findHeadChangeSet,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { ChangeSetsApi, SchemasApi } from "@systeminit/api-client";
import { Context } from "../../../context.ts";
import {
  buildAttributeDocsIndex,
  formatDocumentation,
} from "../data/schemaAttributes.ts";
import { isValid } from "ulid";

const name = "schema-attributes-documentation";
const title = "Schema Attributes Documentation";
const description = `<description>Look up the documentation for Schema Attributes - you can look up many at once for a single schema. Returns an object with the schemaName and an array of documentation and path attribute objects. (if any). On failure, returns error details.</description><usage>Use this tool to understand how to use a particular attribute, or what values it accepts. Use attribute paths that mirror those returned from the schema-attributes-list tool. In addition, you can ask for the documentation for paths *earlier* than those returned by the attributes-list tool - for example, the tool might return '/domain/Tags/[array]/Key', but the user wants documentation for '/domain/Tags' - both are valid.</usage>`;

const DocumentSchemaAttributesInputSchemaRaw = {
  schemaNameOrId: z
    .string()
    .describe(
      "The Schema Name or Schema ID to retrieve attribute documentation for.",
    ),
  changeSetId: z
    .string()
    .optional()
    .describe(
      "The change set to retrieve attribute documentation in. If none is provided, the HEAD change set is used.",
    ),
  schemaAttributePaths: z
    .array(
      z
        .string()
        .min(1, "Provide a schema attribute path")
        .refine((p: string) => p.startsWith("/"), {
          message: "Each path must start with '/' (e.g., /domain/RoleName)",
        })
        .describe("Schema attribute path (e.g., /domain/RoleName)"),
    )
    .nonempty("Provide at least one attribute path")
    .describe("List of schema attribute paths to retrieve"),
};

const DocumentSchemaAttributesOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z
    .object({
      schemaName: z
        .string()
        .describe("The schema these attribute docs belong to"),
      attributes: z
        .array(
          z
            .object({
              schemaAttributePath: z
                .string()
                .describe("The schema attribute path"),
              documentation: z
                .string()
                .describe("The documentation for this attribute"),
            })
            .describe("Documentation for a single attribute path"),
        )
        .describe("All requested attribute docs for this schema"),
    })
    .optional()
    .describe("The documentation payload for the single schema"),
};
const DocumentSchemaAttributesOutputSchema = z.object(
  DocumentSchemaAttributesOutputSchemaRaw,
);
type DocumentSchemaAttributesOutput = z.infer<
  typeof DocumentSchemaAttributesOutputSchema
>;

export function schemaAttributesDocumentationTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "schemaAttributesListResponse",
        DocumentSchemaAttributesOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      inputSchema: DocumentSchemaAttributesInputSchemaRaw,
      outputSchema: DocumentSchemaAttributesOutputSchemaRaw,
    },
    async ({
      schemaNameOrId,
      changeSetId,
      schemaAttributePaths,
    }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const apiConfig = Context.apiConfig();
        const workspaceId = Context.workspaceId();
        let responseData: DocumentSchemaAttributesOutput["data"];

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
          let schemaId, schemaName;
          if (!isValid(schemaNameOrId)) {
            schemaName = schemaNameOrId;
            try {
              const response = await siSchemasApi.findSchema({
                workspaceId: workspaceId,
                changeSetId: changeSetId!,
                schema: schemaNameOrId,
              });
              schemaId = response.data.schemaId;
            } catch (error) {
              const errorMessage =
                error instanceof Error ? error.message : String(error);
              return errorResponse({
                message: `Unable to find the schema - check the name and try again. Tell the user we are sorry: ${errorMessage}`,
              });
            }
          } else {
            schemaId = schemaNameOrId;
          }

          const response = await siSchemasApi.getDefaultVariant({
            workspaceId: workspaceId,
            changeSetId: changeSetId!,
            schemaId: schemaId,
          });

          if (response.status === 202) {
            return errorResponse({
              message:
                "The data is not yet available for this request. Try again in a few seconds",
            });
          }

          const variant = response.data;

          // Build a path -> { description, docLink } index once (O(n))
          const docsIndex = buildAttributeDocsIndex(variant);

          // Create one entry per requested path
          const attributes = schemaAttributePaths.map(
            (schemaAttributePath: string) => {
              const documentation =
                formatDocumentation(docsIndex, schemaAttributePath) ??
                "There is no documentation for this attribute; if it is an AWS schema, consider looking up the data for the corresponding cloudformation resource";

              return { schemaAttributePath, documentation };
            },
          );

          if (!schemaName) {
            schemaName = response.data.displayName as string;
          }

          responseData = {
            schemaName,
            attributes,
          };

          return successResponse(
            responseData,
            "You can use a web search to find the cloudformation schema",
          );
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
