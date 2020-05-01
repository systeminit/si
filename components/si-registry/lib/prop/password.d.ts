import { Prop, PropValue } from "../prop";
import { PropText } from "./text";
export declare class PropPassword extends PropText {
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
