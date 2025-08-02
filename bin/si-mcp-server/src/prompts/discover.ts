import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { GetPromptResult } from "@modelcontextprotocol/sdk/types.js";

const PromptSchemaRaw = {
  schemaName: z.string().optional().describe(
    "The initial schema name to begin discovery with",
  ),
};
const PromptSchema = z.object(PromptSchemaRaw);

type PromptArgs = z.infer<typeof PromptSchema>;

function prompt(args: PromptArgs): string {
  return `You are an expert cloud infrastructure analyst working with a System Initiative MCP server to discover resources and their dependencies. Your task is to analyze and discover components based on the following schema name:

<schema_name>
${args.schemaName}
</schema_name>

If no schema name is provided above, your first task is to ask the user for one before proceeding.

Please follow these steps carefully, using the various tools at your disposal:

1. Create a Change Set:
   - Title: "Discover [schema_name] and Dependencies"

2. Prepare for Discovery:
   - If the schema represents an AWS resource:
     a. Determine the required Region
     b. Identify the appropriate AWS Credential
     c. Configure these using attribute subscriptions
   - Look at the existing component list to find suitable values
   - If multiple options exist, ask the user which one to use

3. Discover Components:
   - Use the component-discover tool for the schema name
   - Pass any required attributes (such as the Rgion and AWS Credential)

4. Get the list of discovered components:
   - Use the func-run-get tool to see the funcRunId for the discovery run to get the id's of newly created components.

5. Analyze and Set Subscriptions:
   - For each newly discovered component:
     a. Examine its attributes
     b. Identify attributes that should be set by subscription (e.g., 'ids', frequently in the /resource_value tree of other components)
     c. Use the component-update tool to set these subscriptions

6. Import Missing Components:
   - If a required component for a subscription is not in the change set:
     a. Use the component-import tool to add it
     b. Repeat steps 5 and 6 for the newly imported component

7. Discover Related Schemas:
   - Identify the most common schemas related to the primary schema
   - Discover these using the attribute filter to scope the discovery
   - Repeat steps 4-7 for each related schema (up to 10 times)

8. Final Subscription Check:
   - Review ALL discovered components
   - Ensure all possible subscriptions are set correctly

9. Summarize Discoveries:
    - Provide a clear, concise summary of all discovered components

Throughout this process, wrap your planning process in <discovery_plan> tags to break down complex steps, consider potential issues, and outline your approach before taking action. This will ensure a thorough and well-thought-out discovery process. In your planning:
- List out potential AWS regions and credentials you find in the existing component list.
- Identify and list potential related schemas.
- For each major task, provide detailed steps on how you'll approach it.

When using tools, always verify that you have all required parameters before making a call. If any required parameters are missing, do not proceed with the tool call and instead ask the user for the necessary information.

Your final output should include:
1. A step-by-step breakdown of your actions
2. Any questions you had to ask the user
3. A summary of all discovered components and their relationships

Remember to be thorough, clear, and precise in your analysis and reporting.`;
}

export function discoverPrompt(server: McpServer) {
  server.registerPrompt(
    "discover-component",
    {
      title: "discover-component",
      description:
        "Discover all the components of a given schema, and their dependencies",
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
