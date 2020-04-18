import { pascalCase, snakeCase, camelCase } from "change-case";

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
  hidden: boolean;
  repeated: boolean;
  universal: boolean;
  lookupTag: null | string;
  parentName: string;
  reference: boolean;
  componentTypeName: string;

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
  }

  abstract kind(): string;
  abstract defaultValue(): PropValue;

  graphqlTypeName(inputType?: boolean): string {
    let request = "";
    if (inputType) {
      request = "Request";
    }
    return `${pascalCase(this.componentTypeName)}${pascalCase(
      this.parentName,
    )}${pascalCase(this.name)}${request}`;
  }

  graphqlFieldName(): string {
    return `${camelCase(this.name)}`;
  }

  bagNames(): string[] {
    return [];
  }
}
