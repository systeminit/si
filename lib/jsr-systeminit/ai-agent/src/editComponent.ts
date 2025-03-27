/**
 * Edit component functionality for AI Agent
 *
 * This module provides functionality to edit existing AWS components in System Initiative
 * based on natural language instructions. It ensures that property names match the
 * case-sensitive requirements of CloudFormation schemas.
 *
 * @module
 */

// Using 'any' to handle the complexity of CloudFormation schema structure
// Note: we've disabled the no-explicit-any lint rule in deno.json
type SchemaProperty = Record<string, any>;

import OpenAI from "jsr:@openai/openai@^4";
import { getServiceByName } from "jsr:@systeminit/cf-db@0";
import { extractFields } from "./extractFields.ts";
import { editComponentPrompt } from "./prompts/edit-component-prompt.ts";

/**
 * Schema for the edit component response
 *
 * Defines the expected JSON structure for the AI model's response when editing components.
 * Includes status and operations to update components with SI and domain properties.
 */
const EditComponentResponse = {
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "status": {
      "type": "string",
      "enum": ["ok", "error"],
    },
    "ops": {
      "type": "object",
      "properties": {
        "update": {
          "type": "object",
          "additionalProperties": {
            "type": "object",
            "properties": {
              "properties": {
                "type": "object",
                "properties": {
                  "si": {
                    "type": "object",
                    "properties": {
                      "name": {
                        "type": ["string", "null"],
                      },
                    },
                    "required": ["name"],
                    "additionalProperties": false,
                  },
                  "domain": {
                    "type": "string",
                    "description":
                      "JSON-serialized object containing CloudFormation properties to update",
                  },
                },
                "required": ["si", "domain"],
                "additionalProperties": false,
              },
            },
            "additionalProperties": false,
            "required": ["properties"],
          },
        },
      },
      "additionalProperties": false,
    },
  },
  "additionalProperties": false,
  "required": ["status", "ops"],
};

/**
 * Helper function to check if a component's domain properties have case mismatches
 * compared to CloudFormation schema
 *
 * AWS CloudFormation is case-sensitive, so this function validates that property names
 * match the expected casing in the schema to prevent deployment errors.
 *
 * @param kind The CloudFormation resource type (e.g., "AWS::EC2::Instance")
 * @param domain The domain object to check for case mismatches
 * @returns An object with error details if case issues found, otherwise null
 */
// Helper to find correct case of a property in the schema
function findCorrectCaseProperty(
  key: string,
  schemaProps: SchemaProperty,
): string | null {
  // Exact match - no case issue
  if (key in schemaProps) {
    return key;
  }

  // Case-insensitive match
  const lowerKey = key.toLowerCase();
  for (const propName in schemaProps) {
    if (propName.toLowerCase() === lowerKey) {
      return propName; // Return the correctly cased property name
    }
  }

  return null; // No match found
}

// Function to recursively check property names against schema
function checkProperties(
  obj: Record<string, unknown>,
  schemaProps: SchemaProperty,
  errors: string[],
  path: string = "",
) {
  // Check each property in the object
  for (const key in obj) {
    // Find the correctly cased property in the schema
    const propKey = findCorrectCaseProperty(key, schemaProps);

    // If the property exists in the schema but with different case
    if (propKey && propKey !== key) {
      errors.push(`${path ? path + "." : ""}${key} should be ${propKey}`);
    }

    // Recursively check nested objects/arrays if schema info exists
    const value = obj[key];
    const schemaProp = schemaProps[propKey || key];

    if (schemaProp && value !== null && value !== undefined) {
      // Handle properties that are objects
      if (
        typeof value === "object" && !Array.isArray(value) &&
        schemaProp.properties
      ) {
        checkProperties(
          value as Record<string, unknown>,
          schemaProp.properties,
          errors,
          path ? `${path}.${propKey || key}` : propKey || key,
        );
      }

      // Handle arrays with item type definitions
      if (Array.isArray(value)) {
        // Check if the array has items with properties defined
        const itemProps = schemaProp.itemType?.properties || // Try itemType
          schemaProp.items?.properties || // Try items
          (schemaProp.properties && schemaProp.properties.properties); // Deeper nesting

        if (itemProps) {
          for (let i = 0; i < value.length; i++) {
            const item = value[i];
            if (item && typeof item === "object" && !Array.isArray(item)) {
              checkProperties(
                item as Record<string, unknown>,
                itemProps,
                errors,
                path
                  ? `${path}.${propKey || key}[${i}]`
                  : `${propKey || key}[${i}]`,
              );
            }
          }
        }
      }
    }
  }
}

export function checkPropertyCaseMismatches(
  kind: string,
  domain: Record<string, unknown>,
): Promise<{ hasErrors: boolean; details: string } | null> | null {
  try {
    // Skip case validation for non-CloudFormation components
    // CloudFormation resource types always follow the pattern AWS::Service::Resource
    if (!kind || !kind.startsWith("AWS::") || !kind.includes("::")) {
      // Silently skip non-CloudFormation types
      return null; // Not a CloudFormation resource type
    }

    // Try to load the CloudFormation schema, but handle case where it doesn't exist
    let service;
    try {
      service = getServiceByName(kind);
    } catch (_schemaError) {
      // If schema doesn't exist, silently skip validation
      return null;
    }

    // Check if we have a valid schema with properties
    if (!service || !service.properties) {
      return null; // Can't check without schema
    }

    const errors: string[] = [];

    // Start the recursive check with top-level properties
    checkProperties(domain, service.properties, errors);

    // Return results if we found any case mismatches
    if (errors.length > 0) {
      return Promise.resolve({
        hasErrors: true,
        details: `Case mismatches found in ${kind}:\n` + errors.join("\n"),
      });
    }

    return null; // No case mismatches found
  } catch (error) {
    // Log but don't fail if we have issues with schema validation
    console.log(`Error checking property case mismatches for ${kind}:`, error);
    return null;
  }
}

/**
 * Edits an existing System Initiative component based on natural language instructions
 *
 * @param componentName - The name of the component to edit
 * @param kind - The AWS CloudFormation resource type
 * @param properties - The existing component properties (si and domain)
 * @param request - Natural language description of the requested changes
 * @param maxRetries - Maximum number of retry attempts if parsing or case errors are found (default: 3)
 * @returns A System Initiative management function response with update operations
 */
export async function editComponent(
  componentName: string,
  kind: string,
  properties: {
    si: {
      name?: string | null;
      type?: string | null;
      protected?: boolean | null;
      color?: string | null;
    };
    domain: Record<string, unknown>;
  },
  request: string,
  maxRetries: number = 3, // Increased to 3 to allow for case mismatch retries
): Promise<{
  status: string;
  ops: {
    update: {
      [name: string]: {
        properties: {
          si: {
            name: string | null;
            type: string | null;
            protected?: boolean | null;
            color?: string | null;
          };
          domain?: Record<string, unknown>;
        };
      };
    };
  };
}> {
  let retryCount = 0;
  let jsonParseError: Error | null = null;
  let invalidDomainJson: string | null = null;
  let caseMismatchErrors: string | null = null;

  while (retryCount <= maxRetries) {
    try {
      // Use the static edit component prompt
      const instructions = editComponentPrompt;

      console.log(`Extracting fields for ${kind} ${componentName}`);
      // Step 2: Use extractFields to analyze the request in context of the existing component
      // We pass the existing properties so the AI can understand what's already set
      const extractResponse = await extractFields(kind, request, {
        si: properties.si,
        domain: properties.domain,
      });
      console.log(
        `Extracting fields finished for ${kind} ${componentName}`,
      );
      console.log(`\n# ${kind} ${properties.si.name} reasoning\n`);
      for (const f of extractResponse.properties) {
        console.log(`## ${f.path.join("/")}\n`);
        console.log("### Reasoning\n");
        console.log(`${f.reasoning}\n`);
        console.log("### Documentation Summary\n");
        console.log(`${f.docSummary}\n`);
        console.log("### Documentation Link\n");
        console.log(`${f.documentationUrl}\n`);
      }

      // Prepare input messages
      const inputMessages: Array<
        { role: "user" | "developer" | "system" | "assistant"; content: string }
      > = [
        {
          role: "developer",
          content: `Component Name: ${componentName}\nKind: ${kind}`,
        },
        {
          role: "developer",
          content: "Current Properties: \n\n" +
            JSON.stringify(properties, null, 2),
        },
        {
          role: "developer",
          content: "Extracted Field Updates: \n\n" +
            JSON.stringify(extractResponse, null, 2),
        },
      ];

      // Add case mismatch feedback if we're retrying due to case issues
      if (retryCount > 0 && caseMismatchErrors) {
        inputMessages.push({
          role: "developer",
          content:
            `ERROR: Your previous response contained property name case mismatches in the domain field for component "${componentName}". ` +
            `AWS CloudFormation is case-sensitive and requires exact property name casing. ` +
            `\n\n${caseMismatchErrors}\n\n` +
            `Please ensure you use the exact property names as specified in the AWS CloudFormation documentation. ` +
            `For AWS::ECS::TaskDefinition, top-level properties use PascalCase (e.g., "ContainerDefinitions"), ` +
            `and nested properties for container definitions also use PascalCase (e.g., "Name", "Image", "PortMappings", "ContainerPort", etc.).`,
        });
      } // Add JSON parse error feedback if we're retrying due to invalid JSON
      else if (retryCount > 0 && jsonParseError && invalidDomainJson) {
        inputMessages.push({
          role: "developer",
          content:
            `ERROR: Your previous response contained invalid JSON in the domain field for component "${componentName}". ` +
            `Please ensure the domain field contains valid JSON that can be parsed. ` +
            `The error was: ${jsonParseError.message}\n\n` +
            `The invalid JSON was: ${invalidDomainJson}\n\n` +
            `Make sure all JSON strings are properly escaped and the structure is valid. ` +
            `CRITICAL: Always preserve the exact case of property keys as they appear in AWS CloudFormation documentation. ` +
            `For example, use "VpcId" not "vpcId", "CidrBlock" not "cidrBlock", "SecurityGroupIds" not "securityGroupIds", etc.`,
        });
      }

      // Add the user request
      inputMessages.push({
        role: "user",
        content: request,
      });

      console.log(`Editing component ${kind} ${componentName} beginning...`);

      // Step 3: Create the edit response using the OpenAI API
      const openai = new OpenAI();
      const response = await openai.responses.create({
        model: "gpt-4o",
        text: {
          format: {
            name: "editComponentResponse",
            type: "json_schema",
            schema: EditComponentResponse,
          },
        },
        instructions,
        input: inputMessages,
      });

      console.log(`Editing component ${kind} ${componentName} ending...`);

      if (response.error) {
        throw new Error(response.error.message);
      }

      // Store output text in a local variable
      const outputText = response.output_text;

      // Parse the response
      const result = JSON.parse(outputText);

      // Track if we need to retry
      let needsRetry = false;
      jsonParseError = null;
      invalidDomainJson = null;
      caseMismatchErrors = null;

      // Deserialize the JSON string in each component's domain property
      if (result.ops?.update) {
        for (const compName in result.ops.update) {
          const component = result.ops.update[compName];
          if (component.properties?.domain) {
            try {
              // Try to parse the JSON string into an object
              if (typeof component.properties.domain === "string") {
                component.properties.domain = JSON.parse(
                  component.properties.domain,
                );
              }

              // Check for property case mismatches
              // No need for extra check here as checkPropertyCaseMismatches already filters non-AWS types
              const caseMismatches = await checkPropertyCaseMismatches(
                kind,
                component.properties.domain,
              );
              if (caseMismatches && caseMismatches.hasErrors) {
                console.log(
                  `Case mismatches in domain properties for ${compName}:`,
                  caseMismatches.details,
                );

                // Store the case mismatch errors for feedback in the next retry
                caseMismatchErrors = caseMismatches.details;

                // Flag that we need to retry
                needsRetry = true;
                break;
              }
            } catch (error) {
              console.log(
                `Failed to parse domain JSON for component ${compName}:`,
                error,
              );

              // Store the error and invalid JSON for feedback in the next retry
              jsonParseError = error instanceof Error
                ? error
                : new Error(String(error));
              invalidDomainJson =
                typeof component.properties.domain === "string"
                  ? component.properties.domain
                  : JSON.stringify(component.properties.domain);

              // Flag that we need to retry
              needsRetry = true;
              break;
            }
          }
        }
      }

      // If we need to retry and have retries left, continue to the next iteration
      if (needsRetry && retryCount < maxRetries) {
        retryCount++;
        if (caseMismatchErrors) {
          console.log(
            `Retrying editComponent due to property case mismatches (attempt ${retryCount}/${maxRetries})`,
          );
        } else {
          console.log(
            `Retrying editComponent due to JSON parse error (attempt ${retryCount}/${maxRetries})`,
          );
        }
        continue;
      }

      // If we don't need to retry, or we're out of retries, return the result
      return result;
    } catch (error) {
      // Handle other errors
      if (retryCount < maxRetries) {
        retryCount++;
        console.log(
          `Error in editComponent, retrying (${retryCount}/${maxRetries}):`,
          error,
        );
        continue;
      } else {
        // If we've used all retries, rethrow the error
        throw error;
      }
    }
  }

  // This should never be reached due to the return inside the try block
  // or the throw inside the catch block, but TypeScript needs it
  throw new Error("Unexpected state in editComponent: all retries failed");
}
