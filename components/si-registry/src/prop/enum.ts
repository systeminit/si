import { Prop, PropValue } from "../prop";
import { pascalCase, constantCase } from "change-case";

export class PropEnum extends Prop {
  baseDefaultValue: string;
  variants: string[];

  constructor({
    name,
    label,
    componentTypeName,
    parentName,
    rules,
    required,
    defaultValue,
  }: {
    name: Prop["name"];
    label: Prop["label"];
    componentTypeName: Prop["componentTypeName"];
    parentName?: Prop["parentName"];
    rules?: Prop["rules"];
    required?: Prop["required"];
    defaultValue?: string;
  }) {
    super({ name, label, componentTypeName, rules, required });
    this.variants = [];
    this.parentName = parentName;
    this.baseDefaultValue = defaultValue || "";
  }

  kind(): string {
    return "enum";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }
}
