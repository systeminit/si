import { checkAuthentication } from "@/modules/auth";
import { SshKey, SshKeyComponent } from "@/datalayer/component/ssh-key";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import {
  FindComponentInput,
  GetComponentsInput,
  filterComponents,
} from "@/modules/components/queries";
import { matchArray, SearchPrimitive } from "searchjs";

export async function getSshKeyComponents(
  _obj: GqlRoot,
  args: GetComponentsInput,
  _context: GqlContext,
  info: GqlInfo,
): Promise<SshKeyComponent[]> {
  const user = await checkAuthentication(info);
  const data: SshKeyComponent[] = await SshKey.getAll();
  return filterComponents(data, args, user);
}

export async function findSshKeyComponents(
  obj: GqlRoot,
  args: FindComponentInput,
  context: GqlContext,
  info: GqlInfo,
): Promise<SshKeyComponent[]> {
  let data: SshKeyComponent[];

  if (args.where.workspace) {
    data = await getSshKeyComponents(
      obj,
      { where: { workspace: args.where.workspace } },
      context,
      info,
    );
  } else {
    data = await getSshKeyComponents(obj, {}, context, info);
  }
  const searchArguments = JSON.parse(args.where.search) as SearchPrimitive;
  return matchArray(data, searchArguments);
}
