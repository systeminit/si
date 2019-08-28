import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import { Workspace } from "@/datalayer/workspace";
import { checkAuthentication } from "@/modules/auth";

interface CreateWorkspaceResponse {
  workspace: Workspace;
}

export async function createWorkspace(
  _obj: GqlRoot,
  { input: { name: name, description: description } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<CreateWorkspaceResponse> {
  const creator = await checkAuthentication(info);

  const workspace = await Workspace.create({
    name,
    description,
    creator,
  });

  return {
    workspace,
  };
}

interface DeleteWorkspaceResponse {
  workspace: Workspace;
}

export async function deleteWorkspace(
  _obj: GqlRoot,
  { input: { id: workspaceId } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<DeleteWorkspaceResponse> {
  const user = await checkAuthentication(info);
  const workspace = await Workspace.delete({ user, workspaceId });
  return {
    workspace,
  };
}
