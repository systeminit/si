import { Prop, PropValue } from "../prop";

export class PropText extends Prop {
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
    return "google.protobuf.StringValue";
  }

  protobufImportPath(): string {
    return "google/protobuf/wrappers.proto";
  }

  protobufPackageName(): string {
    return "google.protobuf.";
  }

  kind(): string {
    return "text";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }
}
