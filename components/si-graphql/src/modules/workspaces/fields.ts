import { Workspace } from "@/datalayer/workspace";
import { User } from "@/datalayer/user";

// Workspace Fields

export async function creator(workspace: Workspace): Promise<User> {
  let creator = await workspace.$relatedQuery("creator");
  return creator;
}

export async function members(workspace: Workspace): Promise<User[]> {
  let members = await workspace.$relatedQuery("members");
  return members;
}

// User Fields

export async function createdWorkspaces(user: Workspace): Promise<Workspace[]> {
  let workspaces: Workspace[] = await user.$relatedQuery("createdWorkspaces");
  return workspaces;
}

export async function workspaces(user: User): Promise<Workspace[]> {
  let workspaces: Workspace[] = await user.$relatedQuery("workspaces");
  return workspaces;
}
