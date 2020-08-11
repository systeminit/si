import Joi from "joi";
import { Prop, PropValue } from "../prop";

export interface SelectOption {
  key: string;
  value: string;
}

export class PropSelect extends Prop {
  baseDefaultValue: string;
  options: SelectOption[];
  optionsFromType: string | undefined;

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
    this.options = [];
    this.baseDefaultValue = defaultValue || "";
    this.optionsFromType = undefined;
    this.baseValidation = Joi.string().label(this.name);
  }

  kind(): string {
    return "select";
  }

  defaultValue(): PropValue {
    return this.baseDefaultValue;
  }
}
