// deno-lint-ignore-file no-explicit-any
import { getServiceByName, loadCfDatabase } from "@systeminit/cf-db";
import { logger } from "../logger.ts";
import { z } from "zod";

logger.debug("Loading AWS CloudFormation Database");
await loadCfDatabase({});

export const ServiceDocumentationSchemaRaw = {
  description: z.string().describe("A high level overview of the schema"),
  link: z.string().optional().describe(
    "A URL you can look up on the web to gain more information about the service",
  ),
};
export const ServiceDocumentationSchema = z.object(
  ServiceDocumentationSchemaRaw,
).describe("Documentation about a schema");
type ServiceDocumentation = z.infer<typeof ServiceDocumentationSchema>;

export function getDocumentationForService(
  serviceName: string,
): ServiceDocumentation {
  try {
    const cfSchema = getServiceByName(serviceName);
    const result: ServiceDocumentation = {
      description: cfSchema.description,
    };
    if (
      cfSchema.documentationUrl && typeof cfSchema.documentationUrl === "string"
    ) {
      result.link = cfSchema.documentationUrl;
    } else if (
      cfSchema.resourceLink && typeof cfSchema.resourceLink === "string"
    ) {
      result.link = cfSchema.resourceLink;
    }
    return result;
  } catch (_error) {
    return {
      description: `No documentation available for ${serviceName}`,
    };
  }
}

const AttributesForServiceSchema = z.array(
  z.object({
    name: z.string().describe("the attributes name"),
    path: z.string().describe(
      "the absolute path of the attribute, which you can use to look up its documentation",
    ),
    required: z.boolean().describe("if this attribute is required"),
  }).describe("an attribute"),
).describe("array of attributes");
type AttributesForService = z.infer<typeof AttributesForServiceSchema>;

export function getAttributesForService(
  serviceName: string,
): AttributesForService {
  try {
    const cfSchema = getServiceByName(serviceName);
    const attributes: AttributesForService = [];

    // Get required properties from schema
    const createOnlyProps = new Set(cfSchema.createOnlyProperties || []);
    const readOnlyProps = new Set(cfSchema.readOnlyProperties || []);
    const writeOnlyProps = new Set(cfSchema.writeOnlyProperties || []);

    // deno-lint-ignore no-inner-declarations
    function isRequired(path: string, localRequired: string[] = []): boolean {
      return createOnlyProps.has(path) ||
        readOnlyProps.has(path) ||
        writeOnlyProps.has(path) ||
        localRequired.includes(path.split("/").pop() || "");
    }

    // deno-lint-ignore no-inner-declarations
    function walkProperties(
      obj: any,
      basePath: string,
      requiredProps: string[] = [],
    ): void {
      if (!obj || typeof obj !== "object") return;

      for (const [key, value] of Object.entries(obj)) {
        if (!value || typeof value !== "object") continue;

        const currentPath = `${basePath}/${key}`;
        const prop = value as any;

        if (prop.type === "array") {
          // Handle arrays - use [array] in path for array elements
          const arrayPath = `${currentPath}/[array]`;

          if (prop.items?.type === "object" && prop.items.properties) {
            // Array of objects - recurse into object properties
            walkProperties(
              prop.items.properties,
              arrayPath,
              prop.items.required || [],
            );
          } else {
            // Simple array - add the array property itself
            attributes.push({
              name: key,
              path: currentPath,
              required: isRequired(currentPath, requiredProps),
            });
          }
        } else if (prop.type === "object") {
          if (
            prop.additionalProperties &&
            typeof prop.additionalProperties === "object"
          ) {
            // Handle maps - use '[map]' in path for map values
            const mapPath = `${currentPath}/[map]`;
            const mapValueType = prop.additionalProperties;

            if (mapValueType.type === "object" && mapValueType.properties) {
              walkProperties(
                mapValueType.properties,
                mapPath,
                mapValueType.required || [],
              );
            } else {
              attributes.push({
                name: key,
                path: currentPath,
                required: isRequired(currentPath, requiredProps),
              });
            }
          } else if (prop.properties) {
            // Regular nested object
            walkProperties(prop.properties, currentPath, prop.required || []);
          }
        } else {
          // Simple property (string, integer, boolean, etc.)
          attributes.push({
            name: key,
            path: currentPath,
            required: isRequired(currentPath, requiredProps),
          });
        }
      }
    }

    // Start walking from the root properties with /domain prefix
    if (cfSchema.properties) {
      walkProperties(cfSchema.properties, "/domain");
    }

    return attributes;
  } catch (error) {
    logger.error(`Failed to get attributes for service ${serviceName}:`, error);
    return [];
  }
}

function getManualSchemaDocumentation(
  schemaName: string,
  schemaAttributePath: string,
): string | null {
  switch (schemaName) {
    case "AWS::IAM::User":
      switch (schemaAttributePath) {
        case "/domain/UserName":
          return "The name of the user. Do not include the path in this value.\nIAM user, group, role, and policy names must be unique within the account. Names are not distinguished by case. For example, you cannot create resources named both `MyResource` and `myresource`.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateUser.html#API_CreateUser_RequestParameters";
        case "/domain/Path":
          return "The path for the user name. For more information about paths, see [IAM identifiers](https://docs.aws.amazon.com/IAM/latest/UserGuide/Using_Identifiers.html) in the IAM User Guide.\nThis parameter is optional. If it is not included, it defaults to a slash (/).\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of either a forward slash (/) by itself or a string that must begin and end with forward slashes. In addition, it can contain any ASCII character from the ! (\\u0021) through the DEL character (\\u007F), including most punctuation characters, digits, and upper and lowercased letters.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateUser.html#API_CreateUser_RequestParameters";
        case "/domain/PermissionsBoundary":
          return "The ARN of the managed policy that is used to set the permissions boundary for the user.\nA permissions boundary policy defines the maximum permissions that identity-based policies can grant to an entity, but does not grant permissions. Permissions boundaries do not define the maximum permissions that a resource-based policy can grant to an entity. To learn more, see [Permissions boundaries for IAM entities](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_boundaries.html) in the IAM User Guide.\nFor more information about policy types, see [Policy types](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html#access_policy-types) in the IAM User Guide.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateUser.html#API_CreateUser_RequestParameters";
        case "/domain/Tags/[array]/Key":
          return "The tag key.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_Tag.html";
        case "/domain/Tags/[array]/Value":
          return "The tag value.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_Tag.html";
        case "/domain/IdentityType":
          return "The type of identity this user represents. Used to categorize the user within the IAM system.";
        default:
          return null;
      }

    case "AWS::IAM::Role":
      switch (schemaAttributePath) {
        case "/domain/RoleName":
          return "The name of the role. Do not include the path in this value.\nIAM user, group, role, and policy names must be unique within the account. Names are not distinguished by case. For example, you cannot create resources named both `MyResource` and `myresource`.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html#API_CreateRole_RequestParameters";
        case "/domain/AssumeRolePolicyDocument":
          return "The trust relationship policy document as a prettified JSON string, that grants an entity permission to assume the role.\nUpon success, the response includes the same trust policy in JSON format.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html#API_CreateRole_RequestParameters";
        case "/domain/Description":
          return "A description of the role.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html#API_CreateRole_RequestParameters";
        case "/domain/MaxSessionDuration":
          return "The maximum session duration (in seconds) that you want to set for the specified role. If you do not specify a value for this setting, the default value of one hour is applied. This setting can have a value from 1 hour to 12 hours.\nAnyone who assumes the role from the CLI or API can use the `DurationSeconds` API parameter or the `duration-seconds` CLI parameter to request a longer session. The `MaxSessionDuration` setting determines the maximum duration that can be requested using the `DurationSeconds` parameter. If users don't specify a value for the `DurationSeconds` parameter, their security credentials are valid for one hour by default. This applies when you use the `AssumeRole*` API operations or the `assume-role*` CLI operations but does not apply when you use those operations to create a console URL. For more information, see [Using IAM roles](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles_use.html) in the IAM User Guide.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html#API_CreateRole_RequestParameters";
        case "/domain/Path":
          return "The path to the group. For more information about paths, see [IAM identifiers](https://docs.aws.amazon.com/IAM/latest/UserGuide/Using_Identifiers.html) in the IAM User Guide.\nThis parameter is optional. If it is not included, it defaults to a slash (/).\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of either a forward slash (/) by itself or a string that must begin and end with forward slashes. In addition, it can contain any ASCII character from the ! (\\u0021) through the DEL character (\\u007F), including most punctuation characters, digits, and upper and lowercased letters.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html#API_CreateRole_RequestParameters";
        case "/domain/PermissionsBoundary":
          return "The ARN of the managed policy that is used to set the permissions boundary for the role.\nA permissions boundary policy defines the maximum permissions that identity-based policies can grant to an entity, but does not grant permissions. Permissions boundaries do not define the maximum permissions that a resource-based policy can grant to an entity. To learn more, see [Permissions boundaries for IAM entities](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_boundaries.html) in the IAM User Guide.\nFor more information about policy types, see [Policy types](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html#access_policy-types) in the IAM User Guide.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html#API_CreateRole_RequestParameters";
        case "/domain/IdentityType":
          return "The type of identity this role represents. Used to categorize the role within the IAM system.";
        case "/domain/Tags/[array]/Key":
          return "The tag key.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_Tag.html";
        case "/domain/Tags/[array]/Value":
          return "The tag value.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_Tag.html";
        case "/domain/extra/Region":
          return "The AWS region where this role is managed. While IAM roles are global, this specifies which region context to use for management operations.";
        default:
          return null;
      }

    case "AWS::IAM::Group":
      switch (schemaAttributePath) {
        case "/domain/GroupName":
          return "The name of the group. Do not include the path in this value.\nIAM user, group, role, and policy names must be unique within the account. Names are not distinguished by case. For example, you cannot create resources named both `MyResource` and `myresource`.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateGroup.html#API_CreateGroup_RequestParameters";
        case "/domain/Path":
          return "The path to the group. For more information about paths, see [IAM identifiers](https://docs.aws.amazon.com/IAM/latest/UserGuide/Using_Identifiers.html) in the IAM User Guide.\nThis parameter is optional. If it is not included, it defaults to a slash (/).\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of either a forward slash (/) by itself or a string that must begin and end with forward slashes. In addition, it can contain any ASCII character from the ! (\\u0021) through the DEL character (\\u007F), including most punctuation characters, digits, and upper and lowercased letters.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateGroup.html#API_CreateGroup_RequestParameters";
        case "/domain/Users/[array]":
          return "The name of the user to add.\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: `_+=,.@-`.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_AddUserToGroup.html#API_AddUserToGroup_RequestParameters";
        case "/domain/IdentityType":
          return "The type of identity this group represents. Used to categorize the group within the IAM system.";
        default:
          return null;
      }

    case "AWS::IAM::ManagedPolicy":
      switch (schemaAttributePath) {
        case "/domain/PolicyName":
          return 'The friendly name of the policy.\nIAM user, group, role, and policy names must be unique within the account. Names are not distinguished by case. For example, you cannot create resources named both "MyResource" and "myresource".\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreatePolicy.html#API_CreatePolicy_RequestParameters';
        case "/domain/PolicyDocument":
          return "The JSON policy document as a prettified JSON string that you want to use as the content for the new IAM policy. To learn more about JSON policy grammar, see [Grammar of the IAM JSON policy language](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html) in the IAM User Guide.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreatePolicy.html#API_CreatePolicy_RequestParameters";
        case "/domain/Path":
          return "The path for the policy. For more information about paths, see [IAM identifiers](https://docs.aws.amazon.com/IAM/latest/UserGuide/Using_Identifiers.html) in the IAM User Guide.\nThis parameter is optional. If it is not included, it defaults to a slash (/).\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of either a forward slash (/) by itself or a string that must begin and end with forward slashes. In addition, it can contain any ASCII character from the ! (\\u0021) through the DEL character (\\u007F), including most punctuation characters, digits, and upper and lowercased letters.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreatePolicy.html#API_CreatePolicy_RequestParameters";
        case "/domain/Description":
          return 'A friendly description of the policy.\nTypically used to store information about the permissions defined in the policy. For example, "Grants access to production DynamoDB tables".\nThe policy description is immutable. After a value is assigned, it cannot be changed.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreatePolicy.html#API_CreatePolicy_RequestParameters';
        case "/domain/Tags/[array]/Key":
          return "The tag key.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_Tag.html";
        case "/domain/Tags/[array]/Value":
          return "The tag value.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_Tag.html";
        case "/domain/extra/Region":
          return "The AWS region where this policy is managed. While IAM policies are global, this specifies which region context to use for management operations.";
        default:
          return null;
      }

    case "AWS::IAM::UserPolicy":
      switch (schemaAttributePath) {
        case "/domain/UserName":
          return "The name (friendly name, not ARN) of the IAM user to attach the policy to.\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: `_+=,.@-`\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_AttachUserPolicy.html#API_AttachUserPolicy_RequestParameters";
        case "/domain/PolicyArn":
          return "The Amazon Resource Name (ARN) of the IAM policy you want to attach.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_AttachUserPolicy.html#API_AttachUserPolicy_RequestParameters";
        case "/domain/PolicyType":
          return "The type of policy attachment. Specifies whether this is an inline policy or a managed policy attachment.";
        default:
          return null;
      }

    case "AWS::IAM::RolePolicy":
      switch (schemaAttributePath) {
        case "/domain/RoleName":
          return "The name (friendly name, not ARN) of the role to attach the policy to.\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: `_+=,.@-`\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_AttachRolePolicy.html#API_AttachRolePolicy_RequestParameters";
        case "/domain/PolicyArn":
          return "The Amazon Resource Name (ARN) of the IAM policy you want to attach.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_AttachRolePolicy.html#API_AttachRolePolicy_RequestParameters";
        case "/domain/PolicyType":
          return "The type of policy attachment. Specifies whether this is an inline policy or a managed policy attachment.";
        case "/domain/IdentityType":
          return "The type of identity this policy attachment represents. Used to categorize the policy attachment within the IAM system.";
        default:
          return null;
      }

    case "AWS::IAM::InstanceProfile":
      switch (schemaAttributePath) {
        case "/domain/InstanceProfileName":
          return "The name of the instance profile to create.\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: `_+=,.@-`\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateInstanceProfile.html#API_CreateInstanceProfile_RequestParameters";
        case "/domain/Path":
          return "The path to the instance profile. For more information about paths, see [IAM Identifiers](https://docs.aws.amazon.com/IAM/latest/UserGuide/Using_Identifiers.html) in the IAM User Guide.\nThis parameter is optional. If it is not included, it defaults to a slash (/).\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of either a forward slash (/) by itself or a string that must begin and end with forward slashes. In addition, it can contain any ASCII character from the (`\\u0021`) through the DEL character (`\\u007F`), including most punctuation characters, digits, and upper and lowercased letters.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateInstanceProfile.html#API_CreateInstanceProfile_RequestParameters";
        case "/domain/RoleName":
          return "The name of the role to add.\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: `_+=,.@-`\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateInstanceProfile.html#API_CreateInstanceProfile_RequestParameters";
        case "/domain/Tags/[array]/Key":
          return "The tag key.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_Tag.html";
        case "/domain/Tags/[array]/Value":
          return "The tag value.\n\nDocumentation: https://docs.aws.amazon.com/IAM/latest/APIReference/API_Tag.html";
        case "/domain/extra/Region":
          return "The AWS region where this instance profile is managed. While IAM instance profiles are global, this specifies which region context to use for management operations.";
        default:
          return null;
      }

    case "String Template":
      switch (schemaAttributePath) {
        case "/domain/Template":
          return 'The string you want to see, with template syntax to fill in Variables\' values.\n\n`<%= VariableName %>` will be replaced with the corresponding Variables\' values.\n\nExample AWS policy document template:\n\n```json\n{\n   "Version": "2012-10-17",\n   "Statement": [\n       {\n           "Sid": "AllowBucketAccess",\n           "Effect": "Allow",\n           "Action": [ "s3:PutObject", "s3:GetObject", "s3:ListBucket", "s3:DeleteObject" ],\n           "Resource": [\n               "<%= BucketArn %>/*",\n               "<%= BucketArn %>"\n           ]\n       }\n   ]\n}\n```';
        case "/domain/Variables/[map]":
          return "A variable value that can be substituted into the template. The key represents the variable name and the value is what gets substituted.";
        case "/domain/Rendered/Value":
          return "The final string, with Variables replaced. This is what you subscribe to when you use the string. DO NOT SET THIS YOURSELF.";
        case "/domain/Rendered/Error":
          return "If there was an error rendering the template, it's stored here.";
        default:
          return null;
      }

    case "Region":
      switch (schemaAttributePath) {
        case "/domain/region":
          return "The AWS region identifier (e.g., us-east-1, eu-west-1). Must be a valid AWS region. See https://docs.aws.amazon.com/global-infrastructure/latest/regions/aws-regions.html for the complete list of available regions.";
        default:
          return null;
      }

    case "AWS Credential":
      switch (schemaAttributePath) {
        case "/secrets/AWS Credential/SessionToken":
          return "The session token for temporary AWS credentials. Used with access key and secret key when assuming roles or using temporary credentials. This field is optional and only required when using temporary credentials.";
        case "/secrets/AWS Credential/AccessKeyId":
          return "The AWS access key ID. This is the public part of the AWS credential pair used for authentication. Required for programmatic access to AWS services.";
        case "/secrets/AWS Credential/SecretAccessKey":
          return "The AWS secret access key. This is the private part of the AWS credential pair and must be kept secure. Never share or expose this value. Required for programmatic access to AWS services.";
        case "/secrets/AWS Credential/AssumeRole":
          return "The ARN of an IAM role to assume when using these credentials. This allows cross-account access or elevated permissions. Optional field used when you need to assume a different role than the one associated with the base credentials.";
        case "/secrets/AWS Credential/Endpoint":
          return "The custom endpoint URL for AWS services. Typically used for testing with LocalStack or custom AWS-compatible services. Optional field - leave empty to use standard AWS endpoints.";
        default:
          return null;
      }

    default:
      return null;
  }
}

export function getSchemaAttributeDocumentation(
  schemaName: string,
  schemaAttributePath: string,
): string | null {
  // We are hardcoding some documentation of props so that the MCP server can handle IAM
  // assets that ARE NOT installed on the graph it's able to query. This allows the MCP
  // server to respond with more appropariate documentation for the associated props and
  // schemas for IAM specifically.
  const manualDoc = getManualSchemaDocumentation(
    schemaName,
    schemaAttributePath,
  );
  if (manualDoc) {
    return manualDoc;
  }

  try {
    const cfSchema = getServiceByName(schemaName);

    // Remove /domain prefix from path for traversal
    const path = schemaAttributePath.startsWith("/domain/")
      ? schemaAttributePath.slice("/domain".length)
      : schemaAttributePath;

    // Split path into segments, filtering out empty strings
    const pathSegments = path.split("/").filter((segment) =>
      segment.length > 0
    );

    if (pathSegments.length === 0 || !cfSchema.properties) {
      return null;
    }

    // deno-lint-ignore no-inner-declarations
    function traversePath(
      obj: any,
      segments: string[],
      currentPath: string,
    ): string | null {
      if (!obj || typeof obj !== "object") return null;

      if (segments.length === 0) {
        return obj.description || null;
      }

      const [currentSegment, ...remainingSegments] = segments;
      const nextProperty = obj[currentSegment];

      if (!nextProperty) return null;

      // Handle array traversal
      if (currentSegment === "[array]") {
        // We're looking for documentation on array items
        if (remainingSegments.length === 0) {
          return obj.description || null;
        }

        if (obj.items && obj.items.properties) {
          return traversePath(
            obj.items.properties,
            remainingSegments,
            `${currentPath}/[array]`,
          );
        }
        return null;
      }

      // Handle map traversal
      if (currentSegment === "[map]") {
        if (remainingSegments.length === 0) {
          return obj.description || null;
        }

        if (obj.additionalProperties && obj.additionalProperties.properties) {
          return traversePath(
            obj.additionalProperties.properties,
            remainingSegments,
            `${currentPath}/[map]`,
          );
        }
        return null;
      }

      // Regular property traversal
      if (nextProperty.type === "array") {
        if (remainingSegments.length === 0) {
          return nextProperty.description || null;
        }

        // Check if next segment is [array]
        if (remainingSegments[0] === "[array]") {
          return traversePath(
            nextProperty,
            remainingSegments,
            `${currentPath}/${currentSegment}`,
          );
        }

        // If looking for a specific field within array items but no [array] specified,
        // fall back to the array's documentation
        return nextProperty.description || null;
      } else if (nextProperty.type === "object") {
        if (remainingSegments.length === 0) {
          return nextProperty.description || null;
        }

        // Handle maps (objects with additionalProperties)
        if (
          nextProperty.additionalProperties && remainingSegments[0] === "[map]"
        ) {
          return traversePath(
            nextProperty,
            remainingSegments,
            `${currentPath}/${currentSegment}`,
          );
        }

        // Regular nested object
        if (nextProperty.properties) {
          const result = traversePath(
            nextProperty.properties,
            remainingSegments,
            `${currentPath}/${currentSegment}`,
          );

          // If we couldn't find documentation for the specific field, fall back to the parent object
          if (result === null && nextProperty.description) {
            return nextProperty.description;
          }

          return result;
        }

        return nextProperty.description || null;
      } else {
        // Simple property
        if (remainingSegments.length === 0) {
          return nextProperty.description || null;
        }

        // Path continues but this is a simple property - invalid path
        return null;
      }
    }

    const result = traversePath(cfSchema.properties, pathSegments, "");

    // If we couldn't find specific documentation, try to find fallback documentation
    // by traversing up the path for array/map containers
    if (
      result === null &&
      (schemaAttributePath.includes("[array]") ||
        schemaAttributePath.includes("[map]"))
    ) {
      // Try to find the parent array or map documentation
      const parentPath = schemaAttributePath.split("/").slice(0, -1).join("/");
      if (parentPath !== schemaAttributePath) {
        return getSchemaAttributeDocumentation(schemaName, parentPath);
      }
    }

    return result;
  } catch (error) {
    logger.error(
      `Failed to get documentation for ${schemaName} path ${schemaAttributePath}:`,
      error,
    );
    return null;
  }
}
