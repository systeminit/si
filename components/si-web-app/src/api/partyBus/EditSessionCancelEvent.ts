import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";

const NAME = "EditSessionCancel";

interface IConstructor {
  editSessionId: string;
}

export class EditSessionCancelEvent extends PartyBusEvent {
  editSessionId: IConstructor["editSessionId"];

  constructor(payload: IConstructor) {
    super(NAME);
    this.editSessionId = payload.editSessionId;
  }

  static eventName(): string {
    return NAME;
  }
}
