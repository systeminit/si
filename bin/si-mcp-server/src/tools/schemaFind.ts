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
import { ChangeSetItem } from "../data/changeSets.ts";
import { isValid } from "ulid";

const name = "schema-find";
const title = "Find component schemas";
const description = `<description>Finds component schemas by name or Schema ID. Returns the Schema ID, Name, Description, and external documentation Link. On failure, returns error details. When looking for AWS Schemas, you can use the AWS Cloudformation Resource name (examples: AWS::EC2::Instance, AWS::Bedrock::Agent, or AWS::ControlTower::EnabledBaseline)</description><usage>Use this tool to find if a schema exists in System Initiative, to look up the Schema Name or Schema ID if you need it, or to display high level information about the schema.</usage>`;

const FindSchemaInputSchemaRaw = {
  changeSetId: z
    .string()
    .optional()
    .describe(
      "The change set to look up the schema in; if not provided, HEAD will be used",
    ),
  schemaNameOrId: z
    .string()
    .describe("The Schema Name or Schema ID to retrieve"),
};

const FindSchemaOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z
    .object({
      schemaId: z.string().describe("the schema id"),
      schemaName: z.string().describe("the name of the schema"),
      description: z
        .string()
        .optional()
        .describe(
          "a description of the schema, frequently containing documentation",
        ),
      link: z
        .string()
        .url()
        .optional()
        .describe(
          "an external URL that contains documentation about what this schema is modeling",
        ),
    })
    .optional()
    .describe("the schema information"),
};
const FindSchemaOutputSchema = z.object(FindSchemaOutputSchemaRaw);
type FindSchemaOutput = z.infer<typeof FindSchemaOutputSchema>;

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
      return await withAnalytics(name, async () => {
        if (!changeSetId) {
          const changeSetsApi = new ChangeSetsApi(apiConfig);
          try {
            const changeSetList = await changeSetsApi.listChangeSets({
              workspaceId: WORKSPACE_ID,
            });
            const head = (
              changeSetList.data.changeSets as ChangeSetItem[]
            ).find((cs) => cs.isHead);
            if (!head) {
              return errorResponse({
                message:
                  "No HEAD change set found; this is a bug! Tell the user we are sorry.",
              });
            }
            changeSetId = head.id;
          } catch (error) {
            const errorMessage =
              error instanceof Error ? error.message : String(error);
            return errorResponse({
              message: `No change set id was provided, and we could not find HEAD; this is a bug! Tell the user we are sorry: ${errorMessage}`,
            });
          }
        }
        const siApi = new SchemasApi(apiConfig);
        try {
          if (schemaNameOrId.startsWith("AWS::IAM")) {
            switch (schemaNameOrId) {
              case "AWS::IAM::User":
              case "AWS::IAM::Role":
              case "AWS::IAM::Group":
              case "AWS::IAM::ManagedPolicy":
              case "AWS::IAM::UserPolicy":
              case "AWS::IAM::RolePolicy":
              case "AWS::IAM::InstanceProfile":
                break;
              default:
                return errorResponse({
                  message:
                    "AWS::IAM schema not found. Use one of AWS::IAM::User, AWS::IAM::Role, AWS::IAM::RolePolicy, AWS::IAM::UserPolicy, AWS::IAM::ManagedPolicy, AWS::IAM::InstanceProfile, or AWS::IAM::Group.",
                });
            }
          }

          let schemaId = "";
          if (!isValid(schemaNameOrId)) {
            try {
              const response = await siApi.findSchema({
                workspaceId: WORKSPACE_ID,
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

          const response = await siApi.getDefaultVariant({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId!,
            schemaId: schemaId,
          });

          if (response.status === 202) {
            return errorResponse({
              message:
                "The data is not yet available for this request. Try again in a few seconds",
            });
          }

          const responseData: NonNullable<FindSchemaOutput["data"]> = {
            schemaId: schemaNameOrId,
            schemaName: response.data.displayName,
            description: response.data.description,
          };

          if (response.data.link) {
            responseData.link = response.data.link;
          }

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
