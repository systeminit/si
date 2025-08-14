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
    ({ schemaName }): CallToolResult => {
      let responseData: ListSchemaAttributesOutput["data"];

      if (schemaName == "AWS::IAM::User") {
        responseData = {
          schemaName: "AWS::IAM::User",
          "attributes": [
            {
              "name": "UserName",
              "path": "/domain/UserName",
              "required": true,
            },
            {
              "name": "Path",
              "path": "/domain/Path",
              "required": false,
            },
            {
              "name": "PermissionsBoundary",
              "path": "/domain/PermissionsBoundary",
              "required": false,
            },
            {
              "name": "Key",
              "path": "/domain/Tags/[array]/Key",
              "required": true,
            },
            {
              "name": "Value",
              "path": "/domain/Tags/[array]/Value",
              "required": true,
            },
            {
              "name": "IdentityType",
              "path": "/domain/IdentityType",
              "required": false,
            },
          ],
        };
      } else if (schemaName == "AWS::IAM::Role") {
        responseData = {
          schemaName: "AWS::IAM::Role",
          "attributes": [
            {
              "name": "RoleName",
              "path": "/domain/RoleName",
              "required": true,
            },
            {
              "name": "AssumeRolePolicyDocument",
              "path": "/domain/AssumeRolePolicyDocument",
              "required": true,
            },
            {
              "name": "Description",
              "path": "/domain/Description",
              "required": false,
            },
            {
              "name": "MaxSessionDuration",
              "path": "/domain/MaxSessionDuration",
              "required": false,
            },
            {
              "name": "Path",
              "path": "/domain/Path",
              "required": false,
            },
            {
              "name": "PermissionsBoundary",
              "path": "/domain/PermissionsBoundary",
              "required": false,
            },
            {
              "name": "IdentityType",
              "path": "/domain/IdentityType",
              "required": false,
            },
            {
              "name": "Key",
              "path": "/domain/Tags/[array]/Key",
              "required": true,
            },
            {
              "name": "Value",
              "path": "/domain/Tags/[array]/Value",
              "required": true,
            },
            {
              "name": "Region",
              "path": "/domain/extra/Region",
              "required": false,
            },
          ],
        };
      } else if (schemaName == "AWS::IAM::Group") {
        responseData = {
          schemaName: "AWS::IAM::Group",
          "attributes": [
            {
              "name": "GroupName",
              "path": "/domain/GroupName",
              "required": true,
            },
            {
              "name": "Path",
              "path": "/domain/Path",
              "required": false,
            },
            {
              "name": "UserName",
              "path": "/domain/Users/[array]",
              "required": false,
            },
            {
              "name": "IdentityType",
              "path": "/domain/IdentityType",
              "required": false,
            },
          ],
        };
      } else if (schemaName == "AWS::IAM::ManagedPolicy") {
        responseData = {
          schemaName: "AWS::IAM::ManagedPolicy",
          "attributes": [
            {
              "name": "PolicyName",
              "path": "/domain/PolicyName",
              "required": true,
            },
            {
              "name": "PolicyDocument",
              "path": "/domain/PolicyDocument",
              "required": true,
            },
            {
              "name": "Path",
              "path": "/domain/Path",
              "required": false,
            },
            {
              "name": "Description",
              "path": "/domain/Description",
              "required": false,
            },
            {
              "name": "Key",
              "path": "/domain/Tags/[array]/Key",
              "required": true,
            },
            {
              "name": "Value",
              "path": "/domain/Tags/[array]/Value",
              "required": true,
            },
            {
              "name": "Region",
              "path": "/domain/extra/Region",
              "required": false,
            },
          ],
        };
      } else if (schemaName == "AWS::IAM::UserPolicy") {
        responseData = {
          schemaName: "AWS::IAM::UserPolicy",
          "attributes": [
            {
              "name": "UserName",
              "path": "/domain/UserName",
              "required": true,
            },
            {
              "name": "PolicyArn",
              "path": "/domain/PolicyArn",
              "required": true,
            },
            {
              "name": "PolicyType",
              "path": "/domain/PolicyType",
              "required": false,
            },
          ],
        };
      } else if (schemaName == "AWS::IAM::RolePolicy") {
        responseData = {
          schemaName: "AWS::IAM::RolePolicy",
          "attributes": [
            {
              "name": "RoleName",
              "path": "/domain/RoleName",
              "required": true,
            },
            {
              "name": "PolicyArn",
              "path": "/domain/PolicyArn",
              "required": true,
            },
            {
              "name": "PolicyType",
              "path": "/domain/PolicyType",
              "required": false,
            },
            {
              "name": "IdentityType",
              "path": "/domain/IdentityType",
              "required": false,
            },
          ],
        };
      } else if (schemaName == "AWS::IAM::InstanceProfile") {
        responseData = {
          schemaName: "AWS::IAM::InstanceProfile",
          "attributes": [
            {
              "name": "InstanceProfileName",
              "path": "/domain/InstanceProfileName",
              "required": true,
            },
            {
              "name": "Path",
              "path": "/domain/Path",
              "required": false,
            },
            {
              "name": "RoleName",
              "path": "/domain/RoleName",
              "required": false,
            },
            {
              "name": "Key",
              "path": "/domain/Tags/[array]/Key",
              "required": true,
            },
            {
              "name": "Value",
              "path": "/domain/Tags/[array]/Value",
              "required": true,
            },
            {
              "name": "Region",
              "path": "/domain/extra/Region",
              "required": false,
            },
          ],
        };
      } else if (schemaName == "String Template") {
        responseData = {
          schemaName: "String Template",
          "attributes": [
            {
              "name": "Template",
              "path": "/domain/Template",
              "required": false,
            },
            {
              "name": "Value",
              "path": "/domain/Variables/[map]",
              "required": false,
            },
            {
              "name": "Value",
              "path": "/domain/Rendered/Value",
              "required": false,
            },
            {
              "name": "Error",
              "path": "/domain/Rendered/Error",
              "required": false,
            },
          ],
        };
      } else if (schemaName == "Region") {
        responseData = {
          schemaName: "Region",
          "attributes": [
            {
              "name": "region",
              "path": "/domain/region",
              "required": true,
            },
          ],
        };
      } else if (schemaName == "AWS Credential") {
        responseData = {
          schemaName: "AWS Credential",
          "attributes": [
            {
              "name": "SessionToken",
              "path": "/secrets/AWS Credential/SessionToken",
              "required": false,
            },
            {
              "name": "AccessKeyId",
              "path": "/secrets/AWS Credential/AccessKeyId",
              "required": false,
            },
            {
              "name": "SecretAccessKey",
              "path": "/secrets/AWS Credential/SecretAccessKey",
              "required": false,
            },
            {
              "name": "AssumeRole",
              "path": "/secrets/AWS Credential/AssumeRole",
              "required": false,
            },
            {
              "name": "Endpoint",
              "path": "/secrets/AWS Credential/Endpoint",
              "required": false,
            },
          ],
        };
      } else if (schemaName == "AWS Account") {
        responseData = {
          schemaName: "AWS Account",
          "attributes": [
            {
              "name": "Account",
              "path": "/domain/AccountData/Account",
              "required": false,
            },
            {
              "name": "Arn",
              "path": "/domain/AccountData/Arn",
              "required": false,
            },
            {
              "name": "UserId",
              "path": "/domain/AccountData/UserId",
              "required": false,
            },
            {
              "name": "CanonicalUserId",
              "path": "/domain/CanonicalUserId",
              "required": false,
            },
          ],
        };
      } else {
        const attributes = getAttributesForService(schemaName);
        responseData = { schemaName, attributes };
      }
      return successResponse(
        responseData,
        "If this is an AWS resource, the attributes map 1:1 to to the Cloudformation resource, where the path is calculated by looking at the Cloudformation resources nesting. You should look up the documentation for any attribute by its schemaName and path with the schema-attributes-documentation tool before setting any values.",
      );
    },
  );
}
