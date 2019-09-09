import { Workspace } from "@/datalayer/workspace";
import { User } from "@/datalayer/user";

// Workspace Fields
export async function creator(workspace: Workspace): Promise<User> {
  return workspace.creator();
}

export async function members(workspace: Workspace): Promise<User[]> {
  return workspace.members();
}

// User Fields
export async function createdWorkspaces(user: User): Promise<Workspace[]> {
  return Workspace.getWorkspacesCreatedByUser(user);
}

export async function workspaces(user: User): Promise<Workspace[]> {
  return Workspace.getWorkspacesForUser(user);
}
