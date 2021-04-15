import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { Schematic } from "@/api/sdf/model/schematic";

const NAME = "NodeDeleted";

interface IConstructor {
  nodeId: string;
  schematic: Schematic;
}

export class NodeDeletedEvent extends PartyBusEvent {
  nodeId: string | null;
  schematic: Schematic | null;

  constructor(payload: IConstructor) {
    super(NAME);
    this.nodeId = payload.nodeId;
    this.schematic = payload.schematic;
  }

  static eventName(): string {
    return NAME;
  }
}
