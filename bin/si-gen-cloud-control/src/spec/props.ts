import {
  CfProperty,
  normalizeAnyOfAndOneOfTypes,
  normalizePropertyType,
} from "../cfDb.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import { PropSpecData } from "../bindings/PropSpecData.ts";

export type OnlyProperties = {
  "createOnly": string[];
  "readOnly": string[];
  "writeOnly": string[];
};

export function isExpandedPropSpec(prop: PropSpec): prop is ExpandedPropSpec {
  const metadata = (prop as ExpandedPropSpec).metadata;
  return metadata &&
    typeof metadata.readOnly === "boolean" &&
    typeof metadata.writeOnly === "boolean" &&
    typeof metadata.createOnly === "boolean";
}

export type ExpandedPropSpec =
  & ({
    "metadata": {
      "createOnly": boolean;
      "readOnly": boolean;
      "writeOnly": boolean;
    };
  })
  & PropSpec;

type CreatePropQueue = {
  addTo: null | ((data: ExpandedPropSpec) => undefined);
  name: string;
  cfProp: CfProperty;
}[];

export function createProp(
  name: string,
  cfProp: CfProperty,
  onlyProperties: OnlyProperties,
) {
  const queue: CreatePropQueue = [
    {
      name,
      cfProp,
      addTo: null,
    },
  ];

  let rootProp = undefined;

  while (queue.length > 0) {
    const data = queue.shift();
    if (!data) break;

    const prop = createPropInner(data.name, data.cfProp, onlyProperties, queue);

    if (!data.addTo) {
      rootProp = prop;
    } else {
      data.addTo(prop);
    }
  }

  if (!rootProp) {
    throw new Error(`createProp for ${name} did not generate a prop`);
  }

  return rootProp;
}

function createPropInner(
  name: string,
  cfProp: CfProperty,
  onlyProperties: OnlyProperties,
  queue: CreatePropQueue,
): ExpandedPropSpec {
  const propUniqueId = ulid();
  const data: PropSpecData = {
    name,
    validationFormat: null,
    defaultValue: null,
    funcUniqueId: null,
    inputs: null,
    widgetKind: null,
    widgetOptions: null,
    hidden: false,
    docLink: null,
    documentation: null,
  };

  const partialProp: unknown = {
    name,
    data,
    uniqueId: propUniqueId,
    metadata: {
      "createOnly": onlyProperties.createOnly.includes(name),
      "readOnly": onlyProperties.readOnly.includes(name),
      "writeOnly": onlyProperties.writeOnly.includes(name),
    },
  };

  let normalizedCfData = normalizePropertyType(cfProp);
  normalizedCfData = normalizeAnyOfAndOneOfTypes(normalizedCfData);

  if (
    normalizedCfData.type === "integer" || normalizedCfData.type === "number"
  ) {
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "number" }>;
    prop.kind = "number";
    prop.data!.widgetKind = "Text";
    return prop;
  } else if (normalizedCfData.type === "boolean") {
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "boolean" }>;
    prop.kind = "boolean";
    prop.data!.widgetKind = "Checkbox";

    return prop;
  } else if (normalizedCfData.type === "string") {
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "string" }>;
    prop.kind = "string";
    prop.data!.widgetKind = "Text";

    return prop;
  } else if (normalizedCfData.type === "array") {
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "array" }>;
    prop.kind = "array";
    prop.data!.widgetKind = "Array";

    queue.push({
      addTo: (data: ExpandedPropSpec) => {
        prop.typeProp = data;
      },
      name: `${name}Item`,
      cfProp: normalizedCfData.items,
    });

    return prop;
  } else if (normalizedCfData.type === "object") {
    if (normalizedCfData.patternProperties) {
      const prop = partialProp as Extract<ExpandedPropSpec, { kind: "map" }>;
      prop.kind = "map";
      prop.data!.widgetKind = "Map";

      const patternProps = Object.entries(normalizedCfData.patternProperties);

      const [_, patternProp] = patternProps[0];

      if (patternProps.length !== 1 || !patternProp) {
        console.log(patternProps);
        throw new Error("too many pattern props you fool");
      }

      queue.push({
        addTo: (data: ExpandedPropSpec) => {
          prop.typeProp = data;
        },
        name: `${name}Item`,
        cfProp: patternProp,
      });

      return prop;
    } else if (normalizedCfData.properties) {
      const prop = partialProp as Extract<ExpandedPropSpec, { kind: "object" }>;
      prop.kind = "object";
      prop.data!.widgetKind = "Header";
      prop.entries = [];

      Object.entries(normalizedCfData.properties).forEach(
        ([objName, objProp]) => {
          queue.push({
            addTo: (data: ExpandedPropSpec) => {
              prop.entries.push(data);
            },
            name: objName,
            cfProp: objProp,
          });
        },
      );
      return prop;
    }
  }

  console.log(cfProp);
  console.log(normalizedCfData);

  throw new Error("no matching kind");
}

type DefaultPropType = "domain" | "secrets" | "resource";

export function createDefaultProp(
  type: DefaultPropType,
): Extract<PropSpec, { kind: "object" }> {
  const data: PropSpecData = {
    name: type,
    validationFormat: null,
    defaultValue: null,
    funcUniqueId: null,
    inputs: null,
    widgetKind: "Header",
    widgetOptions: null,
    hidden: null,
    docLink: null,
    documentation: null,
  };

  return {
    kind: "object",
    data,
    name: type,
    entries: [],
    uniqueId: ulid(),
  };
}
