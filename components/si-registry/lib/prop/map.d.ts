import { Prop, PropValue } from "../prop";
export declare class PropMap extends Prop {
    baseDefaultValue: Record<string, string>;
    constructor({ name, label, componentTypeName, defaultValue, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        defaultValue?: PropMap["baseDefaultValue"];
    });
    kind(): string;
    defaultValue(): PropValue;
}
