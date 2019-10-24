import { CreateComponent, ComponentObject } from "@/datalayer/component";
import { CreateEntity, EntityObject } from "@/datalayer/entity";

export interface DiskImageComponent extends ComponentObject {
  format: string;
  operatingSystemId?: string;
}

export const DiskImage = CreateComponent<DiskImageComponent>({
  __typename: "DiskImageComponent",
  nodeType: "Disk Image",
  fqKey: "component:diskimage",
});

export interface DiskImageEntity extends EntityObject {
  format: string;
  operatingSystemId?: string;
}

export const DiskImageEntity = CreateEntity<DiskImageEntity>({
  __typename: "DiskImageEntity",
  nodeType: "Disk Image",
  fqKey: "entity:diskimage",
});

DiskImageEntity.hasOneComponent({
  from: "componentId",
  to: { field: "component", model: DiskImage },
});
