import { CreateComponent, ComponentObject } from "@/datalayer/component";

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
