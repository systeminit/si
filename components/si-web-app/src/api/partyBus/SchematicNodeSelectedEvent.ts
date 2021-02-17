import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { ISchematicNode } from "@/api/sdf/model/schematic";

const NAME = "SchematicNodeSelected";

interface IConstructor {
  schematicNode: ISchematicNode;
}

export class SchematicNodeSelectedEvent extends PartyBusEvent {
  schematicNode: ISchematicNode;

  constructor(payload: IConstructor) {
    super(NAME);
    this.schematicNode = payload.schematicNode;
  }

  static eventName(): string {
    return NAME;
  }
}
