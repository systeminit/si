import { Prop } from "../prop";
import { PropText } from "../prop/text";

export class PropNumber extends PropText {
  baseDefaultValue: string;
  numberKind: "int32" | "uint32" | "int64" | "uint64";

  constructor({
    name,
    label,
    componentTypeName,
    defaultValue,
  }: {
    name: Prop["name"];
    label: Prop["label"];
    componentTypeName: Prop["componentTypeName"];
    defaultValue?: PropNumber["baseDefaultValue"];
  }) {
    super({ name, label, componentTypeName });
    this.baseDefaultValue = defaultValue || "";
    this.numberKind = "int64";
  }

  kind(): string {
    return "number";
  }
}
