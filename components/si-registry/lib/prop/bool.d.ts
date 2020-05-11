import { Prop, PropValue } from "../prop";
export declare class PropBool extends Prop {
    baseDefaultValue: boolean;
    constructor({ name, label, componentTypeName, rules, required, defaultValue, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        rules?: Prop["rules"];
        required?: Prop["required"];
        defaultValue?: boolean;
    });
    kind(): string;
    defaultValue(): PropValue;
}
