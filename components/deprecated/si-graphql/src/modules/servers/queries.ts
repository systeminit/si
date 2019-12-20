import { checkAuthentication } from "@/modules/auth";
import { Server, ServerComponent } from "@/datalayer/component/server";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import {
  GetComponentsInput,
  filterComponents,
} from "@/modules/components/queries";

export async function getServerComponents(
  _obj: GqlRoot,
  args: GetComponentsInput,
  _context: GqlContext,
  info: GqlInfo,
): Promise<ServerComponent[]> {
  const user = await checkAuthentication(info);
  const data: ServerComponent[] = await Server.getAll();
  return filterComponents(data, args, user);
}
