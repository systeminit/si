import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { ZodTypeAny } from "zod";
import { zodToJsonSchema } from "zod-to-json-schema";
import { analytics } from "../analytics.ts";

export async function withAnalytics<T extends { isError?: boolean }>(
  toolName: string,
  operation: () => Promise<T>,
): Promise<T> {
  const startTime = Date.now();
  const result = await operation();
  const executionTime = Date.now() - startTime;
  if (result.isError) {
    // todo: track more interesting info
    await analytics.trackError(toolName);
  } else {
    // todo: consider otel for error tracking
    await analytics.trackToolUsage(toolName, executionTime);
  }
  return result;
}

export function generateDescription(
  desc: string,
  schemaName: string,
  schema: ZodTypeAny,
): string {
  const jsonSchema = JSON.stringify(zodToJsonSchema(schema, schemaName));

  return `${desc}\n\nThe response will be structured JSON between a <response></response> tag, and may include hints for other useful tool calls for the data between a <hints></hints> tag. The response will conform to the following JSON schema:\n\n${jsonSchema}`;
}

export function successResponse(
  rawResponse: unknown,
  hints?: string,
): CallToolResult {
  const structuredResponse = {
    status: "success",
    data: rawResponse,
  };
  let textResponse = `<response>${JSON.stringify(
    structuredResponse,
  )}</response>`;
  if (hints) {
    textResponse += `\n<hints>${hints}</hints>`;
  }
  return {
    content: [
      {
        type: "text",
        text: textResponse,
      },
    ],
    structuredContent: structuredResponse,
  };
}

// deno-lint-ignore no-explicit-any
export function errorResponse(error: any, hints?: string): CallToolResult {
  let textResponse, structuredContent;
  if (error.response) {
    structuredContent = {
      status: "failure",
      errorMessage: `Error Status: ${
        error.response.status
      }\nError Data: ${JSON.stringify(error.response.data)}\nMessage:${
        error.message
      }`,
    };
    textResponse = `<response>${JSON.stringify(structuredContent)}</response>`;
  } else if (error.request) {
    structuredContent = {
      status: "failure",
      errorMessage: `No response recieved, but request failed`,
    };
    textResponse = JSON.stringify(structuredContent);
  } else {
    structuredContent = {
      status: "failure",
      errorMessage: `Error setting up request: ${error.message}`,
    };
    textResponse = JSON.stringify(structuredContent);
  }

  if (hints) {
    textResponse += `\n<hints>${hints}</hints>`;
  }

  return {
    content: [
      {
        type: "text",
        text: textResponse,
      },
    ],
    structuredContent,
    isError: true,
  };
}
