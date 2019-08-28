import { checkAuthentication } from "@/modules/auth";
import {
  DiskImage,
  DiskImageComponent,
} from "@/datalayer/component/disk-image";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import {
  GetComponentsInput,
  filterComponents,
} from "@/modules/components/queries";

export async function getDiskImageComponents(
  _obj: GqlRoot,
  args: GetComponentsInput,
  _context: GqlContext,
  info: GqlInfo,
): Promise<DiskImageComponent[]> {
  const user = await checkAuthentication(info);
  const data = await DiskImage.getAll();
  return filterComponents(data, args, user);
}
