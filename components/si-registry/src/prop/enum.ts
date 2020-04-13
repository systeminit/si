import { Prop, PropValue } from "@/prop";
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

  protobufType(suffix = ""): string {
    return `${pascalCase(this.parentName)}${pascalCase(this.name)}${pascalCase(
      suffix,
    )}`;
  }

  protobufEnumDefinition(inputNumber: number): string {
    let result = `  ${constantCase(
      this.protobufType(),
    )}_UNKNOWN = ${inputNumber};`;
    for (const variant of this.variants) {
      inputNumber++;
      result =
        result +
        `\n  ${constantCase(this.protobufType())}_${constantCase(
          variant,
        )} = ${inputNumber};`;
    }
    return result;
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }
}
