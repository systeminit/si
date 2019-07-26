import { Workspace } from "@/datalayer/workspace";
import { checkAuthentication } from "@/modules/auth";

interface CreateWorkspaceResponse {
  workspace: Workspace;
}

export async function createWorkspace(
  _obj,
  { input: { name: wsName, description: description } },
  _context,
  info,
): Promise<CreateWorkspaceResponse> {
  const currentUser = await checkAuthentication(info);

  const workspace = await Workspace.createWorkspace(
    wsName,
    currentUser,
    description,
  );
  return {
    workspace,
  };
}

interface DeleteWorkspaceResponse {
  workspace: Workspace;
}

export async function deleteWorkspace(
  _obj,
  { input: { id: id } },
  _context,
  info,
): Promise<CreateWorkspaceResponse> {
  const currentUser = await checkAuthentication(info);
  const workspace = await Workspace.deleteWorkspace(currentUser, id);
  return {
    workspace,
  };
}
