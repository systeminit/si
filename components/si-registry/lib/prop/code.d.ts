import { Prop, PropValue } from "../prop";
export declare class PropCode extends Prop {
    baseDefaultValue: string;
    language: string;
    parsed: boolean;
    constructor({ name, label, componentTypeName, parsed, rules, required, defaultValue, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        language?: PropCode["language"];
        parsed?: PropCode["parsed"];
        rules?: Prop["rules"];
        required?: Prop["required"];
        defaultValue?: string;
    });
    kind(): string;
    defaultValue(): PropValue;
    realValue(value: PropValue): PropValue;
}
