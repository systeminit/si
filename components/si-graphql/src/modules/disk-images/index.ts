import { UserInputError } from "apollo-server";

import { DiskImage, DiskImageEntity } from "@/datalayer/component/disk-image";
import { GqlRoot, GqlContext, GqlInfo } from "@/app.module";
import typeDefs from "./schema.graphql";
import { createComponentModule } from "@/modules/base";
import { checkAuthentication } from "@/modules/auth";

interface CreateDiskImagePayload {
  diskImage: DiskImageEntity;
}

async function createEntity(
  _obj: GqlRoot,
  { input: { constraints, args, workspace } },
  _context: GqlContext,
  info: GqlInfo,
): Promise<CreateDiskImagePayload> {
  const user = await checkAuthentication(info);

  const searchValue = constraints;
  const componentList = await DiskImage.find(
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

  const data: DiskImageEntity = {
    name,
    description,
    format: component.format,
    operatingSystemId: component.operatingSystemId,
    componentId: component.fqId(),
    userId: user.fqId,
    workspaceId: `workspace:${workspace}`,
  };

  const diskImageEntity = DiskImageEntity.New(data);
  await diskImageEntity.save();
  return { diskImage: diskImageEntity };
}

export const DiskImages = createComponentModule({
  typeDefs,
  createEntity,
  componentName: "DiskImage",
  component: DiskImage,
  entity: DiskImageEntity,
});
