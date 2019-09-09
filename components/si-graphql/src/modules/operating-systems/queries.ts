import { checkAuthentication } from "@/modules/auth";
import {
  OperatingSystem,
  OperatingSystemComponent,
} from "@/datalayer/component/operating-system";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import {
  GetComponentsInput,
  filterComponents,
} from "@/modules/components/queries";

export async function getOperatingSystemComponents(
  _obj: GqlRoot,
  args: GetComponentsInput,
  _context: GqlContext,
  info: GqlInfo,
): Promise<OperatingSystemComponent[]> {
  const user = await checkAuthentication(info);
  const data = await OperatingSystem.getAll();
  return filterComponents(data, args, user);
}
