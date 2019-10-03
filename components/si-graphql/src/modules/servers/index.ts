import { UserInputError } from "apollo-server";

import { Server, ServerEntity } from "@/datalayer/component/server";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import typeDefs from "./schema.graphql";
import { createComponentModule } from "@/modules/base";
import { checkAuthentication } from "@/modules/auth";

interface CreateServerPayload {
  server: ServerEntity;
}

async function createEntity(
  _obj: GqlRoot,
  { input: { constraints, args, workspace } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<CreateServerPayload> {
  const user = await checkAuthentication(info);

  const searchValue =
    constraints ||
    JSON.stringify({
      name: "AWS EC2 m5.large",
    });
  const componentList = await Server.find(
    { where: { workspace: workspace, search: searchValue } },
    user,
  );
  if (componentList.length > 1) {
    throw new UserInputError(
      `Constraints resolve to ${componentList.length} components; must resolve to 1`,
    );
  }
  const component = componentList[0];

  let name = component.name;
  let description = component.description;

  if (args) {
    if (args.name) {
      name = args.name;
    }
    if (args.description) {
      description = args.description;
    }
  }

  const data: ServerEntity = {
    name,
    description,
    memoryGIB: component.memoryGIB,
    cpuCores: component.cpuCores,
    cpuId: component.cpuId,
    componentId: component.fqId(),
    userId: user.fqId,
    workspaceId: `workspace:${workspace}`,
  };

  const serverEntity = ServerEntity.New(data);
  await serverEntity.save();
  return { server: serverEntity };
}

export const Servers = createComponentModule({
  typeDefs,
  createEntity,
  componentName: "Server",
  component: Server,
  entity: ServerEntity,
});
