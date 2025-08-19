import { WORKSPACE_ID } from "../si_client.ts";

export interface DeepLinkConfig {
  baseUrl: string;
  workspaceId: string;
}

export function createDeepLinkConfig(): DeepLinkConfig {
  const baseUrl = Deno.env.get("SI_BASE_URL") || "https://api.systeminit.com";
  const webUrl = baseUrl.replace("api.", "app");

  return {
    baseUrl: webUrl,
    workspaceId: WORKSPACE_ID,
  };
}

export function generateChangeSetDeepLink(changeSetId: string): string {
  const config = createDeepLinkConfig();
  return `${config.baseUrl}/n/${config.workspaceId}/${changeSetId}/h`;
}

export function generateChangeSetHint(
  changeSetId: string,
  changeSetName: string,
  action?: string,
): string {
  const deepLink = generateChangeSetDeepLink(changeSetId);

  let actionText = "";
  if (action === "applied") {
    actionText = " has been applied and";
  } else if (action === "created") {
    actionText = " has been created and";
  }

  return `Change set "${changeSetName}"${actionText} can be viewed at: ${deepLink}`;
}

export function generateChangeSetListHints(
  changeSets: Array<{ id: string; name: string }>,
): string {
  if (changeSets.length === 0) {
    return "No change sets available.";
  }

  const links = changeSets
    .map((cs) => `• ${cs.name}: ${generateChangeSetDeepLink(cs.id)}`)
    .join("\n");

  return `Available change sets:\n${links}`;
}

export function generateComponentDeepLink(
  changeSetId: string, 
  componentId: string
): string {
  const config = createDeepLinkConfig();
  return `${config.baseUrl}/n/${config.workspaceId}/${changeSetId}/h/${componentId}/c`;
}

export function generateFuncRunDeepLink(
  changeSetId: string,
  funcRunId: string
): string {
  const config = createDeepLinkConfig();
  return `${config.baseUrl}/n/${config.workspaceId}/${changeSetId}/h/${funcRunId}/r`;
}
