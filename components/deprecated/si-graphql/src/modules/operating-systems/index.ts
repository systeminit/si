import { UserInputError } from "apollo-server";

import {
  OperatingSystem,
  OperatingSystemEntity,
} from "@/datalayer/component/operating-system";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import typeDefs from "./schema.graphql";
import { createComponentModule } from "@/modules/base";
import { checkAuthentication } from "@/modules/auth";

interface CreateOperatingSystemPayload {
  operatingSystem: OperatingSystemEntity;
}

async function createEntity(
  _obj: GqlRoot,
  { input: { constraints, args, workspace } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<CreateOperatingSystemPayload> {
  const user = await checkAuthentication(info);

  const searchValue =
    constraints ||
    JSON.stringify({
      platform: "ubuntu",
      platformVersion: "18.04",
    });
  const componentList = await OperatingSystem.find(
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

  const data: OperatingSystemEntity = {
    name,
    description,
    operatingSystemName: component.operatingSystemName,
    operatingSystemRelease: component.operatingSystemRelease,
    operatingSystemVersion: component.operatingSystemVersion,
    platform: component.platform,
    platformVersion: component.platformVersion,
    platformRelease: component.platformRelease,
    architecture: component.architecture[0],
    componentId: component.fqId(),
    userId: user.fqId,
    workspaceId: `workspace:${workspace}`,
  };

  const operatingSystemEntity = OperatingSystemEntity.New(data);
  await operatingSystemEntity.save();
  return { operatingSystem: operatingSystemEntity };
}

export const OperatingSystems = createComponentModule({
  typeDefs,
  createEntity,
  componentName: "OperatingSystem",
  component: OperatingSystem,
  entity: OperatingSystemEntity,
});
