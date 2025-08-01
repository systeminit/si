import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { ZodTypeAny } from "zod";
import { zodToJsonSchema } from "zod-to-json-schema";

export function generateDescription(
  desc: string,
  schemaName: string,
  schema: ZodTypeAny,
): string {
  const jsonSchema = JSON.stringify(zodToJsonSchema(schema, schemaName));

  return `${desc}\n\nThe response will be structured JSON between a <response></response> tag, and may include hints for other useful tool calls for the data between a <hints></hints> tag. The response will conform to the following JSON schema:\n\n${jsonSchema}`;
}

export function successResponse(
  // deno-lint-ignore no-explicit-any
  rawResponse: any,
  hints?: string,
): CallToolResult {
  const structuredResponse = {
    status: "success",
    data: rawResponse,
  };
  let textResponse = `<response>${
    JSON.stringify(structuredResponse)
  }</response>`;
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
export function errorResponse(error: any): CallToolResult {
  if (error.response) {
    const errorResponse = {
      status: "failure",
      errorMessage: `Error Status: ${error.response.status}\nError Data: ${
        JSON.stringify(error.response.data)
      }\nMessage:${error.message}`,
    };
    return {
      content: [
        {
          type: "text",
          text: `<response>${JSON.stringify(errorResponse)}</response>`,
        },
      ],
      structuredContent: errorResponse,
      isError: true,
    };
  } else if (error.request) {
    const errorResponse = {
      status: "failure",
      errorMessage: `No response recieved, but request failed`,
    };
    return {
      content: [
        {
          type: "text",
          text: JSON.stringify(errorResponse),
        },
      ],
      structuredContent: errorResponse,
      isError: true,
    };
  } else {
    const errorResponse = {
      status: "failure",
      errorMessage: `Error setting up request: ${error.message}`,
    };
    return {
      content: [
        {
          type: "text",
          text: JSON.stringify(errorResponse),
        },
      ],
      structuredContent: errorResponse,
      isError: true,
    };
  }
}
