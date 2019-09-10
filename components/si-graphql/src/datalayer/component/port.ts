import { CreateComponent, ComponentObject } from "@/datalayer/component";
import { CreateEntity, EntityObject } from "@/datalayer/entity";

export interface PortComponent extends ComponentObject {
  serviceName: string;
  protocol: string;
  number: number;
}

export interface PortEntity extends EntityObject {
  serviceName: PortComponent["serviceName"];
  protocol: PortComponent["protocol"];
  number: PortComponent["number"];
}

export const Port = CreateComponent<PortComponent>({
  __typename: "PortComponent",
  nodeType: "Port",
  fqKey: "component:port",
});

export const PortEntity = CreateEntity<PortEntity>({
  __typename: "PortEntity",
  nodeType: "Port",
  fqKey: "entity:port",
});
