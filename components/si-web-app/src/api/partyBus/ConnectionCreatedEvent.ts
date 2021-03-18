import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { Edge } from "@/api/sdf/model/edge";

const NAME = "ConnectionCreated";

interface IConstructor {
  edge: Edge;
}

// Temp event.

export class ConnectionCreatedEvent extends PartyBusEvent {
  edge: Edge | null;

  constructor(payload: IConstructor) {
    super(NAME);
    this.edge = payload.edge;
  }

  static eventName(): string {
    return NAME;
  }
}
