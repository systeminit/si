import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod-v3";
import type { GetPromptResult } from "@modelcontextprotocol/sdk/types.js";

const PromptSchemaRaw = {
  resourceId: z.string().optional().describe("The resource id to import"),
  schemaName: z.string().optional().describe(
    "The schema name for the resource id",
  ),
};
const PromptSchema = z.object(PromptSchemaRaw);

type PromptArgs = z.infer<typeof PromptSchema>;

function prompt(args: PromptArgs): string {
  return `I want to import a resource using the System Initiative MCP server using the following information:

    <resourceId>${args.resourceId}</resourceId>
    <schemaName>${args.schemaName}</schemaName>

  Follow these instructions:

    1. If I did not provide a Schema Name, ask me for one.
    2. If I did not provide a resource id, ask me for one.
    3. Create a new change set, titled "Import $schemaName $resourceId" using the values either providied initially or in follow on responses.
    4. Determine extra attributes for the import call. If the schema represents an AWS resource, you likely need a Region and an AWS Credential, which you should configure with an attribute subscription. Look at the list of components to see if there is one you should use. If there are multiple options, ask the user which one they want to use.
    5. Import the component in the change set.
    6. Examine the imported component in the change set, and determine if any of its attributes should be set by subscription rather than directly. Examples of attributes that should be subscriptions are things that look like 'resourceIds' of other components - for example, a 'VpcId' field likely should be a subscription to an 'AWS::EC2::VPC' component with the same id in its '/resource_value/VpcId' field. Use the component-update tool to set these subscriptions.
    7. If there were fields that should be subscriptions, but no component available in the change set that has the required id, ask me if I want to import that schemaName and resourceId.
`;
}

export function importPrompt(server: McpServer) {
  server.registerPrompt(
    "import-component",
    {
      title: "import-component",
      description: "Import a single component",
      argsSchema: PromptSchemaRaw,
    },
    (args): GetPromptResult => {
      return {
        messages: [{
          role: "user",
          content: {
            type: "text",
            text: prompt(args),
          },
        }],
      };
    },
  );
}
