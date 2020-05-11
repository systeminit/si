import { Prop, PropValue } from "../prop";
export declare class PropSelect extends Prop {
    baseDefaultValue: string;
    options: string[];
    constructor({ name, label, componentTypeName, options, rules, required, defaultValue, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        options: PropSelect["options"];
        rules?: Prop["rules"];
        required?: Prop["required"];
        defaultValue?: string;
    });
    kind(): string;
    defaultValue(): PropValue;
}
