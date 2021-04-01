import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { ISchematicNode, Schematic } from "@/api/sdf/model/schematic";

const NAME = "NodeCreated";

interface IConstructor {
  node: ISchematicNode;
  schematic?: Schematic;
}

export class NodeCreatedEvent extends PartyBusEvent {
  node: ISchematicNode | null;
  schematic?: Schematic | null;

  constructor(payload: IConstructor, sourcePanelId?: string) {
    super(NAME, sourcePanelId);
    this.node = payload.node;
    this.schematic = payload.schematic;
  }

  static eventName(): string {
    return NAME;
  }
}
