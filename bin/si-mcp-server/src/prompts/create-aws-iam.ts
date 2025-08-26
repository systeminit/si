import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { GetPromptResult } from "@modelcontextprotocol/sdk/types.js";

const CreateIAMPromptSchemaRaw = {
  useCase: z.string().describe(
    "The specific use case or scenario for the IAM setup (e.g., 'EC2 service role', 'Lambda execution role', 'S3 read-only access')",
  ),
  components: z.string().optional().describe(
    "Comma-separated list of specific IAM components to create (Role, User, Group, ManagedPolicy, etc.). If not provided, will be determined based on use case.",
  ),
};
const CreateIAMPromptSchema = z.object(CreateIAMPromptSchemaRaw);

type CreateIAMPromptArgs = z.infer<typeof CreateIAMPromptSchema>;

function createIAMPrompt(args: CreateIAMPromptArgs): string {
  return `You are an expert AWS IAM specialist working with a System Initiative MCP server to create and configure AWS IAM components. Your task is to build secure, well-structured IAM configurations based on the following requirements:

<use_case>
${args.useCase}
</use_case>

${args.components ? `<requested_components>
${args.components}
</requested_components>` : ''}

# Available AWS IAM Schemas in System Initiative

<system-reminder>
These are the ONLY IAM schemas available in System Initiative. Do not suggest or attempt to use any other IAM-related schemas that are not explicitly listed here.
</system-reminder>

The complete list of IAM schemas available:
- **AWS::IAM::Role** - For service roles and cross-account access
- **AWS::IAM::User** - For human users or programmatic access  
- **AWS::IAM::Group** - For grouping users with similar permissions
- **AWS::IAM::ManagedPolicy** - For reusable permission policies
- **AWS::IAM::RolePolicy** - For inline policies attached to roles
- **AWS::IAM::UserPolicy** - For inline policies attached to users
- **AWS::IAM::InstanceProfile** - For EC2 instance roles

**IMPORTANT**: These seven schemas are the ONLY IAM-related schemas available in System Initiative. Do not attempt to create or reference any other IAM schemas (such as AWS::IAM::AccessKey, AWS::IAM::Policy, etc.) as they do not exist in this system.

<system-reminder>
All IAM schemas in System Initiative have full CRUD (Create, Read, Update, Delete) actions available, but these actions use non-standard names that differ from typical AWS API operations. You MUST use the schema query tooling to discover the supported actions for each schema before attempting any operations.
</system-reminder>

**Schema Actions**: Each IAM schema supports full lifecycle management through System Initiative's action system. However, the action names are non-standard and specific to System Initiative. Before working with any schema, use the schema query tools to discover what actions are available for that specific schema type.

# Implementation Steps

Follow these steps carefully:

## 1. Create Change Set
- Title: "Create IAM Setup for [use_case]"

## 2. Analyze Requirements  
- Based on the use case, determine which IAM components are needed
- Consider security best practices (principle of least privilege)
- Plan the relationships between components

## 3. Configure AWS Context
- Determine the required AWS Region
- Identify the appropriate AWS Credential from existing components
- If multiple options exist, ask the user which to use

## 4. Query Schema Actions and Create Core IAM Components
- **FIRST**: Use schema query tools to discover available actions for your target schema
- Start with the primary component (usually Role or User)  
- Use component-create tool with appropriate schema
- Configure all required properties with proper values
- Note: Action names are System Initiative-specific, not standard AWS API names

## 5. Configure Policies (CRITICAL - JSON Formatting)

<system-reminder>
When setting PolicyDocument or AssumeRolePolicyDocument properties, you must NEVER set it as a raw [object Object] or similar placeholders. Always provide complete, valid JSON as a string with proper escaping.
</system-reminder>

**Policy Configuration Rules:**
- NEVER use [object Object] or similar placeholders
- ALWAYS provide complete, valid JSON as a string  
- Use proper JSON escaping for quotes
- Include Version field ("2012-10-17")
- Follow AWS policy syntax exactly

<good-example>
Trust policy for EC2 role:
"{
  \"Version\": \"2012-10-17\",
  \"Statement\": [{
    \"Effect\": \"Allow\",
    \"Principal\": { \"Service\": \"ec2.amazonaws.com\" },
    \"Action\": \"sts:AssumeRole\"
  }]
}"
</good-example>

<bad-example>
Using placeholder values:
[object Object]
or
"{{ trust_policy }}"
or
undefined
</bad-example>

## 6. Create Supporting Components
- Add any required ManagedPolicies with specific permissions
- Create InstanceProfile if needed for EC2 roles
- Set up Groups if organizing multiple users

## 7. Configure Relationships
- Use component-update to set attribute subscriptions
- Link roles to instance profiles
- Attach policies to roles/users/groups
- Set up proper ARN references

## 8. Validate Configuration and Check Qualifications
- Review all components for completeness
- Ensure JSON policies are properly formatted
- Verify relationships are correctly established
- **CRITICAL**: Query the qualifications on each schema to check for validation errors before applying the change set
- Address any qualification failures before proceeding

## 9. Apply Best Practices
- Use descriptive names and descriptions
- Apply appropriate tags for organization
- Set reasonable session durations
- Include permissions boundaries where appropriate

# Planning Framework

<iam_planning>
Before creating components, analyze:
- What AWS services need to be accessed?
- Is this for human users or service roles?  
- What's the minimum set of permissions required?
- Are there existing policies that can be reused?
- What security constraints should be applied?
</iam_planning>

# Common Policy Templates

## S3 Read-Only Access Policy
\`\`\`json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": ["s3:GetObject", "s3:ListBucket"],
    "Resource": ["arn:aws:s3:::bucket-name/*", "arn:aws:s3:::bucket-name"]
  }]
}
\`\`\`

## Lambda Execution Role Trust Policy  
\`\`\`json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow", 
    "Principal": { "Service": "lambda.amazonaws.com" },
    "Action": "sts:AssumeRole"
  }]
}
\`\`\`

## EC2 Instance Role Trust Policy
\`\`\`json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Principal": { "Service": "ec2.amazonaws.com" },
    "Action": "sts:AssumeRole"  
  }]
}
\`\`\`

# Pre-Deployment Validation

<system-reminder>
Before applying any change set, you MUST query the qualifications on all schemas to ensure there are no validation errors. Qualification checks verify that all required properties are set correctly and relationships are properly established.
</system-reminder>

**Qualification Checking Process:**
- Query qualifications for each created component
- Review any validation errors or warnings
- Fix qualification failures before applying the change set
- Only proceed with deployment after all qualifications pass

# Final Deliverables

Your final output should include:
1. **Summary of created IAM components**
2. **Qualification check results and any remediation taken**
3. **Security considerations and recommendations** 
4. **Next steps for testing and deployment**

<system-reminder>
Security is paramount - always follow principle of least privilege and AWS security best practices. Never compromise on proper JSON formatting for policies. Always validate qualifications before deployment.
</system-reminder>`;
}

export function createAwsIamPrompt(server: McpServer) {
  server.registerPrompt(
    "create-aws-iam",
    {
      title: "create-aws-iam",
      description:
        "Create and configure AWS IAM components (roles, users, policies, etc.) for specific use cases with proper JSON policy formatting",
      argsSchema: CreateIAMPromptSchemaRaw,
    },
    (args): GetPromptResult => {
      return {
        messages: [{
          role: "user",
          content: {
            type: "text",
            text: createIAMPrompt(args as CreateIAMPromptArgs),
          },
        }],
      };
    },
  );
}
