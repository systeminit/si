import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import { WhoamiApi } from "@systeminit/api-client";
import { Context } from "../../../context.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const name = "validate-credentials";
const description = `<description>Validates System Initiative API credentials and workspace access. Returns information about the user, workspace, and token on success. On failure, returns error details.</description><usage>Use this tool to confirm a working credential, or after an API failure due to authentication, to confirm the credentials are working.</usage>`;

const ValidateCredentialOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z
    .object({
      userEmail: z.string().email().describe("The Users Email"),
      workspaceId: z
        .string()
        .describe("The Workspace ID that is valid for this token"),
      token: z
        .object({
          expiry: z.number().describe("The expiration time for the token"),
          workspaceId: z
            .string()
            .describe("The Workspace ID that is valid for this token"),
          role: z
            .string()
            .describe(
              "The role for this token; will be 'automation' if its using an API token",
            ),
        })
        .describe("Token attributes"),
    })
    .optional()
    .describe("The response data, populated on success"),
};
const ValidateCredentialOutputSchema = z.object(
  ValidateCredentialOutputSchemaRaw,
);

// Filter out some of the API response here just to take away any notion of leaked creds
interface WhoamiResponse {
  userEmail: string;
  workspaceId: string;
  token: {
    exp: number;
    workspaceId: string;
    role: string;
  };
}

export function validateCredentialsTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title: "Validate credentials with System Initiative",
      description: generateDescription(
        description,
        "validateCredentialsResponse",
        ValidateCredentialOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      outputSchema: ValidateCredentialOutputSchemaRaw,
    },
    async (): Promise<CallToolResult> => {
      return await withAnalytics(name, async () => {
        const apiConfig = Context.apiConfig();
        const siApi = new WhoamiApi(apiConfig);
        try {
          const response = await siApi.whoami();
          const data = response.data as WhoamiResponse;
          return successResponse({
            userEmail: data.userEmail,
            workspaceId: data.workspaceId,
            token: {
              expiry: data.token.exp,
              workspaceId: data.token.workspaceId,
              role: data.token.role,
            },
          });
        } catch (error) {
          return errorResponse(error);
        }
      });
    },
  );
}
