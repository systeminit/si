import { Workspace } from "@/datalayer/workspace";
import { checkAuthentication } from "@/modules/auth";

export async function getWorkspaceById(
  _obj,
  { input: { id: id } },
  _context,
  info,
): Promise<Workspace> {
  await checkAuthentication(info);
  return Workspace.query().findById(id);
}

export async function getWorkspaces(
  _obj,
  _input,
  _context,
  info,
): Promise<Workspace[]> {
  const user = await checkAuthentication(info);
  return user.$relatedQuery("workspaces").orderBy("name");
}
