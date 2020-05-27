import { ObjectTypes } from "../systemComponent";
import { Props, PropObject } from "../attrList";
import { PropEnum } from "../prop/enum";
export declare class ProtobufFormatter {
    systemObjects: ObjectTypes[];
    recurseKinds: string[];
    constructor(systemObjects: ObjectTypes[]);
    first(): ObjectTypes;
    protobufPackageName(): string;
    protobufServices(): string;
    protobufMessages(): string;
    protobufImportForProp(prop: Props): string;
    protobufTypeForProp(prop: Props): string;
    protobufDefinitionForProp(prop: Props, inputNumber: number): string;
    protobufMessageForPropObject(prop: PropObject | PropEnum): string;
    protobufImports(): string;
    protobufImportWalk(props: Props[]): Set<string>;
    generateProto(): Promise<void>;
    generateString(): string;
}
