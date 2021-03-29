import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { IUpdateEntityReplySuccess } from "@/api/sdf/dal/attributeDal";

export interface IUpdateEntityEvent extends IUpdateEntityReplySuccess {
  storeId: string;
}

const NAME = "UpdateEntity";

export class UpdateEntityEvent extends PartyBusEvent {
  entity: IUpdateEntityEvent["entity"];
  diff: IUpdateEntityEvent["diff"];
  label: IUpdateEntityEvent["label"];
  storeId: IUpdateEntityEvent["storeId"];

  constructor(payload: IUpdateEntityEvent) {
    super(NAME);
    this.storeId = payload.storeId;
    this.entity = payload.entity;
    this.diff = payload.diff;
    this.label = payload.label;
  }

  static eventName(): string {
    return NAME;
  }
}
