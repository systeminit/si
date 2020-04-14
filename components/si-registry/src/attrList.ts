import { Prop, PropDefaultValues, PropConstructor } from "@/prop";
import { PropText } from "@/prop/text";
import { PropCode } from "@/prop/code";
import { PropSelect } from "@/prop/select";
import { PropNumber } from "@/prop/number";
import { PropMap } from "@/prop/map";
import { PropComponent } from "@/prop/component";
import { PropEntity } from "@/prop/entity";
import { PropEnum } from "@/prop/enum";
import { PropBool } from "@/prop/bool";
import { PropEntityEvent } from "@/prop/entityEvent";
import { PropLink } from "@/prop/link";
import { PropConstraints } from "@/prop/constraints";
import { PropProperties } from "@/prop/properties";
import { propRegistry } from "@/propRegistry";
import { Component } from "@/component";
import { registry } from "@/componentRegistry";

import { pascalCase, camelCase } from "change-case";

export type Props =
  | PropText
  | PropSelect
  | PropCode
  | PropNumber
  | PropObject
  | PropMap
  | PropEnum
  | PropComponent
  | PropEntity
  | PropEntityEvent
  | PropBool
  | PropConstraints
  | PropProperties
  | PropLink;

interface AddArguments {
  name: string;
  label: string;
  componentTypeName?: string;
  parentName?: string;
  options?(p: Props): void;
}

interface AttrListConstructor {
  component?: Component;
  parentName?: string;
  readOnly?: boolean;
  autoCreateEdits?: boolean;
}

export class AttrList {
  attrs: Props[];
  readOnly: boolean;
  parentName: string;
  autoCreateEdits: boolean;
  component: Component;

  constructor({
    parentName,
    readOnly,
    component,
    autoCreateEdits,
  }: AttrListConstructor) {
    this.parentName = parentName || "";
    this.attrs = [];
    this.component = component;
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

  getEntry(name: string): Props | undefined {
    return this.attrs.find(e => e.name == name);
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
    propRegistry.add(p);
    this.attrs.push(p);
  }

  addBool(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    const p = new PropBool(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addText(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    const p = new PropText(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addEnum(addArgs: AddArguments): void {
    addArgs.parentName = pascalCase(this.parentName);
    addArgs.componentTypeName = this.component.typeName;
    const p = new PropEnum(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addNumber(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    const p = new PropNumber(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addLink(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    const p = new PropLink(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addObject(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    addArgs.parentName = pascalCase(this.parentName);
    const p = new PropObject(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addAction(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    addArgs.parentName = pascalCase(this.parentName);
    const p = new PropAction(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addMethod(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    addArgs.parentName = pascalCase(this.parentName);
    const p = new PropMethod(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addComponent(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    addArgs.parentName = pascalCase(this.parentName);
    const p = new PropComponent(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addEntity(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    addArgs.parentName = pascalCase(this.parentName);
    const p = new PropEntity(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addEntityEvent(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    addArgs.parentName = pascalCase(this.parentName);
    const p = new PropEntityEvent(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addMap(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    const p = new PropMap(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addCode(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    const p = new PropCode(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addConstraints(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    const p = new PropConstraints(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addProperties(addArgs: AddArguments): void {
    addArgs.componentTypeName = this.component.typeName;
    const p = new PropProperties(addArgs as PropConstructor);
    this.addProp(p, addArgs);
  }

  addFromRegistry(lookupTag: string, addArgs: AddArguments): void {
    const p = propRegistry.get(lookupTag);
    if (propRegistry.get(lookupTag) === undefined) {
      throw `Cannot find ${lookupTag} in Prop Registry`;
    }
    if (addArgs.options) {
      addArgs.options(p);
    }
    this.attrs.push(p);
  }

  autoCreateEditAction(p: Props): void {
    const notAllowedKinds = ["method", "action"];
    if (notAllowedKinds.includes(p.kind())) {
      return;
    }
    this.component.entityActions.addAction({
      name: `Edit${camelCase(p.parentName)}${pascalCase(p.name)}Action`,
      label: `Edit ${camelCase(p.parentName)}${pascalCase(p.name)} Property`,
      options(pa: PropAction) {
        pa.universal = true;
        pa.request.addLink({
          name: p.name,
          label: p.label,
          options(p: PropLink) {
            p.lookup = {
              component: p.componentTypeName,
              propType: "properties",
              names: [p.name],
            };
          },
        });
        //pa.request.addProp(p, { name: p.name, label: p.label });
      },
    });
  }
}

export class PropObject extends Prop {
  baseDefaultValue: Record<string, any>;
  properties: AttrList;
  realParentName: string;

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
    this.parentName = parentName;
    this.properties = new AttrList({
      parentName: `${pascalCase(parentName)}${pascalCase(name)}`,
      component: registry.get(this.componentTypeName),
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
  request: AttrList;
  reply: AttrList;
  realParentName: string;

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
    this.parentName = parentName;
    this.request = new AttrList({
      parentName: `${pascalCase(parentName)}${pascalCase(name)}Request`,
      component: registry.get(this.componentTypeName),
    });
    this.reply = new AttrList({
      parentName: `${pascalCase(parentName)}${pascalCase(name)}Reply`,
      component: registry.get(this.componentTypeName),
    });
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
  // Actions have a Request and a Response
  //
  // The Response is always `{ entity: Entity, entityEvent: EntityEvent }`;
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
    this.reply.addEntity({
      name: "entity",
      label: "Entity",
      options(p) {
        p.required = true;
      },
    });
    this.reply.addEntityEvent({
      name: "entityEvent",
      label: "Entity Event",
      options(p) {
        p.required = true;
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
