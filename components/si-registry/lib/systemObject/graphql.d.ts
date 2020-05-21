import { PropObject, Props } from "../attrList";
import { ObjectTypes } from "../systemComponent";
import { DocumentNode } from "graphql";
interface QueryArgs {
    methodName: string;
    overrideName?: string;
    overrideFields?: string;
    associations?: {
        [key: string]: string[];
    };
}
interface VariablesObjectArgs {
    methodName: string;
}
interface ValidateResultArgs {
    methodName: string;
    data: Record<string, any>;
    overrideName?: string;
}
export declare class SiGraphql {
    systemObject: ObjectTypes;
    constructor(systemObject: SiGraphql["systemObject"]);
    validateResult(args: ValidateResultArgs): Record<string, any>;
    variablesObjectForProperty(prop: Props, repeated?: boolean): any;
    variablesObject(args: VariablesObjectArgs): Record<string, any>;
    graphqlTypeName(prop: Props, inputType?: boolean): string;
    associationFieldList(associations: QueryArgs["associations"], systemObject: ObjectTypes): string;
    fieldList(propObject: PropObject, associations: QueryArgs["associations"], systemObjectMemo: ObjectTypes): string;
    query(args: QueryArgs): DocumentNode;
    mutation(args: QueryArgs): DocumentNode;
}
export {};
