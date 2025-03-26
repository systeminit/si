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

import { getServiceByName } from "jsr:@systeminit/cf-db@0";
import { extractFields, type ExtractFieldsResponse } from "./extractFields.ts";
import { editComponentPrompt } from "./prompts/edit-component-prompt.ts";
import {
  DEFAULT_MODEL,
  DEFAULT_SCHEMA_TEMPERATURE,
  getClient,
} from "./client.ts";

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
 * Interface defining the error context for retries
 */
interface ErrorContext {
  jsonParseError: Error | null;
  invalidDomainJson: string | null;
  caseMismatchErrors: string | null;
  typeError: string | null;
  expectedType: string | null;
  actualType: string | null;
  typeMismatchPath: string | null;
  componentNameError?: string;
  needsRetry: boolean;
}

/**
 * Type definition for input messages to the OpenAI API
 */
type InputMessage = {
  role: "user" | "developer" | "system" | "assistant";
  content: string;
};

/**
 * Type definition for the component properties
 */
interface ComponentProperties {
  si: {
    name?: string | null;
    type?: string | null;
    protected?: boolean | null;
    color?: string | null;
  };
  domain: Record<string, unknown>;
}

/**
 * Type definition for the edit component response
 */
interface EditComponentResult {
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
}

/**
 * Helper function to check if a component's domain properties have case mismatches
 * compared to CloudFormation schema
 *
 * AWS CloudFormation is case-sensitive, so this function validates that property names
 * match the expected casing in the schema to prevent deployment errors.
 *
 * @param key The key to check in the schema properties
 * @param schemaProps The schema properties to check against
 * @returns The correctly cased property name, or null if not found
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

/**
 * Validates property types against CloudFormation schema types
 *
 * Traverses the given object and checks whether each property's type matches the
 * expected type defined in the CloudFormation resource schema.
 *
 * @param obj The object to check
 * @param schemaProps The CloudFormation schema properties
 * @param errors Array to collect error messages
 * @param path Current property path for error reporting
 * @param propertyType Type information (for tracking found mismatches)
 */
function checkPropertyTypes(
  obj: Record<string, unknown>,
  schemaProps: SchemaProperty,
  errors: string[],
  propertyType: {
    error: string | null;
    expectedType: string | null;
    actualType: string | null;
    path: string | null;
  },
  path: string = "",
) {
  // Skip if we already found an error
  if (propertyType.error) return;

  // Check each property in the object
  for (const key in obj) {
    // Find the correctly cased property in the schema (reusing from case check)
    const propKey = findCorrectCaseProperty(key, schemaProps);
    if (!propKey) continue; // Skip if we can't find the property

    const value = obj[key];
    const schemaProp = schemaProps[propKey];

    // Skip if no schema property definition or value is null/undefined
    if (!schemaProp || value === null || value === undefined) continue;

    // Get the expected CloudFormation type
    const cfnType = (schemaProp as any).primitiveType || schemaProp.type;
    if (!cfnType) continue; // Skip if we can't determine the type

    // Get the JavaScript type of the value
    const jsType = Array.isArray(value) ? "array" : typeof value;

    // Mapping of CloudFormation types to JavaScript types
    const typeMappings: Record<string, string[]> = {
      // Standard CloudFormation primitive types (PascalCase)
      "String": ["string"],
      "Number": ["number"],
      "Double": ["number"],
      "Integer": ["number"],
      "Long": ["number"],
      "Boolean": ["boolean"],
      "Timestamp": ["string", "number"], // Accept both ISO date strings and timestamps
      "Json": ["object", "array"], // Accept any valid JSON structure

      // Handle lowercase types that might come directly from the schema
      "string": ["string"],
      "number": ["number"],
      "integer": ["number"], // lowercase "integer" should map to JavaScript number
      "boolean": ["boolean"],
      "array": ["array"],
      "object": ["object"],
    };

    // Check if the value type matches the expected type
    const allowedTypes = typeMappings[cfnType] || ["string"]; // Default to string if unknown type

    if (!allowedTypes.includes(jsType)) {
      const fullPath = path ? `${path}.${propKey}` : propKey;

      // Standardize on uppercase for error messages for consistency
      const displayType = cfnType.charAt(0).toUpperCase() + cfnType.slice(1);

      // Create a human-readable explanation of the type mismatch
      const explanation =
        `Type mismatch at ${fullPath}: CloudFormation type '${displayType}' requires ${
          allowedTypes.join(" or ")
        } in JavaScript, but got ${jsType} with value ${JSON.stringify(value)}`;

      errors.push(explanation);

      // Store the first type error for feedback
      if (!propertyType.error) {
        propertyType.error =
          `For CloudFormation type '${displayType}', expected JavaScript ${
            allowedTypes.join(" or ")
          } but received ${jsType} with value ${JSON.stringify(value)}`;
        propertyType.expectedType = displayType;
        propertyType.actualType = jsType;
        propertyType.path = fullPath;
      }
      return; // Exit after first error to focus feedback
    }

    // Additional validation for numeric types to ensure integers are not provided as floats
    if (
      (cfnType === "Integer" || cfnType === "integer") && jsType === "number" &&
      !Number.isInteger(value as number)
    ) {
      const fullPath = path ? `${path}.${propKey}` : propKey;
      // Standardize on uppercase for error messages for consistency
      const displayType = cfnType.charAt(0).toUpperCase() + cfnType.slice(1);
      const explanation =
        `Type mismatch at ${fullPath}: CloudFormation type '${displayType}' requires a whole number, but received a floating-point number (${value})`;
      errors.push(explanation);

      // Store the first type error for feedback
      if (!propertyType.error) {
        propertyType.error =
          `For CloudFormation type '${displayType}', expected a whole number but received a floating-point number (${value})`;
        propertyType.expectedType = displayType;
        propertyType.actualType = "Float";
        propertyType.path = fullPath;
      }
      return; // Exit after first error to focus feedback
    }

    // Recursively check nested objects/arrays
    if (value !== null && value !== undefined) {
      // Handle objects with defined properties
      if (
        typeof value === "object" && !Array.isArray(value) &&
        (schemaProp as any).properties
      ) {
        checkPropertyTypes(
          value as Record<string, unknown>,
          (schemaProp as any).properties,
          errors,
          propertyType,
          path ? `${path}.${propKey}` : propKey,
        );
      } // Handle map types with arbitrary keys (common in AWS resources like Options)
      else if (
        typeof value === "object" && !Array.isArray(value) &&
        ((schemaProp as any).type === "map" ||
          (schemaProp as any).additionalProperties)
      ) {
        // For map types, use the additionalProperties schema for all values
        const valueSchema = (schemaProp as any).additionalProperties;
        if (valueSchema && typeof valueSchema === "object") {
          // Create a simplified representation for validation
          const mapValueProps = { "*": valueSchema };
          checkPropertyTypes(
            value as Record<string, unknown>,
            mapValueProps,
            errors,
            propertyType,
            path ? `${path}.${propKey}` : propKey,
          );
        }
      } // Handle arrays with item definitions
      else if (Array.isArray(value)) {
        // Get the item schema definition (if exists)
        const itemProps = (schemaProp as any).itemType?.properties ||
          (schemaProp as any).items?.properties;

        const itemType = (schemaProp as any).itemType?.type ||
          (schemaProp as any).items?.type ||
          (schemaProp as any).itemType?.primitiveType ||
          (schemaProp as any).items?.primitiveType;

        // If we have item properties (array of objects), validate each item
        if (itemProps) {
          for (let i = 0; i < value.length; i++) {
            const item = value[i];
            if (item && typeof item === "object" && !Array.isArray(item)) {
              checkPropertyTypes(
                item as Record<string, unknown>,
                itemProps,
                errors,
                propertyType,
                path ? `${path}.${propKey}[${i}]` : `${propKey}[${i}]`,
              );
            }
          }
        } // If we have a primitive item type but no properties, just validate the type
        else if (itemType) {
          // We validate type at the array level, not per item, to avoid spamming errors
          // This helps with the error message being more helpful
          if (
            value.length > 0 &&
            typeof value[0] !== (typeMappings[itemType] || ["string"])[0]
          ) {
            // Using just the first item as a sample to avoid spamming errors
            const fullPath = path ? `${path}.${propKey}[0]` : `${propKey}[0]`;
            const itemJsType = typeof value[0];
            const expectedJsTypes = typeMappings[itemType] || ["string"];

            errors.push(
              `Type mismatch in array at ${fullPath}: expected ${itemType} but got ${itemJsType}`,
            );

            if (!propertyType.error) {
              propertyType.error = `For array items, expected ${itemType} (${
                expectedJsTypes.join(" or ")
              }) but received ${itemJsType}`;
              propertyType.expectedType = itemType;
              propertyType.actualType = itemJsType;
              propertyType.path = fullPath;
            }
          }
        }
      }
    }
  }
}

/**
 * Checks domain properties for type mismatches against CloudFormation schema
 *
 * @param kind The CloudFormation resource type (e.g., "AWS::EC2::Instance")
 * @param domain The domain object to validate
 * @returns Object with error details if type mismatches found, otherwise null
 */
export function checkPropertyTypeMismatches(
  kind: string,
  domain: Record<string, unknown>,
): {
  error: string;
  expectedType: string;
  actualType: string;
  path: string;
  details: string;
} | null {
  try {
    // Skip validation for non-CloudFormation components
    if (!kind || !kind.startsWith("AWS::") || !kind.includes("::")) {
      return null; // Not a CloudFormation resource type
    }

    // Try to load the CloudFormation schema
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
    const propertyType = {
      error: null as string | null,
      expectedType: null as string | null,
      actualType: null as string | null,
      path: null as string | null,
    };

    // Start the recursive type check with top-level properties
    checkPropertyTypes(domain, service.properties, errors, propertyType);

    // Return results if we found any type mismatches
    if (
      errors.length > 0 && propertyType.error && propertyType.expectedType &&
      propertyType.actualType && propertyType.path
    ) {
      return {
        error: propertyType.error,
        expectedType: propertyType.expectedType,
        actualType: propertyType.actualType,
        path: propertyType.path,
        details: `Type mismatches found in ${kind}:\n` + errors.join("\n"),
      };
    }

    return null; // No type mismatches found
  } catch (error) {
    // Log but don't fail if we have issues with schema validation
    console.log(`Error checking property type mismatches for ${kind}:`, error);
    return null;
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
 * Helper function to log the extraction results
 *
 * @param kind The CloudFormation resource type
 * @param name The component name
 * @param extractResponse The extracted fields response
 */
function logExtractionResults(
  kind: string,
  name: string | null | undefined,
  extractResponse: ExtractFieldsResponse,
) {
  const r = extractionResponseToMarkdown(kind, name, extractResponse);
  console.log(r);
}

/**
 * Helper function to turn extraction response into markdown
 *
 * @param kind The CloudFormation resource type
 * @param name The component name
 * @param extractResponse The extracted fields response
 */
export function extractionResponseToMarkdown(
  kind: string,
  name: string | null | undefined,
  extractResponse: ExtractFieldsResponse,
): string {
  let response = `# ${kind} ${name} reasoning\n\n`;
  for (const field of extractResponse.properties) {
    response += `## ${field.path.join("/")}\n\n`;
    response += "### Reasoning\n\n";
    response += `${field.reasoning}\n\n`;
    response += "### Documentation Summary\n\n";
    response += `${field.docSummary}\n\n`;
    response += "### Documentation Link\n\n";
    response += `${field.documentationUrl}\n\n`;
    response += "### Proposed Value\n\n";
    response += JSON.stringify(field.value, null, 2);
    response += "\n\n";
  }
  return response;
}

/**
 * Helper function to prepare input messages for the OpenAI API
 *
 * @param componentName The name of the component to edit
 * @param kind The CloudFormation resource type
 * @param properties The existing component properties
 * @param extractResponse The extracted fields
 * @param retryCount The current retry count
 * @param errorContext The error context from previous attempts
 * @param request The user's natural language request
 * @returns An array of input messages for the OpenAI API
 */
function prepareInputMessages(
  componentName: string,
  kind: string,
  properties: ComponentProperties,
  extractResponse: unknown,
  retryCount: number,
  errorContext: ErrorContext,
  request: string,
): InputMessage[] {
  const inputMessages: InputMessage[] = [
    {
      role: "developer",
      content:
        `COMPONENT NAME (MUST BE PRESERVED EXACTLY): "${componentName}"\nKind: ${kind}`,
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

  // Add error feedback if we're retrying and there are errors to address
  if (retryCount > 0 && errorContext.needsRetry) {
    // Build a consolidated error message covering all detected issues
    let errorMessage =
      `ERROR: Your previous response contained the following issues:\n\n`;

    // Add case mismatch errors if any
    if (errorContext.caseMismatchErrors) {
      errorMessage +=
        `1. PROPERTY CASE MISMATCHES:\n${errorContext.caseMismatchErrors}\n\n` +
        `AWS CloudFormation is case-sensitive and requires exact property name casing. ` +
        `Please ensure you use the exact property names as specified in the AWS CloudFormation documentation. ` +
        `For most AWS resources, properties use PascalCase (e.g., "InstanceType", "SecurityGroupIds").\n\n`;
    }

    // Add type mismatch errors if any
    if (
      errorContext.typeError && errorContext.expectedType &&
      errorContext.actualType && errorContext.typeMismatchPath
    ) {
      errorMessage += `2. TYPE MISMATCH ERROR:\n` +
        `The property at path '${errorContext.typeMismatchPath}' has incorrect type. ` +
        `${errorContext.typeError}\n\n` +
        `Please ensure all property values match the expected types in the CloudFormation specification:\n` +
        `- For String properties, use string values (with quotes in JSON)\n` +
        `- For Number properties, use numeric values (without quotes)\n` +
        `- For Integer properties, use whole numbers without decimals (e.g., 5 not 5.0)\n` +
        `- For Boolean properties, use true/false literals (without quotes)\n` +
        `- For arrays, use proper array syntax\n\n`;
    }

    // Add JSON parse errors if any
    if (errorContext.jsonParseError && errorContext.invalidDomainJson) {
      errorMessage += `3. JSON PARSING ERROR:\n` +
        `The domain field contains invalid JSON that cannot be parsed. ` +
        `Error: ${errorContext.jsonParseError.message}\n\n` +
        `The invalid JSON was: ${errorContext.invalidDomainJson}\n\n` +
        `Make sure all JSON strings are properly escaped and the structure is valid.\n` +
        `CRITICAL: Always preserve the exact case of property keys as they appear in AWS CloudFormation documentation.\n\n`;
    }

    // Add a final instruction to address all issues
    errorMessage += `Please address ALL of these issues in your response.`;

    // Add the consolidated error message to the input messages
    inputMessages.push({
      role: "developer",
      content: errorMessage,
    });
  }

  // Add the user request
  inputMessages.push({
    role: "user",
    content: request,
  });

  return inputMessages;
}

/**
 * Helper function to make an OpenAI API request
 *
 * @param inputMessages The input messages for the OpenAI API
 * @returns The parsed response from the OpenAI API
 */
async function makeOpenAIRequest(
  inputMessages: InputMessage[],
): Promise<EditComponentResult> {
  const client = getClient();
  const response = await client.responses.create({
    model: DEFAULT_MODEL,
    temperature: DEFAULT_SCHEMA_TEMPERATURE, // Use low temperature for more deterministic, structured output
    text: {
      format: {
        name: "editComponentResponse",
        type: "json_schema",
        schema: EditComponentResponse,
      },
    },
    instructions: editComponentPrompt,
    input: inputMessages,
  });

  if (response.error) {
    throw new Error(response.error.message);
  }

  return JSON.parse(response.output_text) as EditComponentResult;
}

/**
 * Helper function to validate the domain property in the response
 *
 * Checks for multiple types of errors:
 * - JSON parsing errors
 * - Case mismatches in property names
 * - Type mismatches between expected and actual values
 *
 * @param result The result from the OpenAI API
 * @param kind The CloudFormation resource type
 * @returns Error context if validation fails, null otherwise
 */
async function validateResponseDomain(
  result: EditComponentResult,
  kind: string,
): Promise<ErrorContext | null> {
  const errorContext: ErrorContext = {
    jsonParseError: null,
    invalidDomainJson: null,
    caseMismatchErrors: null,
    typeError: null,
    expectedType: null,
    actualType: null,
    typeMismatchPath: null,
    needsRetry: false,
  };

  // Deserialize the JSON string in each component's domain property
  if (result.ops?.update) {
    for (const compName in result.ops.update) {
      const component = result.ops.update[compName];
      if (component.properties?.domain) {
        try {
          // Try to parse the JSON string into an object
          if (typeof component.properties.domain === "string") {
            component.properties.domain = JSON.parse(
              component.properties.domain as string,
            );
          }

          // Continue with validation even if there are multiple issues

          // Check for property case mismatches
          const caseMismatches = await checkPropertyCaseMismatches(
            kind,
            component.properties.domain as Record<string, unknown>,
          );
          if (caseMismatches && caseMismatches.hasErrors) {
            console.log(
              `Case mismatches in domain properties for ${compName}:`,
              caseMismatches.details,
            );

            // Store the case mismatch errors for feedback
            errorContext.caseMismatchErrors = caseMismatches.details;
            errorContext.needsRetry = true;
            // Continue checking for other errors instead of returning immediately
          }

          // Check for type mismatches in the domain property
          const typeMismatches = checkPropertyTypeMismatches(
            kind,
            component.properties.domain as Record<string, unknown>,
          );

          if (typeMismatches) {
            console.log(
              `Type mismatches in domain properties for ${compName}:`,
              typeMismatches.details,
            );

            // Store the type mismatch errors for feedback
            errorContext.typeError = typeMismatches.error;
            errorContext.expectedType = typeMismatches.expectedType;
            errorContext.actualType = typeMismatches.actualType;
            errorContext.typeMismatchPath = typeMismatches.path;
            errorContext.needsRetry = true;
            // Continue checking other components instead of returning immediately
          }
        } catch (error) {
          console.log(
            `Failed to parse domain JSON for component ${compName}:`,
            error,
          );

          // Store the error and invalid JSON for feedback
          errorContext.jsonParseError = error instanceof Error
            ? error
            : new Error(String(error));
          errorContext.invalidDomainJson =
            typeof component.properties.domain === "string"
              ? component.properties.domain
              : JSON.stringify(component.properties.domain);
          errorContext.needsRetry = true;
          // Since we can't parse the JSON, we can't do further validation on this component
          // But we'll continue checking other components
        }
      }
    }
  }

  // Return the error context if any errors were found
  return errorContext.needsRetry ? errorContext : null;
}

/**
 * Helper function to log the reason for retrying
 *
 * @param errorContext The error context
 * @param retryCount The current retry count
 * @param maxRetries The maximum number of retries
 */
function logRetryReason(
  errorContext: ErrorContext,
  retryCount: number,
  maxRetries: number,
): void {
  // Create a list of error types to include in the log message
  const errorTypes: string[] = [];

  if (errorContext.caseMismatchErrors) {
    errorTypes.push("property case mismatches");
  }

  if (errorContext.typeError) {
    errorTypes.push("property type mismatches");
  }

  if (errorContext.jsonParseError) {
    errorTypes.push("JSON parse errors");
  }

  // Combine the error types into a single message
  if (errorTypes.length > 0) {
    console.log(
      `Retrying editComponent due to ${
        errorTypes.join(", ")
      } (attempt ${retryCount}/${maxRetries})`,
    );
  } else {
    console.log(
      `Retrying editComponent (attempt ${retryCount}/${maxRetries})`,
    );
  }
}

/**
 * Proposes edits for a component based on natural language instructions
 *
 * This function extracts fields based on the user's request and provides
 * suggestions for component modifications.
 *
 * @param kind - The AWS CloudFormation resource type
 * @param properties - The existing component properties (si and domain)
 * @param request - Natural language description of the requested changes
 * @returns Extracted field suggestions for component modifications
 */
export async function proposeEdits(
  kind: string,
  properties: ComponentProperties,
  request: string,
): Promise<ExtractFieldsResponse> {
  const componentName = properties.si?.name || "Component";

  // Extract fields for the component
  console.log(`Extracting fields for ${kind} ${componentName}`);
  const extractResponse = await extractFields(kind, request, {
    si: properties.si,
    domain: properties.domain,
  });
  console.log(`Extracting fields finished for ${kind} ${componentName}`);

  // Log the extraction results
  logExtractionResults(kind, properties.si.name, extractResponse);

  return extractResponse;
}

/**
 * Edits an existing System Initiative component based on natural language instructions
 *
 * @param componentName - The name of the component to edit
 * @param kind - The AWS CloudFormation resource type
 * @param properties - The existing component properties (si and domain)
 * @param request - Natural language description of the requested changes
 * @param extractResponse - Field extraction response from proposeEdits (if already available)
 * @param maxRetries - Maximum number of retry attempts if parsing or case errors are found (default: 5)
 * @returns A System Initiative management function response with update operations
 */
export async function editComponent(
  componentName: string,
  kind: string,
  properties: ComponentProperties,
  request: string,
  extractResponse?: ExtractFieldsResponse,
  maxRetries: number = 5,
): Promise<EditComponentResult> {
  let retryCount = 0;
  const errorContext: ErrorContext = {
    jsonParseError: null,
    invalidDomainJson: null,
    caseMismatchErrors: null,
    typeError: null,
    expectedType: null,
    actualType: null,
    typeMismatchPath: null,
    needsRetry: false,
  };

  // If extractResponse is not provided, call proposeEdits to get it
  if (!extractResponse) {
    extractResponse = await proposeEdits(kind, properties, request);
  }

  while (retryCount <= maxRetries) {
    try {
      // Prepare input messages with the extraction response
      const inputMessages = prepareInputMessages(
        componentName,
        kind,
        properties,
        extractResponse,
        retryCount,
        errorContext,
        request,
      );

      console.log(`Editing component ${kind} ${componentName} beginning...`);

      // Make API call and get the parsed result
      const result = await makeOpenAIRequest(inputMessages);

      console.log(`Editing component ${kind} ${componentName} ending...`);

      // Fix component name if it doesn't match exactly what was provided
      if (result.ops?.update && Object.keys(result.ops.update).length === 1) {
        const responseComponentName = Object.keys(result.ops.update)[0];

        if (responseComponentName !== componentName) {
          console.log(
            `Fixing component name from "${responseComponentName}" to "${componentName}"`,
          );

          // Create a new update object with the correct component name
          const correctUpdate = {
            [componentName]: result.ops.update[responseComponentName],
          };

          // Replace the update object with our fixed version
          result.ops.update = correctUpdate;
        }
      }

      // Validate result - returns error context if validation fails
      const validationResult = await validateResponseDomain(result, kind);

      if (validationResult && retryCount < maxRetries) {
        // Update error context with validation errors
        Object.assign(errorContext, validationResult);
        retryCount++;
        logRetryReason(errorContext, retryCount, maxRetries);
        continue;
      }

      // If validation passed or we're out of retries, return the result
      return result;
    } catch (error) {
      // Handle other errors (network errors, API errors, etc.)
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

  // This should never be reached due to the returns inside the loop
  throw new Error("Unexpected state in editComponent: all retries failed");
}
