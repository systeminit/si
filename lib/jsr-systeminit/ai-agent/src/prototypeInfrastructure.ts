/**
 * Infrastructure prototyping functionality for AI Agent
 *
 * This module provides functionality to build infrastructure prototypes in System Initiative
 * based on natural language descriptions. It identifies required AWS resources, extracts
 * appropriate field values from CloudFormation schemas, and creates properly configured
 * components with case-sensitive property names.
 *
 * @module
 */

import OpenAI from "jsr:@openai/openai@^4";
import { getServiceByName } from "jsr:@systeminit/cf-db@0";
import { extractTypes } from "./extractTypes.ts";
import { extractFields } from "./extractFields.ts";
import { checkPropertyCaseMismatches } from "./editComponent.ts";
import { integrationPrompt } from "./prompts/integration-prompt.ts";

/**
 * Schema for the infrastructure prototype response
 *
 * Defines the expected JSON structure for the AI model's response when prototyping infrastructure,
 * including operations to create components with kind and properties.
 */
const IntegrationResponse = {
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
        "create": {
          "type": "object",
          "additionalProperties": {
            "type": "object",
            "properties": {
              "kind": {
                "type": "string",
              },
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
                      "JSON-serialized object containing CloudFormation properties",
                  },
                },
                "required": ["si", "domain"],
                "additionalProperties": false,
              },
            },
            "additionalProperties": false,
            "required": ["kind", "properties"],
          },
        },
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
                      "JSON-serialized object containing CloudFormation properties",
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
 * Maximum content length allowed by OpenAI API
 *
 * Set below the actual limit to maintain a safety buffer for other content.
 * Used when processing large CloudFormation schemas.
 */
const _MAX_CONTENT_LENGTH = 250000; // Prefix with underscore as it's used in other files

/**
 * Extracts important information from large CloudFormation schemas
 *
 * Creates a simplified version of the service schema to stay within API token limits
 * while preserving essential property information.
 *
 * @param service The full CloudFormation service schema
 * @returns A summarized version of the schema with essential information
 */
function _extractEssentialSchemaInfo(service: any): any { // Prefix with underscore as it's used in other files
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
 * Builds a prototype of infrastructure with System Initiative based on natural language description
 *
 * This function is resilient to errors in the LLM output, handling:
 * - Invalid CloudFormation resource types
 * - JSON parsing errors in domain properties
 * - Property case mismatches
 *
 * @param request - Natural language description of the infrastructure needed
 * @param maxRetries - Maximum number of retry attempts (default: 3)
 * @returns A System Initiative management function response with create operations
 */
export async function prototypeInfrastructure(
  request: string,
  maxRetries: number = 3,
): Promise<{
  status: string;
  ops: {
    create: {
      [name: string]: {
        kind: string;
        properties: {
          si: {
            name?: string | null;
            type?: string | null;
            protected?: boolean | null;
            color?: string | null;
          };
          domain: Record<string, unknown>; // Object after deserialization (was JSON-serialized string)
        };
      };
    };
  };
}> {
  // Step 1: First get all the relevant CloudFormation types for this request
  let cfTypes = await extractTypes(request);

  // Step 2: Validate the returned types and retry if necessary
  const invalidTypes: string[] = [];
  const validTypes: { cfType: string; justification: string }[] = [];

  // Instead of checking against getAllCfTypes(), which seems to be empty,
  // validate each type by actually trying to get the service schema
  for (const typeInfo of cfTypes) {
    try {
      const service = getServiceByName(typeInfo.cfType);
      if (service) {
        validTypes.push(typeInfo);
      } else {
        invalidTypes.push(typeInfo.cfType);
        console.log(
          `Invalid CloudFormation type (null service): ${typeInfo.cfType}`,
        );
      }
    } catch (error: any) {
      invalidTypes.push(typeInfo.cfType);
      console.log(
        `Invalid CloudFormation type (error): ${typeInfo.cfType} - ${
          error?.message || "Unknown error"
        }`,
      );
    }
  }

  // If we have invalid types, retry the extraction with feedback
  if (invalidTypes.length > 0) {
    console.log(
      `Retrying type extraction with ${invalidTypes.length} invalid types excluded`,
    );
    cfTypes = await extractTypes(request, invalidTypes);

    // Validate the new types
    const secondRoundInvalid: string[] = [];

    // Check each type with getServiceByName
    for (const typeInfo of cfTypes) {
      try {
        const service = getServiceByName(typeInfo.cfType);
        if (service) {
          // Make sure we don't add duplicates
          if (!validTypes.some((vt) => vt.cfType === typeInfo.cfType)) {
            validTypes.push(typeInfo);
          }
        } else {
          secondRoundInvalid.push(typeInfo.cfType);
          console.log(
            `Still invalid CloudFormation type (null service): ${typeInfo.cfType}`,
          );
        }
      } catch (error: any) {
        secondRoundInvalid.push(typeInfo.cfType);
        console.log(
          `Still invalid CloudFormation type (error): ${typeInfo.cfType} - ${
            error?.message || "Unknown error"
          }`,
        );
      }
    }

    // Log if we still have invalid types after retry
    if (secondRoundInvalid.length > 0) {
      console.log(
        `${secondRoundInvalid.length} types are still invalid after retry`,
      );
    }
  }

  // If we still don't have any valid types, we need to inform the user
  if (validTypes.length === 0) {
    console.warn("No valid CloudFormation types were found for the request");
  }

  // Step 3: Create a parallel processing map for each valid component type
  const fieldExtractionPromises: Record<string, Promise<unknown>> = {};
  const fieldResultsMap: Record<string, any> = {};

  // For each valid CloudFormation type, extract the relevant fields
  for (const { cfType } of validTypes) {
    console.log(`Attempting to extract fields for: ${cfType}`);
    fieldExtractionPromises[cfType] = extractFields(cfType, request);
  }

  // Wait for all field extractions to complete, regardless of success or failure
  const settledResults = await Promise.allSettled(
    Object.values(fieldExtractionPromises),
  );

  // Map the results back to their CF types
  Object.keys(fieldExtractionPromises).forEach((cfType, index) => {
    const result = settledResults[index];
    if (result.status === "fulfilled") {
      fieldResultsMap[cfType] = result.value;
    } else {
      console.log(`Error extracting fields for ${cfType}:`, result.reason);
      // Use an empty result for failed extraction
      fieldResultsMap[cfType] = { properties: [] };
    }
  });

  // If we found no valid types or all field extractions failed, return an error status
  if (Object.keys(fieldResultsMap).length === 0) {
    return {
      status: "error",
      ops: {
        create: {},
      },
    };
  }

  let retryCount = 0;
  let jsonParseError: Error | null = null;
  let invalidDomainJson: string | null = null;
  let caseMismatchErrors: Record<string, string> = {};

  while (retryCount <= maxRetries) {
    try {
      // Use the static integration prompt
      const instructions = integrationPrompt;

      // Convert the field results into a format that can be easily processed
      const formattedResults = JSON.stringify(fieldResultsMap, null, 2);

      // Prepare input messages
      const inputMessages: Array<
        { role: "user" | "developer" | "system" | "assistant"; content: string }
      > = [
        {
          role: "developer",
          content: "CloudFormation types and their fields:\n\n" +
            formattedResults,
        },
      ];

      // Add case mismatch feedback if we're retrying due to case issues
      if (retryCount > 0 && Object.keys(caseMismatchErrors).length > 0) {
        const errorDetails = Object.entries(caseMismatchErrors)
          .map(([componentName, errors]) =>
            `Component "${componentName}": ${errors}`
          )
          .join("\n\n");

        inputMessages.push({
          role: "developer",
          content:
            `ERROR: Your previous response contained property name case mismatches in domain fields. ` +
            `AWS CloudFormation is case-sensitive and requires exact property name casing. ` +
            `\n\n${errorDetails}\n\n` +
            `Please ensure you use the exact property names as specified in the AWS CloudFormation documentation. ` +
            `CloudFormation properties typically use PascalCase (e.g., "VpcId", "CidrBlock", "SecurityGroupIds").`,
        });
      } // Add JSON parse error feedback if we're retrying due to invalid JSON
      else if (retryCount > 0 && jsonParseError && invalidDomainJson) {
        inputMessages.push({
          role: "developer",
          content:
            `ERROR: Your previous response contained invalid JSON in a domain field. ` +
            `Please ensure domain fields contain valid JSON that can be parsed. ` +
            `The error was: ${jsonParseError.message}\n\n` +
            `The invalid JSON was: ${invalidDomainJson}\n\n` +
            `Make sure all JSON strings are properly escaped and the structure is valid. ` +
            `CRITICAL: Always preserve the exact case of property keys as they appear in AWS CloudFormation documentation.`,
        });
      }

      // Add the user request
      inputMessages.push({
        role: "user",
        content: request,
      });

      // Use GPT to create the final integrated output
      const openai = new OpenAI();
      const response = await openai.responses.create({
        model: "gpt-4o",
        text: {
          format: {
            name: "integratedResponse",
            type: "json_schema",
            schema: IntegrationResponse,
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

      // Parse the OpenAI response
      const result = JSON.parse(outputText);

      // Remove type property from si objects in create operations
      if (result.ops?.create) {
        for (const componentName in result.ops.create) {
          if (
            result.ops.create[componentName]?.properties?.si?.type !== undefined
          ) {
            // Delete the type field from si properties
            delete result.ops.create[componentName].properties.si.type;
          }
        }
      }

      // Track if we need to retry
      let needsRetry = false;
      jsonParseError = null;
      invalidDomainJson = null;
      caseMismatchErrors = {};

      // Process and validate the components
      if (result.ops?.create) {
        for (const componentName in result.ops.create) {
          const component = result.ops.create[componentName];
          if (component.properties?.domain) {
            try {
              // If domain is already a string, try to parse it into an object
              if (typeof component.properties.domain === "string") {
                component.properties.domain = JSON.parse(
                  component.properties.domain,
                );
              }

              // Check for property case mismatches for CloudFormation resources only
              const kind = component.kind;

              // No need for extra check here as checkPropertyCaseMismatches already filters non-AWS types
              const caseMismatches = await checkPropertyCaseMismatches(
                kind,
                component.properties.domain,
              );
              if (caseMismatches && caseMismatches.hasErrors) {
                console.log(
                  `Case mismatches in domain properties for ${componentName}:`,
                  caseMismatches.details,
                );

                // Store the case mismatch errors for feedback in the next retry
                caseMismatchErrors[componentName] = caseMismatches.details;

                // Flag that we need to retry
                needsRetry = true;
              }
            } catch (error) {
              console.log(
                `Failed to parse domain JSON for component ${componentName}:`,
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
        if (Object.keys(caseMismatchErrors).length > 0) {
          console.log(
            `Retrying prototypeInfrastructure due to property case mismatches (attempt ${retryCount}/${maxRetries})`,
          );
        } else {
          console.log(
            `Retrying prototypeInfrastructure due to JSON parse error (attempt ${retryCount}/${maxRetries})`,
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
          `Error in prototypeInfrastructure, retrying (${retryCount}/${maxRetries}):`,
          error,
        );
        continue;
      } else {
        // If we've used all retries, return an error response
        console.log(`All retries failed in prototypeInfrastructure:`, error);
        return {
          status: "error",
          ops: {
            create: {},
          },
        };
      }
    }
  }

  // This should never be reached but TypeScript needs it
  return {
    status: "error",
    ops: {
      create: {},
    },
  };
}
