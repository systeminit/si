import { DocumentNode } from "graphql";
import _ from "lodash";
import getOmitDeep from "deepdash/getOmitDeep";
import TOML from "@iarna/toml";

export type EntityPropValue = null | string | string[] | Object;
export type EntityPropDefaultValues = {
  [key: string]: EntityPropValue;
};

export abstract class EntityProp {
  name: string;
  label: string;
  rules: ((v: any) => boolean | string)[];
  required: boolean;

  constructor({
    name,
    label,
    rules,
    required,
  }: {
    name: EntityProp["name"];
    label: EntityProp["label"];
    rules?: EntityProp["rules"];
    required?: EntityProp["required"];
  }) {
    this.name = name;
    this.label = label;
    this.rules = rules || [];
    this.required = required || false;
  }

  abstract kind(): string;
  abstract defaultValue(): EntityPropValue;
}

export class EntityAttrList<T extends EntityProp> {
  attrs: T[];

  constructor(attrs: EntityAttrList<T>["attrs"]) {
    this.attrs = attrs;
  }

  entries(): this["attrs"] {
    return this.attrs;
  }

  getEntry(name: string): T | undefined {
    return this.attrs.find(e => e.name == name);
  }

  createValueObject(
    defaultValues?: EntityPropDefaultValues,
  ): EntityPropDefaultValues {
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

  realValues(values: EntityPropDefaultValues): EntityPropDefaultValues {
    const resultValues: EntityPropDefaultValues = {};
    for (const item of this.entries()) {
      if (item.kind() == "code" && item instanceof EntityPropCode) {
        if (values[item.name]) {
          resultValues[item.name] = item.realValue(values[item.name]);
        }
      } else {
        resultValues[item.name] = values[item.name];
      }
    }
    return resultValues;
  }
}

export class EntityPropText extends EntityProp {
  baseDefaultValue: string;

  constructor({
    name,
    label,
    rules,
    required,
    defaultValue,
  }: {
    name: EntityProp["name"];
    label: EntityProp["label"];
    rules?: EntityProp["rules"];
    required?: EntityProp["required"];
    defaultValue?: string;
  }) {
    super({ name, label, rules, required });
    this.baseDefaultValue = defaultValue || "";
  }

  kind(): string {
    return "text";
  }

  defaultValue(): EntityPropValue {
    return this.baseDefaultValue;
  }
}

export class EntityPropSelect extends EntityProp {
  baseDefaultValue: string;
  options: string[];

  constructor({
    name,
    label,
    options,
    rules,
    required,
    defaultValue,
  }: {
    name: EntityProp["name"];
    label: EntityProp["label"];
    options: EntityPropSelect["options"];
    rules?: EntityProp["rules"];
    required?: EntityProp["required"];
    defaultValue?: string;
  }) {
    super({ name, label, rules, required });
    this.options = options;
    this.baseDefaultValue = defaultValue || "";
  }

  kind(): string {
    return "select";
  }

  defaultValue(): EntityPropValue {
    return this.baseDefaultValue;
  }
}

interface ParsedValue {
  parsed: Object | null;
  error: string;
}

export class EntityPropCode extends EntityProp {
  baseDefaultValue: string;
  language: string;
  parsed: boolean;

  constructor({
    name,
    label,
    language,
    parsed,
    rules,
    required,
    defaultValue,
  }: {
    name: EntityProp["name"];
    label: EntityProp["label"];
    language: EntityPropCode["language"];
    parsed?: EntityPropCode["parsed"];
    rules?: EntityProp["rules"];
    required?: EntityProp["required"];
    defaultValue?: string;
  }) {
    super({ name, label, rules, required });
    this.baseDefaultValue = defaultValue || "";
    this.parsed = parsed || false;
    this.language = language;
  }

  kind(): string {
    return "code";
  }

  defaultValue(): EntityPropValue {
    return this.baseDefaultValue;
  }

  realValue(value: EntityPropValue): Object | string | null {
    if (value === null) {
      return null;
    }
    if (this.parsed) {
      if (this.language == "toml" && typeof value == "string") {
        let objectData = TOML.parse(value);
        return objectData;
      } else {
        throw "Do not know how to parse this thing";
      }
    } else {
      return value;
    }
  }
}

type EntityProps = EntityPropText | EntityPropSelect | EntityPropCode;

export class SiComponent {
  typeName: string;
  name: string;
  icon: string;

  // Core Properties
  coreProperties: EntityAttrList<EntityProps>;

  // Constraints
  constraints: EntityAttrList<EntityProps>;

  // Properties
  properties: EntityAttrList<EntityProps>;
  componentProperties: string[];

  // Hints
  hints: {
    constraintName: string;
    hintValue: string;
  }[];

  showActions: {
    displayName: string;
    mutation?: DocumentNode;
  }[];

  showProperties: {
    displayName: string;
    property: string;
    showAs: "text" | "textarea" | "url" | "toml";
  }[];

  siSpec: {
    props: string | string[];
  } | null;

  listHeaders: {
    text: string;
    value: string;
  }[];

  listEntityEventHeaders: {
    text: string;
    value: string;
  }[];

  // Queries
  getEntity: DocumentNode;
  listEntities: DocumentNode;
  pickComponent: DocumentNode;
  streamEntityEvents: DocumentNode;
  listEntityEvents: DocumentNode;

  // Mutations
  createEntity: DocumentNode;

  // Optional
  editEntity?: DocumentNode | null;

  constructor(
    typeName: string,
    {
      getEntity,
      listEntities,
      pickComponent,
      streamEntityEvents,
      createEntity,
      name,
      componentProperties,
      hints,
      showProperties,
      showActions,
      listHeaders,
      listEntityEventHeaders,
      listEntityEvents,
      icon,
      siSpec,
      editEntity,
      constraints,
      properties,
    }: {
      getEntity: DocumentNode;
      listEntities: DocumentNode;
      pickComponent: DocumentNode;
      streamEntityEvents: DocumentNode;
      createEntity: DocumentNode;
      name: string;
      componentProperties: string[];
      hints: SiComponent["hints"];
      showProperties: SiComponent["showProperties"];
      showActions: SiComponent["showActions"];
      listHeaders: SiComponent["listHeaders"];
      listEntityEventHeaders: SiComponent["listEntityEventHeaders"];
      listEntityEvents: SiComponent["listEntityEvents"];
      siSpec?: SiComponent["siSpec"];
      icon: string;
      editEntity?: SiComponent["editEntity"];
      constraints?: SiComponent["constraints"];
      properties?: SiComponent["properties"];
    },
  ) {
    this.typeName = typeName;
    this.getEntity = getEntity;
    this.listEntities = listEntities;
    this.pickComponent = pickComponent;
    this.streamEntityEvents = streamEntityEvents;
    this.createEntity = createEntity;
    this.name = name;
    this.componentProperties = componentProperties;
    this.hints = hints;
    this.showProperties = showProperties;
    this.showActions = showActions;
    this.listHeaders = listHeaders;
    this.listEntityEventHeaders = listEntityEventHeaders;
    this.icon = icon;
    this.listEntityEvents = listEntityEvents;
    this.siSpec = siSpec || null;
    this.editEntity = editEntity || null;
    this.constraints = constraints || new EntityAttrList([]);
    this.properties = properties || new EntityAttrList([]);
    this.coreProperties = new EntityAttrList([
      new EntityPropText({ name: "name", label: "Name", required: true }),
      new EntityPropText({
        name: "displayName",
        label: "Display Name",
        required: true,
      }),
      new EntityPropText({
        name: "description",
        label: "Description",
        required: true,
      }),
    ]);
  }

  listEntitiesResultString(): string {
    return `${this.typeName}ListEntities`;
  }

  getEntityResultString(): string {
    return `${this.typeName}GetEntity`;
  }

  pickComponentResultString(): string {
    return `${this.typeName}PickComponent`;
  }

  streamEntityEventsResultString(): string {
    return `streamEntityEvents`;
  }

  createEntityResultString(): string {
    return `${this.typeName}CreateEntity`;
  }

  resultString(methodName: string): string {
    return `${this.typeName}${methodName}`;
  }

  listEntityEventsResultString(): string {
    return `${this.typeName}ListEntityEvents`;
  }

  generateSpec(entity: any): string {
    interface Spec {
      [key: string]: any;
    }

    const spec: Spec = {
      name: entity["name"],
      displayName: entity["displayName"],
      description: entity["description"],
      constraints: entity["constraints"],
      props: null,
    };
    const siSpec = this.siSpec;

    if (siSpec !== null) {
      if (Array.isArray(siSpec.props)) {
        interface PropsObj {
          [key: string]: any;
        }
        const propsObj: PropsObj = {};
        for (const prop of siSpec.props) {
          propsObj[prop] = entity[prop];
        }
        spec["props"] = propsObj;
      } else {
        spec["props"] = entity[siSpec.props];
      }
    }
    let omitDeep = getOmitDeep(_);
    return TOML.stringify(omitDeep(spec, "__typename"));
  }
}
