import TOML from "@iarna/toml";

import { Prop, PropValue } from "@/prop";

interface ParsedValue {
  parsed: Record<string, any> | null;
  error: string;
}

export class PropCode extends Prop {
  baseDefaultValue: string;
  language: string;
  parsed: boolean;

  constructor({
    name,
    label,
    componentTypeName,
    parsed,
    rules,
    required,
    defaultValue,
  }: {
    name: Prop["name"];
    label: Prop["label"];
    componentTypeName: Prop["componentTypeName"];
    language?: PropCode["language"];
    parsed?: PropCode["parsed"];
    rules?: Prop["rules"];
    required?: Prop["required"];
    defaultValue?: string;
  }) {
    super({ name, label, componentTypeName, rules, required });
    this.baseDefaultValue = defaultValue || "";
    this.parsed = parsed || false;
    this.language = "autodetect";
  }

  kind(): string {
    return "code";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }

  protobufType(): string {
    return "google.protobuf.StringValue";
  }

  protobufPackageName(): string {
    return "google.protobuf.";
  }

  protobufImportPath(): string {
    return "google/protobuf/wrappers.proto";
  }

  realValue(value: PropValue): PropValue {
    if (value === null) {
      return null;
    }
    if (this.parsed) {
      if (this.language == "toml" && typeof value == "string") {
        const objectData = TOML.parse(value);
        return objectData;
      } else {
        throw "Do not know how to parse this thing";
      }
    } else {
      return value;
    }
  }
}

