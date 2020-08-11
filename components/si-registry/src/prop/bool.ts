import { Prop, PropValue } from "../prop";
import Joi from "joi";

export class PropBool extends Prop {
  baseDefaultValue: boolean;

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
    defaultValue?: boolean;
  }) {
    super({ name, label, componentTypeName, rules, required });
    this.baseDefaultValue = defaultValue || false;
    this.baseValidation = Joi.bool().label(this.name);
  }

  kind(): string {
    return "bool";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }
}
