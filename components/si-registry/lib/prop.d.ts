import { RelationshipList } from "./prop/relationships";
export interface PropConstructor {
    name: string;
    label: string;
    componentTypeName: string;
}
export declare type PropValue = null | string | string[] | Record<string, any> | boolean;
export declare type PropDefaultValues = {
    [key: string]: PropValue;
};
export declare abstract class Prop {
    name: string;
    label: string;
    rules: ((v: any) => boolean | string)[];
    required: boolean;
    readOnly: boolean;
    relationships: RelationshipList;
    hidden: boolean;
    repeated: boolean;
    universal: boolean;
    lookupTag: null | string;
    parentName: string;
    reference: boolean;
    componentTypeName: string;
    skip: boolean;
    constructor({ name, label, componentTypeName, rules, required, readOnly, hidden, repeated, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        rules?: Prop["rules"];
        required?: Prop["required"];
        readOnly?: Prop["readOnly"];
        hidden?: Prop["hidden"];
        repeated?: Prop["repeated"];
    });
    abstract kind(): string;
    abstract defaultValue(): PropValue;
    bagNames(): string[];
}
