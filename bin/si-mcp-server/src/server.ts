import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { validateCredentialsTool } from "./tools/validateCredentials.ts";
import { changeSetListTool } from "./tools/changeSetList.ts";
import { changeSetCreateTool } from "./tools/changeSetCreate.ts";
import { changeSetUpdateTool } from "./tools/changeSetUpdateStatus.ts";
import { schemaFindTool } from "./tools/schemaFind.ts";
import { schemaAttributesListTool } from "./tools/schemaAttributesList.ts";
import { schemaAttributesDocumentationTool } from "./tools/schemaAttributesDocumentation.ts";
import { actionListTool } from "./tools/actionList.ts";
import { actionUpdateTool } from "./tools/actionUpdateStatus.ts";
import { funcRunGetTool } from "./tools/funcRunGet.ts";
import { componentListTool } from "./tools/componentList.ts";
import { componentGetTool } from "./tools/componentGet.ts";
import { componentCreateTool } from "./tools/componentCreate.ts";
import { componentUpdateTool } from "./tools/componentUpdate.ts";
import { componentEnqueueActionTool } from "./tools/componentEnqueueAction.ts";

export function createServer(): McpServer {
  const server = new McpServer({
    name: "si-server",
    version: "0.1.0",
  });
  validateCredentialsTool(server);
  changeSetListTool(server);
  changeSetCreateTool(server);
  changeSetUpdateTool(server);
  schemaFindTool(server);
  schemaAttributesListTool(server);
  schemaAttributesDocumentationTool(server);
  actionListTool(server);
  actionUpdateTool(server);
  funcRunGetTool(server);
  componentListTool(server);
  componentGetTool(server);
  componentCreateTool(server);
  componentUpdateTool(server);
  componentEnqueueActionTool(server);

  return server;
}
