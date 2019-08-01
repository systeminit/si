import { IntegrationInstance } from "@/datalayer/integration";
import { Workspace } from "@/datalayer/workspace";
import { checkAuthentication } from "@/modules/auth";

interface CreateIntegrationInstancePayload {
  integrationInstance: IntegrationInstance;
}

export async function createIntegrationInstance(
  _obj,
  { input: { integrationId, name, description, options } },
  _context,
  info,
): Promise<CreateIntegrationInstancePayload> {
  const currentUser = await checkAuthentication(info);
  const integrationInstance = await IntegrationInstance.createIntegrationInstance(
    integrationId,
    name,
    description,
    options,
    currentUser,
  );
  return { integrationInstance };
}

interface DeleteIntegrationInstancePayload {
  integrationInstance: IntegrationInstance;
}

export async function deleteIntegrationInstance(
  _obj,
  { input: { id } },
  _context,
  info,
): Promise<DeleteIntegrationInstancePayload> {
  const currentUser = await checkAuthentication(info);
  const integrationInstance = await IntegrationInstance.deleteIntegrationInstance(
    id,
    currentUser,
  );
  return {
    integrationInstance,
  };
}

interface EnableIntegrationInstanceOnWorkspacePayload {
  integrationInstance: IntegrationInstance;
  workspace: Workspace;
}

export async function enableIntegrationInstanceOnWorkspace(
  _obj,
  { input: { integrationInstanceId, workspaceId } },
  _context,
  info,
): Promise<EnableIntegrationInstanceOnWorkspacePayload> {
  const currentUser = await checkAuthentication(info);
  return await IntegrationInstance.enableOnWorkspace(
    integrationInstanceId,
    workspaceId,
    currentUser,
  );
}

interface DisableIntegrationInstanceOnWorkspacePayload {
  integrationInstance: IntegrationInstance;
  workspace: Workspace;
}

export async function disableIntegrationInstanceOnWorkspace(
  _obj,
  { input: { integrationInstanceId, workspaceId } },
  _context,
  info,
): Promise<DisableIntegrationInstanceOnWorkspacePayload> {
  const currentUser = await checkAuthentication(info);
  return await IntegrationInstance.disableOnWorkspace(
    integrationInstanceId,
    workspaceId,
    currentUser,
  );
}
