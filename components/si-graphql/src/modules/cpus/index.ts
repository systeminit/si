import { UserInputError } from "apollo-server";

import typeDefs from "./schema.graphql";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import { checkAuthentication } from "@/modules/auth";
import { createComponentModule } from "@/modules/base";
import { Cpu, CpuEntity, CpuComponent } from "@/datalayer/component/cpu";

interface CreateCpuPayload {
  cpu: CpuEntity;
}

async function createEntity(
  _obj: GqlRoot,
  { input: { constraints, _args, workspace } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<CreateCpuPayload> {
  const user = await checkAuthentication(info);
  let component: CpuComponent;
  if (constraints) {
    const componentList = await Cpu.find(
      { where: { workspace: workspace, search: constraints } },
      user,
    );
    if (componentList.length > 1) {
      throw new UserInputError(
        `Constraints resolve to ${componentList.length} components; must resolve to 1`,
      );
    }
    component = componentList[0];
    const data = {
      name: component.name,
      description: component.description,
      cores: component.cores,
      baseFreqMHz: component.baseFreqMHz,
      allCoreTurboFreqMHz: component.allCoreTurboFreqMHz,
      singleCoreTurboFreqMHz: component.singleCoreTurboFreqMHz,
      architecture: component.architecture,
      manufacturer: component.manufacturer,
      workspaceId: `workspace:${workspace}`,
      userId: user.fqId,
    };
    if (component) {
      data["componentId"] = component.fqId();
    }
    const cpuEntity = CpuEntity.New(data);
    await cpuEntity.save();
    return { cpu: cpuEntity };
  }
}

export const Cpus = createComponentModule({
  typeDefs,
  createEntity,
  componentName: "Cpu",
  component: Cpu,
  entity: CpuEntity,
});
