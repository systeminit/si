import { GqlRoot, GqlArgs, GqlContext, GqlInfo } from "@/app.module";
import { PortEntity, PortComponent } from "@/datalayer/component/port";
import { findPortComponents } from "./queries";
import { checkAuthentication } from "@/modules/auth";

import { UserInputError } from "apollo-server";

interface CreatePortPayload {
  port: PortEntity;
}

export async function createPort(
  obj: GqlRoot,
  { input: { constraints, args, workspace } },
  context: GqlContext,
  info: GqlInfo,
): Promise<CreatePortPayload> {
  const user = await checkAuthentication(info);
  let portComponent: PortComponent;
  if (constraints) {
    const componentList = await findPortComponents(
      obj,
      { where: { workspace: workspace, search: constraints } },
      context,
      info,
    );
    if (componentList.length > 1) {
      throw new UserInputError(
        `Constraints resolve to ${componentList.length} components; must resolve to 1`,
      );
    }
    portComponent = componentList[0];
  }
  const data = {
    name: args.name,
    description: args.description || portComponent.description,
    serviceName: args.serviceName || portComponent.serviceName,
    protocol: args.protocol || portComponent.protocol,
    number: args.number || portComponent.number,
    workspaceId: `workspace:${workspace}`,
    userId: user.fqId,
  };
  if (portComponent) {
    data["componentId"] = portComponent.fqId();
  }
  const portEntity = PortEntity.New(data);
  console.log(portEntity);
  await portEntity.save();
  return { port: portEntity };
}
