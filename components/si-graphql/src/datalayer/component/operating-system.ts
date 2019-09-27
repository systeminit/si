import { CreateComponent, ComponentObject } from "@/datalayer/component";
import { CreateEntity, EntityObject } from "@/datalayer/entity";

export interface OperatingSystemComponent extends ComponentObject {
  operatingSystemName: string;
  operatingSystemVersion: string;
  operatingSystemRelease: string;
  platform: string;
  platformVersion: string;
  platformRelease: string;
  architecture: string[];
}

export const OperatingSystem = CreateComponent<OperatingSystemComponent>({
  __typename: "OperatingSystemComponent",
  nodeType: "Operating System",
  fqKey: "component:os",
});

export interface OperatingSystemEntity extends EntityObject {
  operatingSystemName: string;
  operatingSystemVersion: string;
  operatingSystemRelease: string;
  platform: string;
  platformVersion: string;
  platformRelease: string;
  architecture: string;
}

export const OperatingSystemEntity = CreateEntity<OperatingSystemEntity>({
  __typename: "OperatingSystemEntity",
  nodeType: "Operating System",
  fqKey: "entity:os",
});

OperatingSystemEntity.hasOneComponent({
  from: "componentId",
  to: { field: "component", model: OperatingSystem },
});
