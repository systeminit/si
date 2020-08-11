import Joi from "joi";
import { Prop, PropValue } from "../prop";
import { PropText } from "./text";

export class PropPassword extends PropText {
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
    this.baseValidation = Joi.string().min(6);
  }

  kind(): string {
    return "password";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }
}
