import { Prop, PropValue } from "../prop";
import { pascalCase } from "change-case";

export class PropConstraints extends Prop {
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
    return "Constraints";
  }

  kind(): string {
    return "constraints";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }

  graphqlTypeName(inputType?: boolean): string {
    let request = "";
    if (inputType) {
      request = "Request";
    }
    return `${pascalCase(this.componentTypeName)}${pascalCase(
      this.parentName,
    )}Constraints${request}`;
  }
}
