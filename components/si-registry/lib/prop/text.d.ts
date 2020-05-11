import { Prop, PropValue } from "../prop";
export declare class PropText extends Prop {
    baseDefaultValue: string;
    constructor({ name, label, componentTypeName, rules, required, defaultValue, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        rules?: Prop["rules"];
        required?: Prop["required"];
        defaultValue?: string;
    });
    kind(): string;
    defaultValue(): PropValue;
}
