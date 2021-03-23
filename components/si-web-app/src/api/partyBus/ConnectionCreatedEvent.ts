import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { Edge } from "@/api/sdf/model/edge";
import { Schematic } from "@/api/sdf/model/schematic";

const NAME = "ConnectionCreated";

interface IConstructor {
  edge: Edge;
  schematic?: Schematic;
}

export class ConnectionCreatedEvent extends PartyBusEvent {
  edge: Edge | null;
  schematic?: Schematic | null;

  constructor(payload: IConstructor) {
    super(NAME);
    this.edge = payload.edge;
    this.schematic = payload.schematic;
  }

  static eventName(): string {
    return NAME;
  }
}
