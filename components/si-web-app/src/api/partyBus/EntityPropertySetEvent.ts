import { PartyBusEvent } from "@/api/partyBus/partyBusEvent";
import { Entity } from "@/api/sdf/model/entity";
import { IEntitySetPropertyRequest } from "@/api/sdf/dal/editorDal";

const NAME = "EntityPropertySet";

interface IConstructor {
  entity: Entity;
  entitySetPropertyRequest: IEntitySetPropertyRequest;
}

export class EntityPropertySetEvent extends PartyBusEvent {
  entity: Entity;
  entitySetPropertyRequest: IEntitySetPropertyRequest;

  constructor(payload: IConstructor) {
    super(NAME);
    this.entity = payload.entity;
    this.entitySetPropertyRequest = payload.entitySetPropertyRequest;
  }

  static eventName(): string {
    return NAME;
  }
}
