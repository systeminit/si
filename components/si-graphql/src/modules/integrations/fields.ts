import { Integration, IntegrationInstance } from "@/datalayer/integration";
import { User } from "@/datalayer/user";
import { Workspace } from "@/datalayer/workspace";

// IntegrationInstance fields
export async function integration(
  integrationInstance: IntegrationInstance,
): Promise<Integration> {
  return integrationInstance.integration();
}

export async function user(
  integrationInstance: IntegrationInstance,
): Promise<User> {
  return integrationInstance.user();
}

export async function workspaces(
  integrationInstance: IntegrationInstance,
): Promise<Workspace[]> {
  return integrationInstance.workspaces();
}

// User fields
export async function integrationInstances(
  user: User,
): Promise<IntegrationInstance[]> {
  return IntegrationInstance.getForUser(user);
}

// Workspace fields
export async function workspaceIntegrationInstances(
  workspace: Workspace,
): Promise<IntegrationInstance[]> {
  return await workspace.integrationInstances();
}
