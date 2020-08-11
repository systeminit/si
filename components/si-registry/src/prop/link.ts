import Joi from "joi";

import { Prop, PropValue } from "../prop";
import { PropLookup, registry } from "../registry";
import { Props } from "../attrList";

import { ObjectTypes } from "../systemComponent";

export class PropLink extends Prop {
  baseDefaultValue: string;
  lookup: undefined | PropLookup;

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

  lookupObject(): ObjectTypes {
    if (this.lookup == undefined) {
      throw new Error("Link must have a lookup object defined on `p.lookup`");
    }
    return registry.get(this.lookup.typeName);
  }

  lookupMyself(): Props {
    if (this.lookup == undefined) {
      throw new Error("Link must have a lookup object defined on `p.lookup`");
    }
    return registry.lookupProp(this.lookup);
  }

  kind(): string {
    return "link";
  }

  defaultValue(): PropValue {
    return this.lookupMyself().baseDefaultValue;
  }

  bagNames(): string[] {
    return this.lookupMyself().bagNames();
  }

  validation(): Joi.StringSchema;
  validation(): Joi.NumberSchema;
  validation(): Joi.DateSchema;
  validation(): Joi.ObjectSchema;
  validation(): Joi.ArraySchema;
  validation(): Joi.AnySchema;
  validation(): Prop["baseValidation"] {
    return this.lookupMyself()
      .validation()
      .label(this.name);
  }
}
