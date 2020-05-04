import { Prop, PropDefaultValues } from "./prop";
import { PropText } from "./prop/text";
import { PropCode } from "./prop/code";
import { PropSelect } from "./prop/select";
import { PropNumber } from "./prop/number";
import { PropMap } from "./prop/map";
import { PropEnum } from "./prop/enum";
import { PropBool } from "./prop/bool";
import { PropLink } from "./prop/link";
import { PropPassword } from "./prop/password";
export declare type Props = PropText | PropPassword | PropSelect | PropCode | PropNumber | PropObject | PropMap | PropEnum | PropBool | PropLink;
interface AddArguments {
    name: string;
    label: string;
    componentTypeName?: string;
    parentName?: string;
    options?(p: Props): void;
}
interface AttrListConstructor {
    componentTypeName?: string;
    parentName?: string;
    readOnly?: boolean;
    autoCreateEdits?: boolean;
}
export declare class AttrList {
    attrs: Props[];
    readOnly: boolean;
    parentName: string;
    autoCreateEdits: boolean;
    componentTypeName: string;
    constructor({ parentName, readOnly, componentTypeName, autoCreateEdits, }: AttrListConstructor);
    get length(): number;
    hasEntries(): boolean;
    entries(): this["attrs"];
    getEntry(name: string): Props;
    createValueObject(defaultValues?: PropDefaultValues): PropDefaultValues;
    realValues(values: PropDefaultValues): PropDefaultValues;
    addExisting(p: Props): void;
    addProp(p: Props, addArgs: AddArguments): void;
    addBool(addArgs: AddArguments): void;
    addText(addArgs: AddArguments): void;
    addPassword(addArgs: AddArguments): void;
    addEnum(addArgs: AddArguments): void;
    addNumber(addArgs: AddArguments): void;
    addLink(addArgs: AddArguments): void;
    addObject(addArgs: AddArguments): void;
    addAction(addArgs: AddArguments): void;
    addMethod(addArgs: AddArguments): void;
    addMap(addArgs: AddArguments): void;
    addCode(addArgs: AddArguments): void;
    autoCreateEditAction(p: Props): void;
}
export declare class PropObject extends Prop {
    baseDefaultValue: Record<string, any>;
    properties: AttrList;
    constructor({ name, label, componentTypeName, parentName, defaultValue, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        parentName?: Prop["parentName"];
        defaultValue?: PropObject["baseDefaultValue"];
    });
    kind(): string;
    protobufType(suffix?: string): string;
    defaultValue(): PropObject["baseDefaultValue"];
    bagNames(): string[];
}
export declare class PropMethod extends Prop {
    baseDefaultValue: Record<string, any>;
    request: PropObject;
    reply: PropObject;
    mutation: boolean;
    skipAuth: boolean;
    isPrivate: boolean;
    constructor({ name, label, componentTypeName, parentName, defaultValue, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        parentName?: Prop["parentName"];
        defaultValue?: PropAction["baseDefaultValue"];
    });
    kind(): string;
    protobufType(suffix?: string): string;
    defaultValue(): PropObject["baseDefaultValue"];
    bagNames(): string[];
}
export declare class PropAction extends PropMethod {
    constructor({ name, label, componentTypeName, parentName, defaultValue, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        parentName?: Prop["parentName"];
        defaultValue?: PropAction["baseDefaultValue"];
    });
    kind(): string;
    protobufType(suffix?: string): string;
}
export {};
