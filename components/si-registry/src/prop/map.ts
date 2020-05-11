import { Prop, PropValue } from "../prop";

export class PropMap extends Prop {
  baseDefaultValue: Record<string, string>;

  constructor({
    name,
    label,
    componentTypeName,
    defaultValue,
  }: {
    name: Prop["name"];
    label: Prop["label"];
    componentTypeName: Prop["componentTypeName"];
    defaultValue?: PropMap["baseDefaultValue"];
  }) {
    super({ name, label, componentTypeName });
    this.baseDefaultValue = defaultValue || {};
  }

  kind(): string {
    return "map";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }
}
