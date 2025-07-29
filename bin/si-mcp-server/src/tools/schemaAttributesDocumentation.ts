import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { generateDescription, successResponse } from "./commonBehavior.ts";
import { getSchemaAttributeDocumentation } from "../data/cfDb.ts";

const name = "schema-attributes-documentation";
const title = "Schema Attributes Documentation";
const description =
  `<description>Look up the documentation for Schema Attributes - you can look up many at once, from multiple schemas at once. Returns an array with the documentation, schemaName, and path attribute (if any). On failure, returns error details. Only supports AWS schemas.</description><usage>Use this tool to understand how to use a particular attribute, or what values it accepts. Use attribute paths that mirror those returned from the schema-attributes-list tool. In addition, you can ask for the documentation for paths *earlier* than those returned by the attributes-list tool - for example, the tool might return '/domain/Tags/[array]/Key', but the user wants documentation for '/domain/Tags' - both are valid.</usage>`;

const DocumentSchemaAttributesInputSchemaRaw = {
  documentationToRetrive: z.array(z.object({
    schemaName: z.string().describe(
      "The Schema Name to retrieve the attribute documentation for",
    ),
    schemaAttributePath: z.string().describe(
      "The schema attribute path to retrieve documentation for.",
    ),
  })).describe(
    "A list of all the schema names and schema attribute paths to retrieve",
  ),
};

const DocumentSchemaAttributesOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.array(
    z.object({
      schemaName: z.string().describe("the schema name"),
      schemaAttributePath: z.string().describe("the schema attribute path"),
      documentation: z.string().describe("the documentation for the attribute"),
    }).describe("the documentation for a given schemaAttributePath"),
  ).describe("an array of documentation"),
};
const DocumentSchemaAttributesOutputSchema = z.object(
  DocumentSchemaAttributesOutputSchemaRaw,
);

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
    ({ documentationToRetrive }): CallToolResult => {
      const docs = [];
      for (const docSpec of documentationToRetrive) {
        const documentation = getSchemaAttributeDocumentation(
          docSpec.schemaName,
          docSpec.schemaAttributePath,
        );
        const response = {
          schemaName: docSpec.schemaName,
          schemaAttributePath: docSpec.schemaAttributePath,
          documentation: documentation ||
            "There is no documentation for this attribute; if it is an AWS schema, consider looking up the data for the corresponding cloudformation resource",
        };
        docs.push(response);
      }
      return successResponse(
        docs,
      );
    },
  );
}
