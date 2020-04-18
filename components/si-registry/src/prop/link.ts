import { Prop, PropValue } from "../prop";
import { PropLookup, registry } from "../registry";
import { Props } from "../attrList";

import { snakeCase } from "change-case";
import { ObjectTypes } from "../systemComponent";

export class PropLink extends Prop {
  baseDefaultValue: string;
  lookup: PropLookup;

  constructor({
    name,
    label,
    componentTypeName,
    rules,
    required,
    defaultValue,
  }: {
    name: Prop["name"];
    label: Prop["label"];
    componentTypeName: Prop["componentTypeName"];
    rules?: Prop["rules"];
    required?: Prop["required"];
    defaultValue?: string;
  }) {
    super({ name, label, componentTypeName, rules, required });
    this.baseDefaultValue = defaultValue || "";
  }

  lookupObject(): ObjectTypes {
    return registry.get(this.lookup.typeName);
  }

  lookupMyself(): Props {
    return registry.lookupProp(this.lookup);
  }

  kind(): string {
    return "link";
  }

  defaultValue(): PropValue {
    return this.lookupMyself().baseDefaultValue;
  }

  bagNames(): string[] {
    return this.lookupMyself().bagNames();
  }
}
