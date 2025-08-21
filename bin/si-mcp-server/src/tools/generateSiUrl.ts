import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ChangeSetsApi } from "@systeminit/api-client";
import { ComponentsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
} from "./commonBehavior.ts";
import { ChangeSet } from "../data/changeSets.ts";

const description = `<description>Generates a URL for a component details page or a review screen for a change set.</description><usage>Use this tool to generate a url to a component details page or a change set review screen in the System Initiative web application. You should never try and create a component to match the users request. You should never offer to link the user to another component and you should never try and find a matching component in a different change set once a change set has been specified.</usage>`;

const GenerateSiUrlInputSchemaRaw = {
  changeSetId: z
    .string()
    .optional()
    .describe(
      "The change set to generate a URL for; if not provided, HEAD will be used",
    ),
  componentId: z.string().optional().describe("the component id to link to"),
  showReview: z
    .boolean()
    .optional()
    .default(false)
    .describe("whether to generate a link to the review screen"),
};

const GenerateSiUrlOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z
    .string()
    .optional()
    .describe(
      "If the status is failure, the error message will contain information about what went wrong",
    ),
  data: z.object({
    url: z
      .string()
      .describe("the generated URL to the System Initiative web application"),
  }),
};
const GenerateSiUrlOutputSchema = z.object(GenerateSiUrlOutputSchemaRaw);

type GenerateSiUrlResult = z.infer<typeof GenerateSiUrlOutputSchema>["data"];

export function generateSiUrlTool(server: McpServer) {
  server.registerTool(
    "generate-si-url",
    {
      title: "Generate a URL to link to the System Initiative web application",
      description: generateDescription(
        description,
        "generateUrlResponse",
        GenerateSiUrlOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      inputSchema: GenerateSiUrlInputSchemaRaw,
      outputSchema: GenerateSiUrlOutputSchemaRaw,
    },
    async ({
      changeSetId,
      componentId,
      showReview,
    }): Promise<CallToolResult> => {
      if (!changeSetId) {
        const changeSetsApi = new ChangeSetsApi(apiConfig);
        try {
          const changeSetList = await changeSetsApi.listChangeSets({
            workspaceId: WORKSPACE_ID,
          });
          const changeSets = changeSetList.data.changeSets as ChangeSet[];
          const head = changeSets.find((cs) => cs.isHead);
          if (!head) {
            return errorResponse({
              message: "Could not find HEAD change set",
            });
          }
          changeSetId = head.id;
        } catch (error) {
          return errorResponse({
            message: `No change set id was provided, and we could not find HEAD; this is a bug! Tell the user we are sorry: ${
              error instanceof Error ? error.message : String(error)
            }`,
          });
        }
      }

      if (!componentId && !showReview) {
        return errorResponse({
          message:
            "Invalid request, either `showReview` or `componentId` must be specified",
        });
      }

      if (componentId && showReview) {
        return errorResponse({
          message:
            "Invalid request, one of `showReview` or `componentId` must be specified, not both",
        });
      }

      const result: GenerateSiUrlResult = {
        url: "",
      };
      if (componentId) {
        // lets first check there's a component of that Id in that changeSet otherwise it's a broken link
        const siApi = new ComponentsApi(apiConfig);
        try {
          await siApi.getComponent({
            workspaceId: WORKSPACE_ID,
            changeSetId: changeSetId,
            componentId,
          });
          result.url = generateComponentLink(changeSetId, componentId);
        } catch {
          return errorResponse({
            message: `No component found in that change set. Tell the user to ensure they are using the correct change set.`,
          });
        }
      }

      if (showReview) {
        result.url = generateReviewLink(changeSetId);
      }

      return successResponse(result);
    },
  );
}

function generateComponentLink(
  changeSetId: string,
  componentId: string,
): string {
  const config = createLinkConfig();
  return `${config.baseUrl}/n/${config.workspaceId}/${changeSetId}/h/${componentId}/c`;
}

function generateReviewLink(changeSetId: string): string {
  const config = createLinkConfig();
  return `${config.baseUrl}/n/${config.workspaceId}/${changeSetId}/h/r`;
}

interface LinkConfig {
  baseUrl: string;
  workspaceId: string;
}

function createLinkConfig(): LinkConfig {
  const baseUrl = Deno.env.get("SI_BASE_URL") || "https://api.systeminit.com";
  const webUrl = baseUrl.replace("api", "app");

  return {
    baseUrl: webUrl,
    workspaceId: WORKSPACE_ID,
  };
}
