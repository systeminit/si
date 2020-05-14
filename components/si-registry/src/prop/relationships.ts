import { PropLookup } from "../registry";
import { Props } from "../attrList";
import { ObjectTypes } from "../systemComponent";
import { registry } from "../registry";

export type Relationships = Updates | Either;

interface RelationshipConstructor {
  partner: PropLookup;
}

export abstract class Relationship {
  partner: PropLookup;

  constructor(args: RelationshipConstructor) {
    this.partner = args.partner;
  }

  partnerObject(): ObjectTypes {
    return registry.get(this.partner.typeName);
  }

  partnerProp(): Props {
    return registry.lookupProp(this.partner);
  }

  abstract kind(): string;
}

// An updates relationship ensures that when one method changes,
// another one gets notified.
export class Updates extends Relationship {
  kind(): string {
    return "updates";
  }
}

export class Either extends Relationship {
  kind(): string {
    return "either";
  }
}

export class RelationshipList {
  relationships: Relationships[] = [];

  all(): RelationshipList["relationships"] {
    return this.relationships;
  }

  updates(args: RelationshipConstructor): void {
    this.relationships.push(new Updates(args));
  }

  either(args: RelationshipConstructor): void {
    this.relationships.push(new Either(args));
  }
}
