import { Prop, PropValue } from "../prop";
import { PropLookup } from "../registry";
import { Props } from "../attrList";
import { ObjectTypes } from "../systemComponent";
export declare class PropLink extends Prop {
    baseDefaultValue: string;
    lookup: PropLookup;
    constructor({ name, label, componentTypeName, rules, required, defaultValue, }: {
        name: Prop["name"];
        label: Prop["label"];
        componentTypeName: Prop["componentTypeName"];
        rules?: Prop["rules"];
        required?: Prop["required"];
        defaultValue?: string;
    });
    lookupObject(): ObjectTypes;
    lookupMyself(): Props;
    kind(): string;
    defaultValue(): PropValue;
    bagNames(): string[];
}
