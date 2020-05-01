import { Prop, PropValue } from "../prop";
export declare class PropEnum extends Prop {
    baseDefaultValue: string;
    variants: string[];
    constructor({ name, label, componentTypeName, parentName, rules, required, defaultValue, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        parentName?: Prop["parentName"];
        rules?: Prop["rules"];
        required?: Prop["required"];
        defaultValue?: string;
    });
    kind(): string;
    defaultValue(): PropValue;
}
