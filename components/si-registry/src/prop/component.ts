import { Prop, PropValue } from "@/prop";

export class PropComponent extends Prop {
  baseDefaultValue: string;

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

  protobufType(): string {
    return "Component";
  }

  kind(): string {
    return "component";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }
}
