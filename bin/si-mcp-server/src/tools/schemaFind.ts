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
      return await withAnalytics(name, async () => {
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

        const response = await siApi.findSchema(args);
        const responseData: NonNullable<FindSchemaOutput["data"]> = {
          schemaId: response.data.schemaId,
          schemaName: response.data.schemaName,
        };
        if (responseData.schemaName == "AWS::IAM::User") {
          responseData.description =
            "Specifies an IAM User. For more information, see [IAM User](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_users.html) in the *Amazon IAM User Guide*.";
          responseData.link =
            "https://docs.aws.amazon.com/IAM/latest/UserGuide/id_users.html";
        } else if (responseData.schemaName == "AWS::IAM::Role") {
          responseData.description =
            "Specifies an IAM Role. For more information, see [IAM Roles](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles.html) in the *Amazon IAM User Guide*. Always ensure the AssumeRolePolicyDocument is a prettified JSON string, not a raw object.";
          responseData.link =
            "https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles.html";
        } else if (responseData.schemaName == "AWS::IAM::Group") {
          responseData.description =
            "Specifies an IAM Group. For more information, see [IAM Groups](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_groups.html) in the *Amazon IAM User Guide*.";
          responseData.link =
            "https://docs.aws.amazon.com/IAM/latest/UserGuide/id_groups.html";
        } else if (responseData.schemaName == "AWS::IAM::ManagedPolicy") {
          responseData.description =
            "You use this operation to define a custom IAM policy. For more information, see [IAM Policies](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_create.html) in the *Amazon IAM User Guide*. Always ensure the PolicyDocument is a prettified JSON string, not a raw object.";
          responseData.link =
            "https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_create.html";
        } else if (responseData.schemaName == "AWS::IAM::UserPolicy") {
          responseData.description =
            "You use this operation to attach a managed policy to a user. For more information, see [IAM Policies and permissions in AWS Identity and Access Management](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html) in the *Amazon IAM User Guide*.";
          responseData.link =
            "https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html";
        } else if (responseData.schemaName == "AWS::IAM::RolePolicy") {
          responseData.description =
            "Use this operation to attach a managed policy to a role. For more information, see [IAM Policies and permissions in AWS Identity and Access Management](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html) in the *Amazon IAM User Guide*.";
          responseData.link =
            "https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html";
        } else if (responseData.schemaName == "AWS::IAM::InstanceProfile") {
          responseData.description =
            "You use this operation to manage an Instance Profile. For more information, see [IAM Instance Profiles](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles_use_switch-role-ec2_instance-profiles.html) in the *Amazon IAM User Guide*.";
          responseData.link =
            "https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles_use_switch-role-ec2_instance-profiles.html";
        } else if (responseData.schemaName == "String Template") {
          responseData.description =
            `Formats a string with dynamic JavaScript and data from your components.

This is typically used to format a complex string property in a component, including properties from *other* components via subscriptions. For example, the Role and ManagedPolicy components have text fields where you input a JSON policy document. Frequently, you want to insert ARNs and IDs from other components you want to give permission to, such as a User or Instance, into these documents.

You can look at the rendered value in Code Gen. To use it, subscribe to \`/domain/Rendered/Value\` on the String Template component from the place you want to use it.

## Example

To use this component to fill in strings, you follow these steps:

1. Create a String Template component for your string (e.g. an AWS JSON Policy Document).
2. Set the Template property, inserting \`<%= VariableName %>\` where you want to pull in data:
   \`\`json
   {
       "Version": "2012-10-17",
       "Statement": [
           {
               "Sid": "AllowBucketAccess",
               "Effect": "Allow",
               "Action": [ "s3:PutObject", "s3:GetObject", "s3:ListBucket", "s3:DeleteObject" ],
               "Resource": [
                   "<%= BucketArn %>/*",
                   "<%= BucketArn %>"
               ]
           }
       ]
   }
   \`\`\`
3. Add the variables you referenced in step 1 to the Variables property, and  subscribe to the data you want!

   e.g. add \`BucketArn\` to Variables and subscribe to \`/resource_value/Arn\` on \`MyBucket\`.
4. In the *destination*, subscribe to \`/domain/Rendered/Value\` on your String Template:

   e.g. go to \`PolicyDocument\` on your IAM ManagedPolicy, and subscribe to \`/domain/Rendered/Value\` on the String Template component.

## Syntax

Variables aren't the only thing you can do: String Template uses [eta](https://eta.js.org/docs/intro/template-syntax), a JavaScript-based templating engine that lets you use arbitrary JavaScript. A brief summary:

* Variables: \`<%= VariableName %>\` is replaced with the variable name
* JavaScript: you can use arbitrary JavaScript. For example, \`<%= JSON.stringify(VariableName) %>\` will take a value and put quotes around it for JSON, escaping anything inside it (such as newlines and embedded quotes) to ensure it's valid JSON.
* Control (loops, if statements, variables): you can do more advanced JavaScript with \`<%\`:

  \`\`\`
  Roles:
  <% const roles = ListOfRolesSeparatedByComma.split(','); %>
  <% ListOfRolesSeparatedByComma.split(',').forEach((role) => { %>
    - <%= role %>
  <% }); %>
  \`\`\`
* More advanced syntax can be found in [eta's template syntax cheatsheet](https://eta.js.org/docs/intro/syntax-cheatsheet).
`;
        } else if (responseData.schemaName == "Region") {
          responseData.description =
            `The geography for an AWS Region is the specific physical location of its infrastructure. This can help you meet your regulatory, compliance, and operational requirements.

When you are preparing to deploy a workload, consider which Region or Regions best meet your needs. For example, select a Region that has the AWS services and features that you need. Also, you can lower network latency when you select a Region that is close to the majority of your users.

Your AWS account determines the Regions that are available to you.`;
          responseData.link =
            "https://docs.aws.amazon.com/global-infrastructure/latest/regions/aws-regions.html";
        } else if (responseData.schemaName == "AWS Credential") {
          responseData.description =
            `AWS Credential must be configured *manually* in System Initiative. It cannot be configured by the MCP server.

          You can authenticate with AWS using one of the following methods:

Short-Term Credentials

Set the following environment variables when using temporary AWS security credentials:
	•	AWS_ACCESS_KEY_ID: Your AWS access key associated with an IAM account.
	•	AWS_SECRET_ACCESS_KEY: The secret key for your access key (acts like a password).
	•	AWS_SESSION_TOKEN: Required if you’re using temporary credentials obtained from AWS Security Token Service (STS). For details, refer to the assume-role command documentation: https://docs.aws.amazon.com/cli/latest/userguide/cli-authentication-short-term.html

Long-Term Credentials

Set these environment variables when using permanent AWS IAM credentials:
	•	AWS_ACCESS_KEY_ID: Your AWS access key associated with an IAM account.
	•	AWS_SECRET_ACCESS_KEY: The secret key for your access key (acts like a password).

For more details, visit AWS CLI Authentication with Long-Term Credentials: https://docs.aws.amazon.com/cli/latest/userguide/cli-authentication-user.html

Assume Role

Alternatively, you can authenticate by assuming an IAM role, which provides longer-term, managed authentication. Follow the instructions in our guide: https://docs.systeminit.com/explanation/aws-authentication#assuming-a-role`;
          responseData.link =
            "https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html";
        } else if (responseData.schemaName == "AWS Account") {
          responseData.description =
            "You use this to get details about the AWS Account. For more information, see [AWS STS GetCallerIdentity](https://docs.aws.amazon.com/STS/latest/APIReference/API_GetCallerIdentity.html) in the *Amazon Security Token Service API Reference*.";
          responseData.link =
            "https://docs.aws.amazon.com/STS/latest/APIReference/API_GetCallerIdentity.html";
        } else {
          const docs = getDocumentationForService(responseData.schemaName);
          responseData.description = docs.description;
          responseData.link = docs.link;
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
