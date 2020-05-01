import { ObjectTypes } from "../systemComponent";
import * as PropPrelude from "../components/prelude";
import { Props } from "../attrList";
interface RustTypeAsPropOptions {
    reference?: boolean;
    option?: boolean;
}
export declare class RustFormatter {
    systemObject: ObjectTypes;
    constructor(systemObject: RustFormatter["systemObject"]);
    structName(): string;
    modelName(): string;
    typeName(): string;
    errorType(): string;
    hasCreateMethod(): boolean;
    implListRequestType(renderOptions?: RustTypeAsPropOptions): string;
    implListReplyType(renderOptions?: RustTypeAsPropOptions): string;
    implServiceRequestType(propMethod: PropPrelude.PropMethod, renderOptions?: RustTypeAsPropOptions): string;
    implServiceReplyType(propMethod: PropPrelude.PropMethod, renderOptions?: RustTypeAsPropOptions): string;
    implServiceMethodName(propMethod: PropPrelude.PropMethod | PropPrelude.PropAction): string;
    rustFieldNameForProp(prop: Props): string;
    implServiceAuth(propMethod: PropPrelude.PropMethod): string;
    implServiceAuthCall(propMethod: PropPrelude.PropMethod): string;
    implServiceGetMethodBody(propMethod: PropPrelude.PropMethod): string;
    serviceMethods(): string;
    rustTypeForProp(prop: Props, renderOptions?: RustTypeAsPropOptions): string;
    implCreateNewArgs(): string;
    implCreatePassNewArgs(): string;
    implServiceMethodListResultToReply(): string;
    implServiceMethodCreateDestructure(): string;
    naturalKey(): string;
    isMigrateable(): boolean;
    isStorable(): boolean;
    implCreateSetProperties(): string;
    implCreateAddToTenancy(): string;
    storableValidateFunction(): string;
    storableOrderByFieldsByProp(topProp: PropPrelude.PropObject, prefix: string): string;
    storableOrderByFieldsFunction(): string;
    storableReferentialFieldsFunction(): string;
}
export declare class RustFormatterService {
    serviceName: string;
    systemObjects: ObjectTypes[];
    constructor(serviceName: string);
    systemObjectsAsFormatters(): RustFormatter[];
    implServiceStructBody(): string;
    implServiceStructConstructorReturn(): string;
    implServiceNewConstructorArgs(): string;
    implServiceTraitName(): string;
    hasComponents(): boolean;
}
export declare class CodegenRust {
    serviceName: string;
    constructor(serviceName: string);
    generateGenMod(): Promise<void>;
    generateGenModelMod(): Promise<void>;
    generateGenService(): Promise<void>;
    generateGenModel(systemObject: ObjectTypes): Promise<void>;
    makePath(pathPart: string): Promise<string>;
    formatCode(): Promise<void>;
    writeCode(filename: string, code: string): Promise<void>;
}
export {};
