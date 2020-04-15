import { Prop, PropValue } from "@/prop";
import { PropLookup, registry } from "@/componentRegistry";
import { Props } from "@/attrList";

import { snakeCase } from "change-case";

export class PropLink extends Prop {
  baseDefaultValue: string;
  lookup: PropLookup;

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

  lookupMyself(): Props {
    return registry.lookupProp(this.lookup);
  }

  protobufType(): string {
    return this.lookupMyself().protobufType();
  }

  kind(): string {
    return "link";
  }

  defaultValue(): PropValue {
    return this.lookupMyself().baseDefaultValue;
  }

  protobufDefinition(inputNumber: number): string {
    const realp = this.lookupMyself();
    const realPackageName = realp.protobufPackageName();
    let packageName: string;
    if (realPackageName.match(/^google/)) {
      packageName = "";
    } else {
      packageName = this.protobufPackageName();
    }
    return realp.protobufDefinition(inputNumber, packageName, this.name);
  }

  protobufImportPath(componentName = ""): string {
    if (componentName == this.lookup.component) {
      return "";
    } else {
      return `si-registry/proto/${this.protobufPackageName()}proto`;
    }
  }

  protobufPackageName(): string {
    return `si.${snakeCase(this.lookup.component)}.`;
  }

  bagNames(): string[] {
    return this.lookupMyself().bagNames();
  }
}
