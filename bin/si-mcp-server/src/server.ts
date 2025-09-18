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
import { componentImportTool } from "./tools/componentImport.ts";
import { importPrompt } from "./prompts/import.ts";
import { componentDiscoverTool } from "./tools/componentDiscover.ts";
import { discoverPrompt } from "./prompts/discover.ts";
import { componentDeleteTool } from "./tools/componentDelete.ts";
import { componentEraseTool } from "./tools/componentErase.ts";
import { componentRestoreTool } from "./tools/componentRestore.ts";
import { generateSiUrlTool } from "./tools/generateSiUrl.ts";
import { upgradeComponentsTool } from "./tools/upgradeComponents.ts";
import { templateGenerateTool } from "./tools/templateGenerate.ts";
import { templateRunTool } from "./tools/templateRun.ts";
import { templateListTool } from "./tools/templateList.ts";
import { changeSetAbandonTool } from "./tools/changeSetAbandon.ts";
import { changeSetForceApplyTool } from "./tools/changeSetForceApply.ts";
import { costExplorerTool } from "./tools/costResourceExplorer.ts";

export function createServer(): McpServer {
  const server = new McpServer({
    name: "si-server",
    version: "0.1.0",
  });
  validateCredentialsTool(server);
  changeSetListTool(server);
  changeSetCreateTool(server);
  changeSetUpdateTool(server);
  changeSetAbandonTool(server);
  changeSetForceApplyTool(server);
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
  componentImportTool(server);
  componentDiscoverTool(server);
  componentDeleteTool(server);
  componentEraseTool(server);
  componentRestoreTool(server);
  templateGenerateTool(server);
  templateRunTool(server);
  templateListTool(server);
  generateSiUrlTool(server);
  upgradeComponentsTool(server);
  importPrompt(server);
  discoverPrompt(server);
  costExplorerTool(server);

  return server;
}
