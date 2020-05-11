import { Prop, PropValue } from "../prop";

export class PropSelect extends Prop {
  baseDefaultValue: string;
  options: string[];

  constructor({
    name,
    label,
    componentTypeName,
    options,
    rules,
    required,
    defaultValue,
  }: {
    name: Prop["name"];
    label: Prop["label"];
    componentTypeName: Prop["componentTypeName"];
    options: PropSelect["options"];
    rules?: Prop["rules"];
    required?: Prop["required"];
    defaultValue?: string;
  }) {
    super({ name, label, componentTypeName, rules, required });
    this.options = options;
    this.baseDefaultValue = defaultValue || "";
  }

  kind(): string {
    return "select";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }
}
