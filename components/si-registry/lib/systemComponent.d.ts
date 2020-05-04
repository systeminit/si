import { PropObject, PropMethod } from "./attrList";
import { AssociationList } from "./systemObject/associations";
import { SiGraphql } from "./systemObject/graphql";
export declare type ObjectTypes = BaseObject | SystemObject | ComponentObject | EntityObject | EntityEventObject;
export interface BaseObjectConstructor {
    typeName: BaseObject["typeName"];
    displayTypeName: BaseObject["displayTypeName"];
    serviceName: string;
    siPathName?: string;
    options?(c: BaseObject): void;
}
export interface AddMethodConstructor {
    isPrivate?: PropMethod["isPrivate"];
}
export declare class BaseObject {
    typeName: string;
    displayTypeName: string;
    siPathName: string;
    serviceName: string;
    rootProp: PropObject;
    methodsProp: PropObject;
    associations: AssociationList;
    private internalGraphql;
    constructor({ typeName, displayTypeName, serviceName, siPathName, }: BaseObjectConstructor);
    get fields(): BaseObject["rootProp"]["properties"];
    get methods(): BaseObject["methodsProp"]["properties"];
    get graphql(): SiGraphql;
    kind(): string;
}
export declare class SystemObject extends BaseObject {
    naturalKey: string;
    migrateable: boolean;
    constructor(args: BaseObjectConstructor);
    setSystemObjectDefaults(): void;
    kind(): string;
    addGetMethod(args?: AddMethodConstructor): void;
    addListMethod(args?: AddMethodConstructor): void;
}
export declare class ComponentObject extends SystemObject {
    baseTypeName: string;
    constructor(args: BaseObjectConstructor);
    setComponentDefaults(): void;
    get constraints(): ComponentObject["rootProp"]["properties"];
    kind(): string;
}
export declare class EntityObject extends SystemObject {
    baseTypeName: string;
    constructor(args: BaseObjectConstructor);
    setEntityDefaults(): void;
    get properties(): EntityObject["rootProp"]["properties"];
    kind(): string;
}
export declare class EntityEventObject extends SystemObject {
    baseTypeName: string;
    constructor(args: BaseObjectConstructor);
    setEntityEventDefaults(): void;
    kind(): string;
}
export interface ComponentAndEntityObjectConstructor {
    typeName: BaseObject["typeName"];
    displayTypeName: BaseObject["displayTypeName"];
    siPathName?: string;
    serviceName: string;
    options?(c: ComponentAndEntityObject): void;
}
export declare class ComponentAndEntityObject {
    component: ComponentObject;
    entity: EntityObject;
    entityEvent: EntityEventObject;
    constructor(args: ComponentAndEntityObjectConstructor);
    get properties(): EntityObject["rootProp"]["properties"];
    get constraints(): ComponentObject["rootProp"]["properties"];
}
