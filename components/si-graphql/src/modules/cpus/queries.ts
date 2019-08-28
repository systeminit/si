import { checkAuthentication } from "@/modules/auth";
import { Cpu, CpuComponent } from "@/datalayer/component/cpu";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import {
  GetComponentsInput,
  filterComponents,
} from "@/modules/components/queries";

export async function getCpuComponents(
  _obj: GqlRoot,
  args: GetComponentsInput,
  _context: GqlContext,
  info: GqlInfo,
): Promise<CpuComponent[]> {
  const user = await checkAuthentication(info);
  const data: CpuComponent[] = await Cpu.getAll();
  return filterComponents(data, args, user);
}
