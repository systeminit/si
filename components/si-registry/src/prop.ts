import { RelationshipList } from "./prop/relationships";
import Joi from "joi";

export interface PropConstructor {
  name: string;
  label: string;
  componentTypeName: string;
}

export type PropValue =
  | null
  | string
  | string[]
  | Record<string, any>
  | boolean;

export type PropDefaultValues = {
  [key: string]: PropValue;
};

export abstract class Prop {
  name: string;
  label: string;
  rules: ((v: any) => boolean | string)[];
  required: boolean;
  readOnly: boolean;
  relationships: RelationshipList;

  baseValidation: Joi.AnySchema;

  // Hidden from the UI
  hidden: boolean;
  repeated: boolean;
  universal: boolean;
  lookupTag: null | string;
  parentName: string;
  reference: boolean;
  componentTypeName: string;
  // Hidden from the API
  skip: boolean;

  constructor({
    name,
    label,
    componentTypeName,
    rules,
    required,
    readOnly,
    hidden,
    repeated,
  }: {
    name: Prop["name"];
    label: Prop["label"];
    componentTypeName: Prop["componentTypeName"];
    rules?: Prop["rules"];
    required?: Prop["required"];
    readOnly?: Prop["readOnly"];
    hidden?: Prop["hidden"];
    repeated?: Prop["repeated"];
  }) {
    this.name = name;
    this.label = label;
    this.componentTypeName = componentTypeName;
    this.rules = rules || [];
    this.required = required || false;
    this.readOnly = readOnly || false;
    this.hidden = hidden || false;
    this.repeated = repeated || false;
    this.universal = false;
    this.lookupTag = null;
    this.parentName = "";
    this.reference = false;
    this.skip = false;
    this.relationships = new RelationshipList();
    this.baseValidation = Joi.any().label(this.name);
  }

  abstract kind(): string;
  abstract defaultValue(): PropValue;

  validation(): Joi.StringSchema;
  validation(): Joi.NumberSchema;
  validation(): Joi.DateSchema;
  validation(): Joi.ObjectSchema;
  validation(): Joi.ArraySchema;
  validation(): Joi.AnySchema;
  validation(): Prop["baseValidation"] {
    return this.baseValidation;
  }

  bagNames(): string[] {
    return [];
  }
}
