import { Prop, PropDefaultValues, PropConstructor } from "./prop";
import { PropText } from "./prop/text";
import { PropCode } from "./prop/code";
import { PropSelect } from "./prop/select";
import { PropNumber } from "./prop/number";
import { PropMap } from "./prop/map";
import { PropEnum } from "./prop/enum";
import { PropBool } from "./prop/bool";
import { PropLink } from "./prop/link";
import { PropPassword } from "./prop/password";

import { pascalCase } from "change-case";

export type Props =
  | PropText
  | PropPassword
  | PropSelect
  | PropCode
  | PropNumber
  | PropObject
  | PropMap
  | PropEnum
  | PropBool
  | PropLink;

interface AddArguments {
  name: string;
  label: string;
  componentTypeName?: string;
  parentName?: string;
  options?(p: Props): void;
}

interface AttrListConstructor {
  componentTypeName?: string;
  parentName?: string;
  readOnly?: boolean;
  autoCreateEdits?: boolean;
}

export interface IntegrationService {
  integrationName: string;
  integrationServiceName: string;
}

export class AttrList {
  attrs: Props[];
  readOnly: boolean;
  parentName: string;
  autoCreateEdits: boolean;
  componentTypeName: string;

  constructor({
    parentName,
    readOnly,
    componentTypeName,
    autoCreateEdits,
  }: AttrListConstructor) {
    this.parentName = parentName || "";
    this.attrs = [];
    this.componentTypeName = componentTypeName || "";
    this.readOnly = readOnly || false;
    this.autoCreateEdits = autoCreateEdits || false;
  }

  get length(): number {
    return this.attrs.length;
  }

  hasEntries(): boolean {
    return this.attrs.length > 0;
  }

  entries(): this["attrs"] {
    return this.attrs;
  }

  getEntry(name: string): Props {
    const result = this.attrs.find(e => e.name == name);
    if (result == undefined) {
      throw new Error(
        `Cannot find property ${name} for ${this.componentTypeName}`,
      );
    }
    return result;
  }

  createValueObject(defaultValues?: PropDefaultValues): PropDefaultValues {
    const resultValues = defaultValues || {};
    for (const item of this.entries()) {
      if (resultValues[item.name]) {
        continue;
      } else {
        resultValues[item.name] = item.defaultValue();
      }
    }
    return resultValues;
  }

  realValues(values: PropDefaultValues): PropDefaultValues {
    const resultValues: PropDefaultValues = {};
    for (const item of this.entries()) {
      if (item.kind() == "code" && item instanceof PropCode) {
        if (values[item.name]) {
          resultValues[item.name] = item.realValue(values[item.name]);
        }
      } else {
        resultValues[item.name] = values[item.name];
      }
    }
    return resultValues;
  }

  addExisting(p: Props): void {
    p.reference = true;
    this.attrs.push(p);
  }

  addProp(p: Props, addArgs: AddArguments): void {
    if (addArgs.options) {
      addArgs.options(p);
    }
    if (this.readOnly) {
      p.readOnly = this.readOnly;
    }
    if (this.autoCreateEdits) {
      this.autoCreateEditAction(p);
    }
    this.attrs.push(p);
  }

  addBool(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    const p = new PropBool(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addText(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    const p = new PropText(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addSelect(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    const p = new PropSelect(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addPassword(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    const p = new PropPassword(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addEnum(addArgs: AddArguments): void {
    addArgs.parentName = pascalCase(this.parentName);
    addArgs.componentTypeName = this.componentTypeName;
    const p = new PropEnum(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addNumber(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    const p = new PropNumber(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addLink(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    const p = new PropLink(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addObject(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    addArgs.parentName = pascalCase(this.parentName);
    const p = new PropObject(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addAction(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    addArgs.parentName = pascalCase(this.parentName);
    const p = new PropAction(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addMethod(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    addArgs.parentName = pascalCase(this.parentName);
    const p = new PropMethod(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addMap(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    const p = new PropMap(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addCode(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.componentTypeName;
    const p = new PropCode(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  autoCreateEditAction(_p: Props): void {
    //We went another way, and no longer need to auto create edits.
    //
    //I'm leaving this code here, just in case we decide to change our minds.
    //
    //const notAllowedKinds = ["method", "action"];
    //if (notAllowedKinds.includes(p.kind())) {
    //  return;
    //}
    //const systemObject = registry.get(p.componentTypeName);
    //systemObject.methods.addAction({
    //  name: `${camelCase(p.name)}Edit`,
    //  label: `Edit ${camelCase(p.parentName)}${pascalCase(p.name)} Property`,
    //  options(pa: PropAction) {
    //    pa.universal = true;
    //    pa.mutation = true;
    //    pa.request.properties.addLink({
    //      name: "property",
    //      label: `The ${p.label} property value`,
    //      options(pl: PropLink) {
    //        pl.lookup = {
    //          typeName: p.componentTypeName,
    //          names: ["properties", p.name],
    //        };
    //      },
    //    });
    //  },
    //});
  }
}

export class PropObject extends Prop {
  baseDefaultValue: Record<string, any>;
  properties: AttrList;

  constructor({
    name,
    label,
    componentTypeName,
    parentName,
    defaultValue,
  }: {
    name: Prop["name"];
    label: Prop["label"];
    componentTypeName: Prop["componentTypeName"];
    parentName?: Prop["parentName"];
    defaultValue?: PropObject["baseDefaultValue"];
  }) {
    super({ name, label, componentTypeName });
    this.baseDefaultValue = defaultValue || {};
    this.parentName = parentName || "";
    this.properties = new AttrList({
      parentName: `${pascalCase(this.parentName)}${pascalCase(name)}`,
      componentTypeName: this.componentTypeName,
    });
  }

  kind(): string {
    return "object";
  }

  protobufType(suffix = ""): string {
    return `${pascalCase(this.parentName)}${pascalCase(this.name)}${pascalCase(
      suffix,
    )}`;
  }

  defaultValue(): PropObject["baseDefaultValue"] {
    return this.baseDefaultValue;
  }

  bagNames(): string[] {
    return ["properties"];
  }
}

export class PropMethod extends Prop {
  baseDefaultValue: Record<string, any>;
  request: PropObject;
  reply: PropObject;
  mutation: boolean;
  skipAuth: boolean;
  isPrivate: boolean;

  // Methods have a Request and a Response
  //
  // The Request is made up of properties!
  // The Reply is made up of properties!

  constructor({
    name,
    label,
    componentTypeName,
    parentName,
    defaultValue,
  }: {
    name: Prop["name"];
    label: Prop["label"];
    componentTypeName: Prop["componentTypeName"];
    parentName?: Prop["parentName"];
    defaultValue?: PropAction["baseDefaultValue"];
  }) {
    super({ name, label, componentTypeName });
    this.baseDefaultValue = defaultValue || {};
    this.parentName = parentName || "";
    this.request = new PropObject({
      name: `${pascalCase(name)}Request`,
      label: `${label} Request`,
      parentName: this.parentName,
      componentTypeName: this.componentTypeName,
    });
    this.reply = new PropObject({
      name: `${pascalCase(name)}Reply`,
      label: `${label} Reply`,
      parentName: this.parentName,
      componentTypeName: this.componentTypeName,
    });
    this.mutation = false;
    this.skipAuth = false;
    this.isPrivate = false;
  }

  kind(): string {
    return "method";
  }

  protobufType(suffix = ""): string {
    return `${pascalCase(this.parentName)}${pascalCase(this.name)}${pascalCase(
      suffix,
    )}`;
  }

  defaultValue(): PropObject["baseDefaultValue"] {
    return this.baseDefaultValue;
  }

  bagNames(): string[] {
    return ["request", "reply"];
  }
}

export class PropAction extends PropMethod {
  integrationServices: IntegrationService[];

  // Actions have a Request and a Response
  //
  // The Response is always `{ entityEvent: EntityEvent }`;
  //
  // The Request is made up of properties!

  constructor({
    name,
    label,
    componentTypeName,
    parentName,
    defaultValue,
  }: {
    name: Prop["name"];
    label: Prop["label"];
    componentTypeName: Prop["componentTypeName"];
    parentName?: Prop["parentName"];
    defaultValue?: PropAction["baseDefaultValue"];
  }) {
    super({ name, label, componentTypeName, parentName, defaultValue });
    this.integrationServices = [];
    this.request.properties.addText({
      name: "id",
      label: "Entity ID",
      options(p) {
        p.universal = true;
        p.required = true;
      },
    });
    this.request.properties.addText({
      name: "changeSetId",
      label: "Change Set ID",
      options(p) {
        p.universal = true;
        p.required = true;
      },
    });
    this.reply.properties.addLink({
      name: "item",
      label: `Entity Event`,
      options(p: PropLink) {
        p.universal = true;
        p.readOnly = true;
        p.lookup = {
          typeName: `${this.componentTypeName}Event`,
        };
      },
    });
  }

  kind(): string {
    return "action";
  }

  protobufType(suffix = ""): string {
    return `${pascalCase(this.parentName)}${pascalCase(this.name)}${pascalCase(
      suffix,
    )}`;
  }
}
