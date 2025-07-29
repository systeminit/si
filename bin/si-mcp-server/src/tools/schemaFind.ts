import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ChangeSetsApi, SchemasApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
} from "./commonBehavior.ts";
import { isValid } from "ulid";
import { getDocumentationForService } from "../data/cfDb.ts";

const name = "schema-find";
const title = "Find component schemas";
const description =
  `<description>Finds component schemas by name or Schema ID. Returns the Schema ID, Name, Description, and external documentation Link. On failure, returns error details. When looking for AWS Schemas, you can use the AWS Cloudformation Resource name (examples: AWS::EC2::Instance, AWS::Bedrock::Agent, or AWS::ControlTower::EnabledBaseline)</description><usage>Use this tool to find if a schema exists in System Initiative, to look up the Schema Name or Schema ID if you need it, or to display high level information about the schema.</usage>`;

const FindSchemaInputSchemaRaw = {
  changeSetId: z.string().optional().describe(
    "The change set to look up the schema in; if not provided, HEAD will be used",
  ),
  schemaNameOrId: z.string().describe(
    "The Schema Name or Schema ID to retrieve",
  ),
};

const FindSchemaOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    schemaId: z.string().describe("the schema id"),
    schemaName: z.string().describe("the name of the schema"),
    description: z.string().optional().describe(
      "a description of the schema, frequently containing documentation",
    ),
    link: z.string().url().optional().describe(
      "an external URL that contains documentation about what this schema is modeling",
    ),
  }).optional().describe("the schema information"),
};
const FindSchemaOutputSchema = z.object(
  FindSchemaOutputSchemaRaw,
);
type FindSchemaOutput = z.infer<typeof FindSchemaOutputSchema>;

interface ChangeSetItem {
  id: string;
  isHead: boolean;
  [key: string]: unknown;
}

export function schemaFindTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "schemaFindResponse",
        FindSchemaOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      inputSchema: FindSchemaInputSchemaRaw,
      outputSchema: FindSchemaOutputSchemaRaw,
    },
    async ({ changeSetId, schemaNameOrId }): Promise<CallToolResult> => {
      if (!changeSetId) {
        const changeSetsApi = new ChangeSetsApi(apiConfig);
        try {
          const changeSetList = await changeSetsApi.listChangeSets({
            workspaceId: WORKSPACE_ID,
          });
          const head = (changeSetList.data.changeSets as ChangeSetItem[]).find((
            cs,
          ) => cs.isHead);
          if (!head) {
            return errorResponse({
              message:
                "No HEAD change set found; this is a bug! Tell the user we are sorry.",
            });
          }
          changeSetId = head.id;
        } catch (error) {
          const errorMessage = error instanceof Error
            ? error.message
            : String(error);
          return errorResponse({
            message:
              `No change set id was provided, and we could not find HEAD; this is a bug! Tell the user we are sorry: ${errorMessage}`,
          });
        }
      }
      const siApi = new SchemasApi(apiConfig);
      try {
        let args: {
          workspaceId: string;
          changeSetId: string;
          schemaId?: string | null;
          schema?: string | null;
        };

        if (isValid(schemaNameOrId)) {
          args = {
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId!,
            schemaId: schemaNameOrId,
            schema: null,
          };
        } else {
          args = {
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId!,
            schema: schemaNameOrId,
            schemaId: null,
          };
        }

        const response = await siApi.findSchema(args);
        const responseData: NonNullable<FindSchemaOutput["data"]> = {
          schemaId: response.data.schemaId,
          schemaName: response.data.schemaName,
        };

        const docs = getDocumentationForService(responseData.schemaName);
        responseData.description = docs.description;
        responseData.link = docs.link;

        return successResponse(
          responseData,
          "You can use a web search to find the cloudformation schema",
        );
      } catch (error) {
        return errorResponse(error);
      }
    },
  );
}
