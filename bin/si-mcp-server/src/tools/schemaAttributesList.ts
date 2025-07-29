import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { generateDescription, successResponse } from "./commonBehavior.ts";
import { getAttributesForService } from "../data/cfDb.ts";

const name = "schema-attributes-list";
const title = "List all the attributes of a schema";
const description =
  `<description>Lists all the attributes of a schema. Returns the schema name and an array of attribute objects that contain the Attribute Name, Path, and if it is Required. On failure, returns error details. Only supports AWS schemas.</description><usage>Use this tool to discover what attributes (sometimes called properties) are available for a schema.</usage>`;

const ListSchemaAttributesInputSchemaRaw = {
  schemaName: z.string().describe(
    "The Schema Name to retrieve the attributes for",
  ),
};

const ListSchemaAttributesOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    schemaName: z.string().describe("the schema name"),
    attributes: z.array(
      z.object({
        name: z.string().describe("the attributes name"),
        path: z.string().describe(
          "the absolute path of the attribute, which you can use to look up its documentation",
        ),
        required: z.boolean().describe("if this attribute is required"),
      }).describe("an attribute"),
    ).describe("array of attributes"),
  }).optional().describe("the schema information"),
};
const ListSchemaAttributesOutputSchema = z.object(
  ListSchemaAttributesOutputSchemaRaw,
);

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
    ({ schemaName }): CallToolResult => {
      const attributes = getAttributesForService(schemaName);
      return successResponse(
        {
          schemaName,
          attributes,
        },
        "If this is an AWS resource, the attributes map 1:1 to to the Cloudformation resource, where the path is calculated by looking at the Cloudformation resources nesting. You can look up the documentation for any attribute by its schemaName and path with the schema-attributes-documentation tool.",
      );
    },
  );
}
