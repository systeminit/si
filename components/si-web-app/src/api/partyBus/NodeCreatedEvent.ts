import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { Node } from "@/api/sdf/model/node";
import { Entity } from "../sdf/model/entity";

const NAME = "NodeCreated";

interface IConstructor {
  node: Node;
  entity: Entity;
}

export class NodeCreatedEvent extends PartyBusEvent {
  node: Node | null;
  entity: Entity | null;

  constructor(payload: IConstructor, sourcePanelId?: string) {
    super(NAME, sourcePanelId);
    this.node = payload.node;
    this.entity = payload.entity;
  }

  static eventName(): string {
    return NAME;
  }
}
