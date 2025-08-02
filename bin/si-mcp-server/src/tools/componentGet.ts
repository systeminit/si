import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";
import { ComponentsApi } from "@systeminit/api-client";
import { apiConfig, WORKSPACE_ID } from "../si_client.ts";
import {
  errorResponse,
  generateDescription,
  successResponse,
} from "./commonBehavior.ts";
import _ from "lodash";
import { AttributesSchema } from "../data/components.ts";

const name = "component-get";
const title = "Get the details of a component";
const description =
  `<description>Get the details about a component in a change set. Returns the componentName, componentId, schemaName, list of actions that can be taken, list of management functions that can be run, resourceId if it is set, raw resource data if it is set (useful for troubleshooting), and the attributes of the component. Optionally include qualifications and code generation (if requested). On failure, returns error details</description><usage>Use this tool when you want to understand how a component is configured. It contains all of its attributes (and where they come from; either directly set our sourced from a subscription), actions, management functions, and resource values.</usage>`;

const GetComponentInputSchemaRaw = {
  changeSetId: z.string().describe(
    "The change set to look up the component in",
  ),
  componentId: z.string().describe("the compoonent id to look up"),
  qualifications: z.boolean().optional().describe(
    "include information about the qualifications for this component; useful when you are debugging why it might not be working as intended. if any qualifications are failing, that means the component is likely to fail when applied. can be verbose, so only include if you need it.",
  ),
  code: z.boolean().optional().describe(
    "include information about code generation functions for this component; useful when troubleshooting that a representation needed by a cloud provider (such as AWS cloudformation) is correct. Can be verbose, so only include it if you need it.",
  ),
};

const GetComponentOutputSchemaRaw = {
  status: z.enum(["success", "failure"]),
  errorMessage: z.string().optional().describe(
    "If the status is failure, the error message will contain information about what went wrong",
  ),
  data: z.object({
    componentId: z.string().describe("the component id"),
    componentName: z.string().describe("the components name"),
    resourceId: z.string().describe(
      "the resource id this component maps to in the real world; frequently the primary identifier for a cloud service, like AWS",
    ),
    attributes: AttributesSchema,
    resource: z.object({
      status: z.enum(["ok", "error", "warning"]).describe(
        "if 'ok', the resource exists and is ok. if 'warning', the resource exists but may have a problem. if 'error', then the resource may exist, but there is an error.",
      ),
      resourceData: z.any().optional().describe(
        "the raw resource data returned from the cloud provider; likely a JSON object, but not guaranteed",
      ),
      lastSynced: z.string().describe(
        "the last time the resource was refreshed",
      ),
    }).optional().describe(
      "the raw resource data, as returned from the cloud provider. the real state of the resource.",
    ),
    code: z.record(
      z.string().describe("the name of the code generator function"),
      z.object({
        code: z.string().describe("the generated source code"),
        format: z.string().describe(
          "the language/format of the generated code",
        ),
      }).describe("the generated code"),
    ).optional().describe(
      "optional code generation output; useful when troubleshooting",
    ),
    qualifications: z.record(
      z.string().describe("the name of the qualification"),
      z.object({
        result: z.enum(["failure", "success", "unknown", "warning"]).describe(
          "'failure' means the qualification has failed, and there is a problem; 'success' means there is no problem; 'unknown' means we don't know the state of the qualification; and 'warning' means there may be a problem",
        ),
        message: z.string().describe("the message about this qualification"),
      }).describe("the qualification result and message"),
    ).optional().describe("optional qualification results"),
    actions: z.array(
      z.object({ actionName: z.string().describe("the action function name") }),
    ).describe("the list of actions this component supports"),
    management: z.array(
      z.object({
        managementName: z.string().describe("the management function name"),
      }),
    ).describe("the list of management functions this component supports"),
  }).describe("the component data"),
};
const GetComponentOutputSchema = z.object(
  GetComponentOutputSchemaRaw,
);

type GetComponentResult = z.infer<typeof GetComponentOutputSchema>["data"];

export function componentGetTool(server: McpServer) {
  server.registerTool(
    name,
    {
      title,
      description: generateDescription(
        description,
        "componentGetResponse",
        GetComponentOutputSchema,
      ),
      annotations: {
        readOnlyHint: true,
      },
      inputSchema: GetComponentInputSchemaRaw,
      outputSchema: GetComponentOutputSchemaRaw,
    },
    async (
      { changeSetId, componentId, code, qualifications },
    ): Promise<CallToolResult> => {
      const siApi = new ComponentsApi(apiConfig);
      try {
        const response = await siApi.getComponent({
          workspaceId: WORKSPACE_ID,
          changeSetId: changeSetId,
          componentId,
        });
        // _.pickBy(response.data.component.attributes, (_value, key) => key.includes('/code'))
        const attributes = _.pickBy(
          response.data.component.attributes,
          (_value: unknown, key: string) =>
            key.startsWith("/domain") || key.startsWith("/si/name") ||
            key.startsWith("/resource_value") || key.startsWith("/secrets"),
        );
        const result: GetComponentResult = {
          componentId: response.data.component.id,
          componentName: response.data.component.name,
          resourceId: response.data.component.resourceId,
          attributes: attributes as GetComponentResult["attributes"],
          actions: response.data.actionFunctions.map((af) => {
            return { actionName: af.funcName };
          }),
          management: response.data.managementFunctions.map((mf) => {
            return { managementName: mf.funcName };
          }),
        };
        if (code) {
          const codeAttributes = _.pickBy(
            response.data.component.attributes,
            (_value: unknown, key: string) => key.startsWith("/code"),
          );
          const code: NonNullable<GetComponentResult["code"]> = {};
          for (const [path, codeValue] of Object.entries(codeAttributes)) {
            const match = path.match(/^\/code\/([^\/]+)\/([^\/]+)/);
            if (match) {
              if (!code[match[1]]) {
                code[match[1]] = { code: "", format: "" };
              }
              if (match[2] === "code" || match[2] === "format") {
                code[match[1]][match[2] as "code" | "format"] =
                  codeValue as string;
              }
            }
          }
          result["code"] = code;
        }
        if (qualifications) {
          const qualAttributes = _.pickBy(
            response.data.component.attributes,
            (_value: unknown, key: string) => key.startsWith("/qualification"),
          );
          const qualifications: NonNullable<
            GetComponentResult["qualifications"]
          > = {};
          for (const [path, qualValue] of Object.entries(qualAttributes)) {
            const match = path.match(/^\/qualification\/([^\/]+)\/([^\/]+)/);
            if (match) {
              if (!qualifications[match[1]]) {
                qualifications[match[1]] = { result: "unknown", message: "" };
              }
              if (match[2] === "result") {
                qualifications[match[1]]["result"] = qualValue as
                  | "failure"
                  | "success"
                  | "unknown"
                  | "warning";
              } else if (match[2] === "message") {
                qualifications[match[1]]["message"] = qualValue as string;
              }
            }
          }
          result["qualifications"] = qualifications;
        }
        const resourceAttributes = _.pickBy(
          response.data.component.attributes,
          (_value: unknown, key: string) => key.startsWith("/resource/"),
        );
        const resource: {
          status?: "ok" | "error" | "warning";
          resourceData?: unknown;
          lastSynced?: string;
        } = {};
        for (
          const [path, resourceValue] of Object.entries(resourceAttributes)
        ) {
          if (path == "/resource/status") {
            resource["status"] = resourceValue as "ok" | "error" | "warning";
          } else if (path == "/resource/payload") {
            resource["resourceData"] = resourceValue;
          } else if (path == "/resource/last_synced") {
            resource["lastSynced"] = resourceValue as string;
          }
        }
        if (!_.isEmpty(resource) && resource.status && resource.lastSynced) {
          result["resource"] = resource as {
            status: "ok" | "error" | "warning";
            lastSynced: string;
            resourceData?: unknown;
          };
        }

        return successResponse(
          result,
        );
      } catch (error) {
        return errorResponse(error);
      }
    },
  );
}
