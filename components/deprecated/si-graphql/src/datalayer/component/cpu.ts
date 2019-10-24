import { CreateComponent, ComponentObject } from "@/datalayer/component";
import { CreateEntity, EntityObject } from "@/datalayer/entity";

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

export interface CpuEntity extends EntityObject {
  cores: number;
  baseFreqMHz: number;
  allCoreTurboFreqMHz: number;
  singleCoreTurboFreqMHz: number;
  architecture: string;
  manufacturer: string;
}

export const CpuEntity = CreateEntity<CpuEntity>({
  __typename: "CpuEntity",
  nodeType: "CPU",
  fqKey: "entity:cpu",
});

CpuEntity.hasOneComponent({
  from: "componentId",
  to: { field: "component", model: Cpu },
});
