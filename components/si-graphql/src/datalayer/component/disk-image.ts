import { CreateComponent, ComponentObject } from "@/datalayer/component";

export interface DiskImageComponent extends ComponentObject {
  format: string;
  operatingSystemId?: string;
}

export const DiskImage = CreateComponent<DiskImageComponent>({
  __typename: "DiskImageComponent",
  nodeType: "Disk Image",
  fqKey: "component:diskimage",
});
