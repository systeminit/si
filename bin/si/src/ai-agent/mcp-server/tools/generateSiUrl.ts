import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod-v3";
import { ChangeSetsApi } from "@systeminit/api-client";
import { ComponentsApi } from "@systeminit/api-client";
import { Context } from "../../../context.ts";
import {
  errorResponse,
  findHeadChangeSet,
  generateDescription,
  successResponse,
} from "./commonBehavior.ts";

const description = `<description>Generates a URL for a component details page, the change set review screen, the change set map view or the default link for the workspace.</description><usage>Use this tool to generate a url to a component details page, change set review screen, the change set map view or the default change set page in the System Initiative web application. You should never try and create a component to match the users request. You should never offer to link the user to another component and you should never try and find a matching component in a different change set once a change set has been specified.</usage>`;

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
  getMapView: z
    .boolean()
    .optional()
    .default(false)
    .describe("whether to link to the map view or not"),
  getWorkspaceDefaultLink: z
    .boolean()
    .optional()
    .default(false)
    .describe("whether to give the default link for the workspace"),
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
      getMapView,
      getWorkspaceDefaultLink,
    }): Promise<CallToolResult> => {
      const apiConfig = Context.apiConfig();
      const workspaceId = Context.workspaceId();
      if (!changeSetId) {
        const changeSetsApi = new ChangeSetsApi(apiConfig);
        const headChangeSet = await findHeadChangeSet(changeSetsApi, false);
        if (headChangeSet.changeSetId) {
          changeSetId = headChangeSet.changeSetId;
        } else {
          return errorResponse(headChangeSet);
        }
      }

      if (
        !componentId &&
        !showReview &&
        !getWorkspaceDefaultLink &&
        !getMapView
      ) {
        return errorResponse({
          message:
            "Invalid request, either `showReview`, `componentId`, `getMapView` or `getWorkspaceDefaultLink` must be specified",
        });
      }

      if (componentId && showReview && getWorkspaceDefaultLink && getMapView) {
        return errorResponse({
          message:
            "Invalid request, one of `showReview`, `componentId`, `getWorkspaceDefaultLink` or `getMapView` must be specified, not both",
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
            workspaceId: workspaceId,
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

      if (generateGridViewLink) {
        result.url = generateGridViewLink(changeSetId);
      }

      if (showReview) {
        result.url = generateReviewLink(changeSetId);
      }

      if (getMapView) {
        result.url = generateMapViewLink(changeSetId);
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

function generateMapViewLink(changeSetId: string): string {
  const config = createLinkConfig();
  return `${config.baseUrl}/n/${config.workspaceId}/${changeSetId}/h?map=1`;
}

function generateGridViewLink(changeSetId: string): string {
  const config = createLinkConfig();
  return `${config.baseUrl}/n/${config.workspaceId}/${changeSetId}/h`;
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
  const baseUrl = Context.apiConfig().basePath;
  if (!baseUrl) {
    throw new Error("this should be unreachable");
  }

  const webUrl = baseUrl.replace("api", "app");

  return {
    baseUrl: webUrl,
    workspaceId: Context.workspaceId(),
  };
}
