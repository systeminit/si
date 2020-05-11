import { PropLookup } from "../registry";
import { Props } from "../attrList";
import { ObjectTypes } from "../systemComponent";
export declare type Relationships = Updates | Either;
interface RelationshipConstructor {
    partner: PropLookup;
}
export declare abstract class Relationship {
    partner: PropLookup;
    constructor(args: RelationshipConstructor);
    partnerObject(): ObjectTypes;
    partnerProp(): Props;
    abstract kind(): string;
}
export declare class Updates extends Relationship {
    kind(): string;
}
export declare class Either extends Relationship {
    kind(): string;
}
export declare class RelationshipList {
    relationships: Relationships[];
    all(): RelationshipList["relationships"];
    updates(args: RelationshipConstructor): Updates;
    either(args: RelationshipConstructor): Either;
}
export {};
