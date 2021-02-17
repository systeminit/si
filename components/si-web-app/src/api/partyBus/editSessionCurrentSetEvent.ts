import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { EditSession } from "@/api/sdf/model/editSession";

const NAME = "EditSessionCurrentSet";

export class EditSessionCurrentSetEvent extends PartyBusEvent {
  editSession: EditSession | null;

  constructor(editSession: EditSession | null) {
    super(NAME);
    this.editSession = editSession;
  }

  static eventName(): string {
    return NAME;
  }
}
