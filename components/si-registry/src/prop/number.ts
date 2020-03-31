import { Prop } from "@/prop";
import { PropText } from "@/prop/text";

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

  protobufType(): string {
    if (this.numberKind == "int32") {
      return "google.protobuf.Int32Value";
    } else if (this.numberKind == "uint32") {
      return "google.protobuf.UInt32Value";
    } else if (this.numberKind == "int64") {
      return "google.protobuf.Int64Value";
    } else if (this.numberKind == "uint64") {
      return "google.protobuf.UInt64Value";
    }
  }

  protobufPackageName(): string {
    return "google.protobuf.";
  }

  protobufImportPath(): string {
    return "google/protobuf/wrappers.proto";
  }
}
