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

import OpenAI from "jsr:@openai/openai@^4";
import { getServiceByName, loadCfDatabase } from "jsr:@systeminit/cf-db@0";
import { extractFieldsPrompt } from "./prompts/extract-fields-prompt.ts";

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

/**
 * Maximum content length allowed by OpenAI API
 *
 * Set below the actual limit to maintain a safety buffer for other content.
 * Used when processing large CloudFormation schemas.
 */
const MAX_CONTENT_LENGTH = 250000;

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
 * @returns A structured response with property paths, documentation, and values
 */
export async function extractFields(
  typeName: string,
  request: string,
  existingProperties?: Record<string, unknown>,
): Promise<ExtractFieldsResponse> {
  const instructions = extractFieldsPrompt;

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
      serviceContent = serviceContent.substring(0, MAX_CONTENT_LENGTH - 1000) +
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

  // If we have existing properties, provide them as context
  if (existingProperties) {
    inputMessages.push({
      role: "developer",
      content: "Existing component properties: \n\n" +
        JSON.stringify(existingProperties, null, 2),
    });
  }

  // Add the user request as the final message
  inputMessages.push({
    role: "user",
    content: request,
  });

  try {
    const openai = new OpenAI();
    const response = await openai.responses.create({
      model: "gpt-4o-mini",
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

    // Parse and return the result as ExtractFieldsResponse
    return JSON.parse(outputText) as ExtractFieldsResponse;
  } catch (error) {
    console.log(`Error in extractFields for ${typeName}:`, error);
    // Return an empty result structure that matches the expected schema
    return { properties: [] } as ExtractFieldsResponse;
  }
}
