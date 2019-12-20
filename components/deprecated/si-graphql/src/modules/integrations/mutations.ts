import { GqlRoot, GqlArgs, GqlContext, GqlInfo } from "@/app.module";
import { IntegrationInstance } from "@/datalayer/integration";
import { Workspace } from "@/datalayer/workspace";
import { checkAuthentication } from "@/modules/auth";

interface CreateIntegrationInstancePayload {
  integrationInstance: IntegrationInstance;
}

export async function createIntegrationInstance(
  _obj: GqlRoot,
  { input: { integrationId, name, description, options } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<CreateIntegrationInstancePayload> {
  const user = await checkAuthentication(info);
  const integrationInstance = await IntegrationInstance.create({
    integrationId,
    name,
    description,
    options,
    user,
  });
  return { integrationInstance };
}

interface DeleteIntegrationInstancePayload {
  integrationInstance: IntegrationInstance;
}

export async function deleteIntegrationInstance(
  _obj: GqlRoot,
  { input: { id } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<DeleteIntegrationInstancePayload> {
  const user = await checkAuthentication(info);
  const integrationInstance = await IntegrationInstance.delete({ id, user });
  return {
    integrationInstance,
  };
}

interface EnableIntegrationInstanceOnWorkspacePayload {
  integrationInstance: IntegrationInstance;
  workspace: Workspace;
}

export async function enableIntegrationInstanceOnWorkspace(
  _obj: GqlRoot,
  { input: { integrationInstanceId, workspaceId } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<EnableIntegrationInstanceOnWorkspacePayload> {
  await checkAuthentication(info);
  const result = await Workspace.enableIntegrationInstance(
    workspaceId,
    integrationInstanceId,
  );
  return { integrationInstance: result[0], workspace: result[1] };
}

interface DisableIntegrationInstanceOnWorkspacePayload {
  integrationInstance: IntegrationInstance;
  workspace: Workspace;
}

export async function disableIntegrationInstanceOnWorkspace(
  _obj: GqlRoot,
  { input: { integrationInstanceId, workspaceId } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<DisableIntegrationInstanceOnWorkspacePayload> {
  await checkAuthentication(info);
  const result = await Workspace.enableIntegrationInstance(
    workspaceId,
    integrationInstanceId,
  );
  return { integrationInstance: result[0], workspace: result[1] };
}
