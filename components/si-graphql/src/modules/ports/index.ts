import { UserInputError } from "apollo-server";

import typeDefs from "./schema.graphql";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import { checkAuthentication } from "@/modules/auth";
import { createComponentModule } from "@/modules/base";
import { Port, PortEntity, PortComponent } from "@/datalayer/component/port";

interface CreatePortPayload {
  port: PortEntity;
}

async function createEntity(
  _obj: GqlRoot,
  { input: { constraints, args, workspace } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<CreatePortPayload> {
  const user = await checkAuthentication(info);
  let portComponent: PortComponent;
  if (constraints) {
    const componentList = await Port.find(
      { where: { workspace: workspace, search: constraints } },
      user,
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

export const Ports = createComponentModule({
  typeDefs,
  createEntity,
  componentName: "Port",
  component: Port,
  entity: PortEntity,
});
