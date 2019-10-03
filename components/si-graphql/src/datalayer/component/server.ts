import { CreateComponent, ComponentObject } from "@/datalayer/component";
import { CreateEntity, EntityObject } from "@/datalayer/entity";

export interface ServerComponent extends ComponentObject {
  cpuId: string;
  cpuCores: number;
  memoryGIB: number;
}

export const Server = CreateComponent<ServerComponent>({
  __typename: "ServerComponent",
  nodeType: "Server",
  fqKey: "component:server",
});

export interface ServerEntity extends EntityObject {
  cpuId: string;
  cpuCores: number;
  memoryGIB: number;
  operatingSystemId?: string;
  sshKeyId?: string;
}

export const ServerEntity = CreateEntity<ServerEntity>({
  __typename: "ServerEntity",
  nodeType: "Server",
  fqKey: "entity:server",
});

ServerEntity.hasOneComponent({
  from: "componentId",
  to: { field: "component", model: Server },
});
