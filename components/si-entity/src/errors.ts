import { findProp } from "si-registry";

export class EntityError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "EntityError";
  }
}

export class InvalidOpPathError extends EntityError {
  constructor(args: { entityType: string; path: string[] }) {
    const message = `Entity type ${args.entityType} does not have a schema entry for path ${args.path}!`;
    super(message);
    this.name = "InvalidOpPathError";
  }
}

export class InvalidContainerPropError extends EntityError {
  constructor(args: { prop: ReturnType<typeof findProp> }) {
    const message = `Prop of type ${args.prop.type} is not a container type, it must be one of: object, map, or array`;
    super(message);
    this.name = "InvalidContainerProp";
  }
}
