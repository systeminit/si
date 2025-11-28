import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import type { ZodTypeAny } from "zod-v3";
import { zodToJsonSchema } from "zod-to-json-schema";
import { analytics } from "../analytics.ts";
import { WORKSPACE_ID } from "../si_client.ts";
import type { ChangeSetItem } from "../data/changeSets.ts";

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
  // deno-lint-ignore no-explicit-any
  const jsonSchema = JSON.stringify(zodToJsonSchema(schema as any, schemaName));

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
  let textResponse = `<response>${
    JSON.stringify(
      structuredResponse,
    )
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
export function errorResponse(error: any, hints?: string): CallToolResult {
  let textResponse, structuredContent;
  if (error.response) {
    structuredContent = {
      status: "failure",
      errorMessage: `Error Status: ${error.response.status}\nError Data: ${
        JSON.stringify(error.response.data)
      }\nMessage:${error.message}`,
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

// deno-lint-ignore no-explicit-any
export async function findHeadChangeSet(changeSetsApi: any, onlyHead: boolean) {
  try {
    const changeSetList = await changeSetsApi.listChangeSets({
      workspaceId: WORKSPACE_ID!,
    });
    const head = (changeSetList.data.changeSets as ChangeSetItem[]).find(
      (cs) => cs.isHead,
    );
    if (!head) {
      return {
        message:
          "No HEAD change set found; this is a bug! Tell the user we are sorry.",
      };
    }
    return {
      changeSetId: head.id,
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    return {
      message: onlyHead
        ? `We could not find the HEAD change set; this is a bug! Tell the user we are sorry: ${errorMessage}`
        : `No change set id was provided, and we could not find HEAD; this is a bug! Tell the user we are sorry: ${
          error instanceof Error ? error.message : String(error)
        }`,
    };
  }
}
