import { CreateComponent, ComponentObject } from "@/datalayer/component";

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
