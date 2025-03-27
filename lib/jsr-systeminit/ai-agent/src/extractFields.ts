/**
 * Extract fields functionality for AI Agent
 *
 * This module provides functionality to extract field values from CloudFormation schemas
 * based on natural language requests. It analyzes AWS resource schemas and determines
 * appropriate property values, returning structured data with paths, documentation URLs,
 * values, and reasoning.
 *
 * @module
 */

import { getServiceByName, loadCfDatabase } from "jsr:@systeminit/cf-db@0";
import { extractFieldsPrompt } from "./prompts/extract-fields-prompt.ts";
import {
  DEFAULT_MODEL,
  DEFAULT_SCHEMA_TEMPERATURE,
  getClient,
  MAX_CONTENT_LENGTH,
} from "./client.ts";

/**
 * Type definition for a single extracted property field
 */
export interface ExtractedField {
  /**
   * The hierarchical path to the property in the CloudFormation schema
   */
  path: string[];

  /**
   * URL to the official AWS documentation for this property
   */
  documentationUrl: string;

  /**
   * Summary of the property documentation
   */
  docSummary: string;

  /**
   * The extracted value for the property
   */
  value: string;

  /**
   * Reasoning for why this value was selected
   */
  reasoning: string;

  /**
   * JSON serialized string of the CloudFormation schema for this property
   * This provides complete type information including constraints for complex nested types
   */
  schemaDefinition?: string;
}

/**
 * Type definition for the extracted fields response
 */
export interface ExtractFieldsResponse {
  /**
   * Array of extracted property fields
   */
  properties: ExtractedField[];
}

/**
 * Schema for extractFields response
 *
 * Defines the expected JSON structure for the AI model's response when extracting field values
 * from CloudFormation schemas, including property paths, documentation, and values.
 */
const ExtractResponse = {
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "properties": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "path": {
            "type": "array",
            "items": {
              "type": "string",
            },
          },
          "documentationUrl": {
            "type": "string",
          },
          "docSummary": {
            "type": "string",
          },
          "value": {
            "type": "string",
          },
          "reasoning": {
            "type": "string",
          },
        },
        additionalProperties: false,
        "required": [
          "path",
          "docSummary",
          "reasoning",
          "documentationUrl",
          "value",
        ],
      },
    },
  },
  additionalProperties: false,
  "required": ["properties"],
};

// MAX_CONTENT_LENGTH is now imported from client.ts

/**
 * Finds the schema definition for a property at the given path
 *
 * @param service The CloudFormation service schema
 * @param path Array of property names forming a path to the desired property
 * @returns The schema definition for the property, or null if not found
 */
function findPropertySchema(service: any, path: string[]): any {
  if (!service || !service.properties || path.length === 0) {
    return null;
  }

  // Start at the top level properties
  let currentSchema = service.properties;
  let currentProperty = null;

  // Traverse the path to find the property schema
  for (let i = 0; i < path.length; i++) {
    const propertyName = path[i];

    // Handle array indices in the path (e.g., "ContainerDefinitions.0.LogConfiguration")
    if (/^\d+$/.test(propertyName)) {
      // This is an array index - we're already using the item schema from the previous level
      // No need to change currentSchema since array items all share the same schema
      continue;
    }

    // Check if the property exists at the current level
    if (!(propertyName in currentSchema)) {
      // Try case-insensitive matching
      const lowerKey = propertyName.toLowerCase();
      let found = false;

      for (const propName in currentSchema) {
        if (propName.toLowerCase() === lowerKey) {
          currentProperty = currentSchema[propName];
          found = true;
          break;
        }
      }

      // If still not found, check if this might be a map type with arbitrary keys
      if (!found) {
        // Look for a property in the schema that's marked as a map
        for (const propName in currentSchema) {
          const prop = currentSchema[propName];
          if (prop.type === "map" || prop.additionalProperties) {
            // Found a map type property - use its schema for all keys
            currentProperty = prop;
            found = true;
            break;
          }
        }
      }

      if (!found) {
        console.log(
          `Property ${propertyName} not found in schema at level ${i} of path ${
            path.join(".")
          }`,
        );
        return null;
      }
    } else {
      currentProperty = currentSchema[propertyName];
    }

    // If this is the last part of the path, return the property schema
    if (i === path.length - 1) {
      return currentProperty;
    }

    // Otherwise, move to the next level in the path
    if (
      currentProperty.type === "array" && currentProperty.itemType?.properties
    ) {
      // Handle array item types
      currentSchema = currentProperty.itemType.properties;
    } else if (
      currentProperty.type === "array" && currentProperty.items?.properties
    ) {
      // Alternative array schema structure
      currentSchema = currentProperty.items.properties;
    } else if (currentProperty.properties) {
      // Handle object property types
      currentSchema = currentProperty.properties;
    } else if (
      currentProperty.type === "map" || currentProperty.additionalProperties
    ) {
      // Handle map types (key-value pairs with arbitrary keys)
      // For maps with additionalProperties, use that as the schema for all values
      if (typeof currentProperty.additionalProperties === "object") {
        // Create a dummy schema that applies to all possible keys
        const valueSchema = currentProperty.additionalProperties;
        currentSchema = { "*": valueSchema };
        // Use a wildcard property for the next iteration
        path[i + 1] = "*";
      } else {
        // For simple map types without additionalProperties,
        // just return the current property as best guess
        return currentProperty;
      }
    } else {
      // If we can't find a valid schema at this level, return the current property
      // This helps with map types and other complex structures
      console.log(
        `No nested properties found at level ${i} of path ${path.join(".")}`,
      );
      return currentProperty; // Return what we found instead of null
    }
  }

  return null; // This should never be reached if the path is valid
}

/**
 * Extracts important information from large CloudFormation schemas
 *
 * Creates a simplified version of the service schema to stay within API token limits
 * while preserving essential property information.
 *
 * @param service The full CloudFormation service schema
 * @returns A summarized version of the schema with essential information
 */
function extractEssentialSchemaInfo(service: any): any {
  if (!service) return null;

  try {
    // Create a simplified version of the schema
    const essential: any = {
      typeName: service.typeName,
      documentation: service.documentation || "",
      properties: {},
    };

    // Copy only the property definitions and their types
    if (service.properties) {
      for (const propName in service.properties) {
        const prop = service.properties[propName];
        essential.properties[propName] = {
          type: prop.type || "unknown",
          required: prop.required || false,
          documentation: prop.documentation || "",
          primitiveType: prop.primitiveType || null,
          primitiveItemType: prop.primitiveItemType || null,
          updateType: prop.updateType || null,
        };
      }
    }

    return essential;
  } catch (error) {
    console.log("Error extracting essential schema info:", error);
    return service; // Fall back to the original if there's an error
  }
}

/**
 * Extracts fields from CloudFormation schemas based on natural language request
 *
 * @param typeName - The AWS CloudFormation resource type (e.g., "AWS::EC2::Instance")
 * @param request - Natural language description of the requested resource configuration
 * @param existingProperties - Optional existing component properties to consider when extracting fields
 * @param maxRetries - Maximum number of retry attempts if parsing errors are found (default: 5)
 * @returns A structured response with property paths, documentation, and values
 */
export async function extractFields(
  typeName: string,
  request: string,
  existingProperties?: Record<string, unknown>,
  maxRetries: number = 5,
): Promise<ExtractFieldsResponse> {
  const instructions = extractFieldsPrompt;
  let retryCount = 0;
  let jsonParseError: Error | null = null;
  let invalidValueField: string | null = null;
  let invalidPropertyIndex: number | null = null;
  let typeError: string | null = null;
  let expectedType: string | null = null;
  let actualType: string | null = null;

  while (retryCount <= maxRetries) {
    // Try to load the CloudFormation database and get the service schema
    // Handle case when typeName is not a CloudFormation resource type
    let service;
    try {
      const _db = await loadCfDatabase({});

      // Only attempt to load schema for CloudFormation resource types
      if (typeName && typeName.startsWith("AWS::") && typeName.includes("::")) {
        service = getServiceByName(typeName);
      }
    } catch (error) {
      console.log("Error loading schema and getting service", error);

      // For non-CloudFormation types or if the schema doesn't exist,
      // return an empty properties array
      return { properties: [] };
    }

    if (!service) {
      // For non-CloudFormation types or if the schema doesn't exist,
      // return an empty properties array
      return { properties: [] };
    }

    // Convert service to JSON string
    const serviceJson = JSON.stringify(service);

    // Check if the service schema is too large for the API
    let serviceContent;
    if (serviceJson.length > MAX_CONTENT_LENGTH) {
      console.log(
        `Service schema for ${typeName} is too large (${serviceJson.length} chars), extracting essential info`,
      );

      // Create a simplified version of the schema with only essential information
      const essentialService = extractEssentialSchemaInfo(service);
      serviceContent = JSON.stringify(essentialService);

      // If it's still too large, we need to truncate
      if (serviceContent.length > MAX_CONTENT_LENGTH) {
        console.log(
          `Even essential info is too large (${serviceContent.length} chars), truncating`,
        );
        serviceContent =
          serviceContent.substring(0, MAX_CONTENT_LENGTH - 1000) +
          "\n... [Schema truncated due to size limitations] ...\n";
      }
    } else {
      serviceContent = serviceJson;
    }

    // Build input messages
    const inputMessages: Array<
      { role: "user" | "developer" | "system" | "assistant"; content: string }
    > = [
      {
        role: "developer",
        content: typeName,
      },
      {
        role: "developer",
        content: "CloudFormation specification JSON: \n\n" + serviceContent,
      },
    ];

    // If we have existing properties, provide them as high-priority context
    if (existingProperties) {
      inputMessages.push({
        role: "developer",
        content: "IMPORTANT - EXISTING COMPONENT PROPERTIES: \n\n" +
          "The following existing properties should be maintained unless explicitly requested to change.\n" +
          "Reuse existing IDs and references where possible for consistency.\n\n" +
          JSON.stringify(existingProperties, null, 2),
      });
    }

    // Add error feedback if we're retrying due to issues with the response
    if (retryCount > 0) {
      // Handle JSON parse errors
      if (
        jsonParseError && invalidValueField !== null &&
        invalidPropertyIndex !== null
      ) {
        inputMessages.push({
          role: "developer",
          content:
            `ERROR: Your previous response contained an invalid JSON string in the 'value' field at property index ${invalidPropertyIndex}. ` +
            `For JSON objects and arrays, the value field should contain valid JSON syntax. ` +
            `Error: ${jsonParseError.message}\n\n` +
            `The invalid value was: ${invalidValueField}\n\n` +
            `Simple strings like "my-instance" don't need to be parsed as JSON and are valid values. ` +
            `But for objects and arrays (values that start with { or [), ensure proper JSON syntax with correct escaping.`,
        });
      } // Handle type mismatch errors
      else if (
        typeError && expectedType !== null && actualType !== null &&
        invalidPropertyIndex !== null
      ) {
        inputMessages.push({
          role: "developer",
          content:
            `ERROR: Your previous response contained a type mismatch in the 'value' field at property index ${invalidPropertyIndex}. ` +
            `The property in the CloudFormation schema expects type ${expectedType}, but the value provided was of type ${actualType}. ` +
            `Error details: ${typeError}\n\n` +
            `Make sure the value matches the expected type in the CloudFormation specification. ` +
            `For numbers, don't use quotes. For strings, use quotes. For booleans, use true/false without quotes. ` +
            `For arrays, use proper array syntax. For objects, use proper object syntax.`,
        });
      }
    }

    // Add the user request as the final message
    inputMessages.push({
      role: "user",
      content: request,
    });

    try {
      const client = getClient();
      const response = await client.responses.create({
        model: DEFAULT_MODEL,
        temperature: DEFAULT_SCHEMA_TEMPERATURE, // Use low temperature for more deterministic output
        text: {
          format: {
            name: "extractedFields",
            type: "json_schema",
            schema: ExtractResponse,
          },
        },
        instructions,
        input: inputMessages,
      });

      if (response.error) {
        throw new Error(response.error.message);
      }

      // Store output text in a local variable
      const outputText = response.output_text;

      // Parse the response
      const result = JSON.parse(outputText) as ExtractFieldsResponse;

      // Add schema definition for each property
      if (service && service.properties) {
        for (const prop of result.properties) {
          try {
            if (prop.path && prop.path.length > 0) {
              // Find the schema definition for this property
              const propSchema = findPropertySchema(service, prop.path);

              if (propSchema) {
                // Add the schema definition as a JSON serialized string
                prop.schemaDefinition = JSON.stringify(propSchema, null, 2);
              }
            }
          } catch (error) {
            console.log(
              `Error adding schema definition for property ${
                prop.path.join(".")
              }: ${error}`,
            );
            // Continue with other properties even if this one fails
          }
        }
      }

      // Check if each property's value field contains valid JSON and matches expected type
      let needsRetry = false;
      jsonParseError = null;
      invalidValueField = null;
      invalidPropertyIndex = null;
      typeError = null;
      expectedType = null;
      actualType = null;

      // Validate all property value fields
      for (let i = 0; i < result.properties.length; i++) {
        const prop = result.properties[i];
        if (prop.value !== undefined) {
          let parsedValue;

          // Only try to parse as JSON if it looks like a JSON object or array
          if (
            prop.value.trim().startsWith("{") ||
            prop.value.trim().startsWith("[")
          ) {
            try {
              // Step 1: Ensure the value is valid JSON if it's meant to be a JSON structure
              parsedValue = JSON.parse(prop.value);
            } catch (error) {
              console.log(
                `Invalid JSON in property value at index ${i}:`,
                error,
              );

              // Store error details for the next retry
              jsonParseError = error instanceof Error
                ? error
                : new Error(String(error));
              invalidValueField = prop.value;
              invalidPropertyIndex = i;

              // Flag that we need to retry
              needsRetry = true;
              break;
            }
          } else {
            // For simple values (strings, numbers, booleans), use the value as is
            parsedValue = prop.value;

            // Step 2: Check if the type matches the CloudFormation specification's expected type
            // Only proceed with type checking if we have a service schema and proper path
            if (
              service && service.properties && prop.path && prop.path.length > 0
            ) {
              // Navigate the schema to find the property definition
              let schemaProps = service.properties;
              let propertyName = prop.path[0];
              let propertyDef = schemaProps[propertyName];

              // For nested properties, traverse the schema using the path
              for (let j = 1; j < prop.path.length && propertyDef; j++) {
                // Handle array item types (using any because CloudFormation schema structure is complex)
                if (
                  propertyDef.type === "array" &&
                  (propertyDef as any).itemType &&
                  (propertyDef as any).itemType.properties
                ) {
                  schemaProps = (propertyDef as any).itemType.properties;
                } // Handle object property types (using any because schema structure varies)
                else if ((propertyDef as any).properties) {
                  schemaProps = (propertyDef as any).properties;
                } // If we can't find the nested schema, break
                else {
                  break;
                }

                propertyName = prop.path[j];
                propertyDef = schemaProps[propertyName];
              }

              // If we found the property definition, check its type
              if (propertyDef) {
                // Get the expected CloudFormation type (using any because schema structure varies)
                const cfnType = (propertyDef as any).primitiveType ||
                  propertyDef.type;

                // Get the JavaScript type of the parsed value
                const jsType = Array.isArray(parsedValue)
                  ? "array"
                  : typeof parsedValue;

                // Mapping of CloudFormation types to JavaScript types
                const typeMappings: Record<string, string[]> = {
                  "String": ["string"],
                  "Number": ["number"],
                  "Double": ["number"],
                  "Integer": ["number"],
                  "Long": ["number"],
                  "Boolean": ["boolean"],
                  "Timestamp": ["string", "number"], // Accept both ISO date strings and timestamps
                  "Json": ["object", "array"], // Accept any valid JSON structure
                  "array": ["array"],
                  "object": ["object"],
                };

                // Check if the value type matches the expected type
                const allowedTypes = typeMappings[cfnType] || ["string"]; // Default to string if unknown type

                if (!allowedTypes.includes(jsType)) {
                  console.log(
                    `Type mismatch at index ${i}: expected ${cfnType} (${
                      allowedTypes.join(" or ")
                    }), got ${jsType}`,
                  );

                  // Store error details for the next retry
                  typeError = `Expected ${cfnType} but received ${jsType}`;
                  expectedType = cfnType;
                  actualType = jsType;
                  invalidPropertyIndex = i;

                  // Flag that we need to retry
                  needsRetry = true;
                  break;
                }

                // Additional validation for numeric types to ensure integers are not provided as floats
                if (
                  cfnType === "Integer" && jsType === "number" &&
                  !Number.isInteger(parsedValue)
                ) {
                  console.log(
                    `Type mismatch at index ${i}: expected Integer but got floating-point number`,
                  );

                  // Store error details for the next retry
                  typeError =
                    "Expected integer but received floating-point number";
                  expectedType = "Integer";
                  actualType = "Float";
                  invalidPropertyIndex = i;

                  // Flag that we need to retry
                  needsRetry = true;
                  break;
                }
              }
            }
          }
        }
      }

      // If we need to retry and have retries left, continue to the next iteration
      if (needsRetry && retryCount < maxRetries) {
        retryCount++;
        if (jsonParseError) {
          console.log(
            `Retrying extractFields due to invalid JSON in value field (attempt ${retryCount}/${maxRetries})`,
          );
        } else if (typeError) {
          console.log(
            `Retrying extractFields due to type mismatch in value field (attempt ${retryCount}/${maxRetries})`,
          );
        }
        continue;
      }

      // If we don't need to retry, or we're out of retries, return the result
      return result;
    } catch (error) {
      // Handle API errors or JSON parsing errors
      if (retryCount < maxRetries) {
        retryCount++;
        console.log(
          `Error in extractFields for ${typeName}, retrying (${retryCount}/${maxRetries}):`,
          error,
        );
        continue;
      } else {
        // If we've used all retries, log the error and return empty result
        console.log(
          `All retries failed in extractFields for ${typeName}:`,
          error,
        );
        return { properties: [] } as ExtractFieldsResponse;
      }
    }
  }

  // This should never be reached due to the return inside the try block
  // or the empty result in the catch block, but TypeScript needs it
  return { properties: [] } as ExtractFieldsResponse;
}
