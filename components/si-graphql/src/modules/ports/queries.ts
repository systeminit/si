import { checkAuthentication } from "@/modules/auth";
import { Port, PortComponent } from "@/datalayer/component/port";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import {
  FindComponentInput,
  GetComponentsInput,
  filterComponents,
} from "@/modules/components/queries";
import { matchArray, SearchPrimitive } from "searchjs";

export async function getPortComponents(
  _obj: GqlRoot,
  args: GetComponentsInput,
  _context: GqlContext,
  info: GqlInfo,
): Promise<PortComponent[]> {
  const user = await checkAuthentication(info);
  const data: PortComponent[] = await Port.getAll();
  return filterComponents(data, args, user);
}

export async function findPortComponents(
  obj: GqlRoot,
  args: FindComponentInput,
  context: GqlContext,
  info: GqlInfo,
): Promise<PortComponent[]> {
  let data: PortComponent[];

  if (args.where.workspace) {
    data = await getPortComponents(
      obj,
      { where: { workspace: args.where.workspace } },
      context,
      info,
    );
  } else {
    data = await getPortComponents(obj, {}, context, info);
  }
  const searchArguments = JSON.parse(args.where.search) as SearchPrimitive;
  return matchArray(data, searchArguments);
}
