import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { WhoamiApi } from "@systeminit/api-client";
import { apiConfig } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
  withAnalytics,
} from "./commonBehavior.ts";

const name = "validate-credentials";
const description =
  `<description>Validates System Initiative API credentials and workspace access. Returns information about the user, workspace, and token on success. On failure, returns error details.</description><usage>Use this tool to confirm a working credential, or after an API failure due to authentication, to confirm the credentials are working.</usage>`;

const ValidateCredentialOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    userId: z.string().describe("The SI User Id"),
    userEmail: z.string().email().describe("The Users Email"),
    workspaceId: z.string().describe(
      "The Workspace ID that is valid for this token",
    ),
    token: z.object({
      iat: z.number().describe("The issue time for the token"),
      exp: z.number().describe("The expiration time for the token"),
      sub: z.string().describe(
        "The subject the token was issued for; usually the same as userId",
      ),
      jti: z.string(),
      version: z.string(),
      userId: z.string().describe("The SI User Id"),
      workspaceId: z.string().describe(
        "The Workspace ID that is valid for this token",
      ),
      role: z.string().describe(
        "The role for this token; will be 'automation' if its using an API token",
      ),
    }).describe("The decoded JWT token"),
  }).optional().describe("The response data, populated on success"),
};
const ValidateCredentialOutputSchema = z.object(
  ValidateCredentialOutputSchemaRaw,
);

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
      const siApi = new WhoamiApi(apiConfig);
      try {
        const response = await siApi.whoami();
        return successResponse(
          response.data,
        );
      } catch (error) {
        return errorResponse(error);
      }
      });
    },
  );
}
