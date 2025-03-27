/**
 * Extract CloudFormation types functionality for AI Agent
 *
 * This module provides functionality to identify appropriate AWS CloudFormation resource
 * types based on natural language descriptions of infrastructure requirements. It returns
 * relevant resource types with justifications for why they match the request.
 *
 * @module
 */

import OpenAI from "jsr:@openai/openai@^4";
import { loadCfDatabase } from "jsr:@systeminit/cf-db@0";
import { extractTypesPrompt } from "./prompts/extract-types-prompt.ts";

/**
 * Schema for extractTypes response
 *
 * Defines the expected JSON structure for the AI model's response when extracting
 * CloudFormation resource types, including the type name and justification.
 */
const ExtractTypesResponse = {
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "types": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "cfType": {
            "type": "string",
          },
          "justification": {
            "type": "string",
          },
        },
        "additionalProperties": false,
        "required": ["cfType", "justification"],
      },
    },
  },
  "additionalProperties": false,
  "required": ["types"],
};

/**
 * Helper function to get all available CloudFormation service types from the database
 *
 * Extracts the list of valid AWS CloudFormation resource types that can be used
 * when identifying appropriate resources for infrastructure requirements.
 *
 * @returns Array of CloudFormation resource type names (e.g., "AWS::EC2::Instance")
 */
async function getAllCfTypes(): Promise<string[]> {
  // First make sure the database is loaded
  const db = await loadCfDatabase({});

  // Extract all CloudFormation type names from the db
  // This requires some knowledge of the internal structure of the database
  // Since we can't modify the cfDb.ts directly, we'll extract this info ourselves
  const typeNames: string[] = [];

  // Access the database's schemas which contain all the service types
  if (db && db.schemas) {
    for (const key in db.schemas) {
      if (key.startsWith("AWS::")) {
        typeNames.push(key);
      }
    }
  }

  return typeNames;
}

/**
 * Extract relevant CloudFormation types based on user request
 *
 * @param request - Natural language description of the infrastructure requirements
 * @param invalidTypes - Optional array of previously suggested invalid types to avoid
 * @returns Array of CloudFormation resource types with justifications for their selection
 */
export async function extractTypes(
  request: string,
  invalidTypes: string[] = [],
): Promise<Array<{ cfType: string; justification: string }>> {
  const instructions = extractTypesPrompt;

  // Get all available service names for context
  const allServiceNames = await getAllCfTypes();

  // Create additional context about invalid types if any exist
  let invalidTypesContext = "";
  if (invalidTypes.length > 0) {
    invalidTypesContext =
      `IMPORTANT: The following CloudFormation types were previously suggested but do NOT exist in the database. DO NOT suggest them again:\n\n${
        invalidTypes.join("\n")
      }\n\nPlease suggest alternative valid types instead.`;
  }

  // Create type-safe input messages
  const inputMessages: Array<
    { role: "user" | "developer" | "system" | "assistant"; content: string }
  > = [
    {
      role: "developer",
      content: "Available CloudFormation Resource Types:\n\n" +
        allServiceNames.join("\n"),
    },
  ];

  if (invalidTypesContext) {
    inputMessages.push({
      role: "developer",
      content: invalidTypesContext,
    });
  }

  inputMessages.push({
    role: "user",
    content: request,
  });

  const openai = new OpenAI();
  const response = await openai.responses.create({
    model: "gpt-4o",
    text: {
      format: {
        name: "extractedTypes",
        type: "json_schema",
        schema: ExtractTypesResponse,
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

  const result = JSON.parse(outputText);
  return result.types;
}
