import { ObjectTypes, BaseObjectConstructor, SystemObject, BaseObject, ComponentObject, EntityObject, ComponentAndEntityObject, ComponentAndEntityObjectConstructor } from "./systemComponent";
import { Props } from "./attrList";
export interface PropLookup {
    typeName: string;
    names?: string[];
}
export declare class Registry {
    objects: ObjectTypes[];
    constructor();
    get(typeName: string): ObjectTypes;
    serviceNames(): string[];
    getObjectsForServiceName(serviceName: string): ObjectTypes[];
    lookupProp(lookup: PropLookup): Props;
    base(constructorArgs: BaseObjectConstructor): BaseObject;
    system(constructorArgs: BaseObjectConstructor): SystemObject;
    component(constructorArgs: BaseObjectConstructor): ComponentObject;
    entity(constructorArgs: BaseObjectConstructor): EntityObject;
    componentAndEntity(constructorArgs: ComponentAndEntityObjectConstructor): ComponentAndEntityObject;
}
export declare const registry: Registry;
