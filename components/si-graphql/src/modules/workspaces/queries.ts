import { Workspace } from "@/datalayer/workspace";
import { GqlRoot, GqlArgs, GqlContext, GqlInfo } from "@/app.module";
import { checkAuthentication } from "@/modules/auth";

export async function getWorkspaceById(
  _obj: GqlRoot,
  { input: { id: id } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<Workspace> {
  await checkAuthentication(info);
  return Workspace.getById(id);
}

export async function getWorkspaces(
  _obj: GqlRoot,
  _input: GqlArgs,
  _context: GqlContext,
  info: GqlInfo,
): Promise<Workspace[]> {
  const user = await checkAuthentication(info);
  return Workspace.getWorkspacesForUser(user);
}
