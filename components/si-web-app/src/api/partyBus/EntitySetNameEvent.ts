import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { Entity } from "@/api/sdf/model/entity";
import { IEntitySetNameRequest } from "@/api/sdf/dal/editorDal";

const NAME = "EntitySetName";

interface IConstructor {
  entity: Entity;
  entitySetNameRequest: IEntitySetNameRequest;
}

export class EntitySetNameEvent extends PartyBusEvent {
  entity: Entity;
  entitySetNameRequest: IEntitySetNameRequest;

  constructor(payload: IConstructor) {
    super(NAME);
    this.entity = payload.entity;
    this.entitySetNameRequest = payload.entitySetNameRequest;
  }

  static eventName(): string {
    return NAME;
  }
}
