import { CreateComponent, ComponentObject } from "@/datalayer/component";

export interface CpuComponent extends ComponentObject {
  cores: number;
  baseFreqMHz: number;
  allCoreTurboFreqMHz: number;
  singleCoreTurboFreqMHz: number;
  architecture: string;
  manufacturer: string;
}

export const Cpu = CreateComponent<CpuComponent>({
  __typename: "CpuComponent",
  nodeType: "CPU",
  fqKey: "component:cpu",
});
