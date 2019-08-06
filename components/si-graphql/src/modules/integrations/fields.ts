import { Integration, IntegrationInstance } from "@/datalayer/integration";
import { User } from "@/datalayer/user";
import { Workspace } from "@/datalayer/workspace";

// IntegrationInstance fields
export async function integration(integrationInstance): Promise<Integration> {
  let integration = await integrationInstance.$relatedQuery("integration");
  return integration;
}

export async function user(integrationInstance): Promise<User> {
  let user = await integrationInstance.$relatedQuery("user");
  return user;
}

export async function workspaces(integrationInstance): Promise<Workspace[]> {
  let workspaces = await integrationInstance.$relatedQuery("workspaces");
  return workspaces;
}

// User fields
export async function integrationInstances(
  user,
): Promise<IntegrationInstance[]> {
  let integrationInstances = await user.$relatedQuery("integrationInstances");
  return integrationInstances;
}

// Workspace fields
export async function workspaceIntegrationInstances(
  workspace,
): Promise<IntegrationInstance[]> {
  let integrationInstances = await workspace.$relatedQuery(
    "integrationInstances",
  );
  return integrationInstances;
}
