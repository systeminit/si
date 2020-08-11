import { Prop } from "../prop";
import { PropText } from "../prop/text";
import Joi from "joi";

export class PropNumber extends PropText {
  baseDefaultValue: string;
  numberKind: "int32" | "uint32" | "int64" | "uint64" | "u128";

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
    this.baseValidation = Joi.number().label(this.name);
  }

  kind(): string {
    return "number";
  }
}
