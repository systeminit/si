import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { ChangeSet } from "@/api/sdf/model/changeSet";

const NAME = "CurrentChangeSet";

export class CurrentChangeSetEvent extends PartyBusEvent {
  changeSet: ChangeSet | null;

  constructor(changeSet: ChangeSet | null) {
    super(NAME);
    this.changeSet = changeSet;
  }

  static eventName(): string {
    return NAME;
  }
}
