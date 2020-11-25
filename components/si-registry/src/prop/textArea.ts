import { Prop, PropValue } from "../prop";
import Joi from "joi";

export class PropTextArea extends Prop {
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
    this.baseValidation = Joi.string().label(this.name);
  }

  kind(): string {
    return "textArea";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }
}
