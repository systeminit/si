import { Integration, IntegrationInstance } from "@/datalayer/integration";
import { User } from "@/datalayer/user";
import { Workspace } from "@/datalayer/workspace";

// IntegrationInstance fields
export async function integration(integrationInstance): Promise<Integration> {
  const integration = await integrationInstance.$relatedQuery("integration");
  return integration;
}

export async function user(integrationInstance): Promise<User> {
  const user = await integrationInstance.$relatedQuery("user");
  return user;
}

export async function workspaces(integrationInstance): Promise<Workspace[]> {
  const workspaces = await integrationInstance.$relatedQuery("workspaces");
  return workspaces;
}

// User fields
export async function integrationInstances(
  user,
): Promise<IntegrationInstance[]> {
  const integrationInstances = await user.$relatedQuery("integrationInstances");
  return integrationInstances;
}

// Workspace fields
export async function workspaceIntegrationInstances(
  workspace,
): Promise<IntegrationInstance[]> {
  const integrationInstances = await workspace.$relatedQuery(
    "integrationInstances",
  );
  return integrationInstances;
}
