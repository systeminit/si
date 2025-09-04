import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";
import { ChangeSetsApi, SchemasApi } from "@systeminit/api-client";
import { ChangeSetItem } from "../data/changeSets.ts";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import { buildAttributesStructure } from "../data/schemaAttributes.ts";

const name = "schema-attributes-list";
const title = "List all the attributes of a schema";
const description =
  `<description>Lists all the attributes of a schema. Returns the schema name and an array of attribute objects that contain the Attribute Name, Path, and if it is Required. On failure, returns error details. Only supports AWS schemas.</description><usage>Use this tool to discover what attributes (sometimes called properties) are available for a schema.</usage>`;

const ListSchemaAttributesInputSchemaRaw = {
  schemaName: z
    .string()
    .describe("The Schema Name to retrieve the attributes for"),
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
    async ({ schemaName }): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        let responseData: ListSchemaAttributesOutput["data"];

        let changeSetId = "";
        const changeSetsApi = new ChangeSetsApi(apiConfig);
        try {
          const changeSetList = await changeSetsApi.listChangeSets({
            workspaceId: WORKSPACE_ID,
          });
          const head = (changeSetList.data.changeSets as ChangeSetItem[]).find(
            (cs) => cs.isHead,
          );
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
              `We could not find the HEAD change set; this is a bug! Tell the user we are sorry: ${errorMessage}`,
          });
        }

        const siApi = new SchemasApi(apiConfig);
        try {
          if (schemaName.startsWith("AWS::IAM")) {
            switch (schemaName) {
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
          try {
            const response = await siApi.findSchema({
              workspaceId: WORKSPACE_ID,
              changeSetId: changeSetId!,
              schema: schemaName,
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

          const attributeDetails = buildAttributesStructure(response.data);
          responseData = {
            schemaName: attributeDetails.schemaName,
            attributes: attributeDetails.attributes,
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
