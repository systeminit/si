import {
  CfProperty,
  normalizeAnyOfAndOneOfTypes,
  normalizePropertyType,
} from "../cfDb.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import { PropSpecData } from "../bindings/PropSpecData.ts";
import { PropSpecWidgetKind } from "../bindings/PropSpecWidgetKind.ts";
import _ from "npm:lodash";
const {
  createHash,
} = await import("node:crypto");

export const CREATE_ONLY_PROP_LABEL = "si_create_only_prop";

export type OnlyProperties = {
  createOnly: string[];
  readOnly: string[];
  writeOnly: string[];
  primaryIdentifier: string[];
};

export function isExpandedPropSpec(prop: PropSpec): prop is ExpandedPropSpec {
  const metadata = (prop as ExpandedPropSpec).metadata;
  return metadata &&
    Array.isArray(metadata.propPath) &&
    typeof metadata.readOnly === "boolean" &&
    typeof metadata.writeOnly === "boolean" &&
    typeof metadata.createOnly === "boolean";
}

export type ExpandedPropSpec =
  & ({
    data: PropSpecData;
    enum?: string[] | number[];
    metadata: {
      createOnly?: boolean;
      readOnly?: boolean;
      writeOnly?: boolean;
      primaryIdentifier?: boolean;
      propPath: string[];
    };
  })
  & PropSpec;

type CreatePropQueue = {
  addTo: null | ((data: ExpandedPropSpec) => undefined);
  name: string;
  cfProp: CfProperty;
  propPath: string[];
}[];

export function createPropFromCf(
  name: string,
  cfProp: CfProperty,
  onlyProperties: OnlyProperties,
  propPath: string[],
): ExpandedPropSpec | undefined {
  if (!cfProp.type) {
    return undefined;
  }

  const queue: CreatePropQueue = [
    {
      name,
      cfProp,
      addTo: null,
      propPath,
    },
  ];

  let rootProp = undefined;

  while (queue.length > 0) {
    if (propPath.length > 10) {
      throw new Error(
        `Prop tree loop detected: Tried creating prop more than 10 levels deep in the prop tree: ${propPath}`,
      );
    }
    const data = queue.shift();
    if (!data) break;

    const prop = createPropFromCfInner(
      data.name,
      data.cfProp,
      onlyProperties,
      data.propPath,
      queue,
    );

    if (!prop) continue;

    if (!data.addTo) {
      rootProp = prop;
    } else {
      data.addTo(prop);
    }
  }

  if (!rootProp) {
    console.log(cfProp);
    throw new Error(`createProp for ${name} did not generate a prop`);
  }

  return rootProp;
}

function createPropFromCfInner(
  name: string,
  cfProp: CfProperty,
  onlyProperties: OnlyProperties,
  propPath: string[],
  queue: CreatePropQueue,
): ExpandedPropSpec | undefined {
  const propUniqueId = ulid();
  const data: PropSpecData = {
    name,
    validationFormat: null,
    defaultValue: null,
    funcUniqueId: null,
    inputs: [],
    widgetKind: null,
    widgetOptions: [],
    hidden: false,
    docLink: null,
    documentation: cfProp.description ?? null,
  };

  propPath.push(name);
  const partialProp: Partial<ExpandedPropSpec> = {
    name,
    data,
    uniqueId: propUniqueId,
    metadata: {
      createOnly: onlyProperties.createOnly.includes(name),
      readOnly: onlyProperties.readOnly.includes(name),
      writeOnly: onlyProperties.writeOnly.includes(name),
      primaryIdentifier: onlyProperties.primaryIdentifier.includes(name),
      propPath,
    },
  };

  if (partialProp.metadata?.createOnly) {
    setCreateOnlyProp(data);
  }

  if (!cfProp.title) {
    cfProp.title = name;
  }

  let normalizedCfData = normalizePropertyType(cfProp);
  normalizedCfData = normalizeAnyOfAndOneOfTypes(normalizedCfData);

  if (!normalizedCfData.type && normalizedCfData.$ref) {
    normalizedCfData = { ...normalizedCfData, type: "string" };
  }

  if (
    normalizedCfData.type === "integer" || normalizedCfData.type === "number"
  ) {
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "number" }>;
    prop.kind = "number";
    if (normalizedCfData.enum) {
      prop.data!.widgetKind = "ComboBox";
      for (const val of normalizedCfData.enum) {
        prop.data!.widgetOptions.push({
          label: val,
          value: val,
        });
      }
    } else {
      prop.data!.widgetKind = "Text";
    }

    return prop;
  } else if (normalizedCfData.type === "boolean") {
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "boolean" }>;
    prop.kind = "boolean";
    prop.data!.widgetKind = "Checkbox";

    return prop;
  } else if (normalizedCfData.type === "string") {
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "string" }>;
    prop.kind = "string";
    if (normalizedCfData.enum) {
      prop.data!.widgetKind = "ComboBox";
      for (const val of normalizedCfData.enum) {
        prop.data!.widgetOptions.push({
          label: val,
          value: val,
        });
      }
    } else {
      prop.data!.widgetKind = "Text";
    }

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
      propPath: _.clone(propPath),
    });

    return prop;
  } else if (normalizedCfData.type === "object") {
    if (normalizedCfData.patternProperties) {
      const prop = partialProp as Extract<ExpandedPropSpec, { kind: "map" }>;
      prop.kind = "map";
      prop.data!.widgetKind = "Map";

      const patternProps = Object.entries(normalizedCfData.patternProperties);

      let cfProp;
      if (patternProps.length === 1) {
        const [_thing, patternProp] = patternProps[0];
        cfProp = patternProp;
      } else if (patternProps.length === 2) {
        // If there is 2 pattern props, that means we have a validation for the key and another one for the value of the map.
        // We take the second one as the type of the value, since it's the thing we can store right now
        const [_thing, patternProp] = patternProps[1];
        cfProp = patternProp;
      } else {
        console.log(patternProps);
        throw new Error("too many pattern props you fool");
      }

      if (!cfProp) {
        throw new Error("could not extract type from pattern prop");
      }

      queue.push({
        addTo: (data: ExpandedPropSpec) => {
          prop.typeProp = data;
        },
        name: `${name}Item`,
        cfProp,
        propPath: _.clone(propPath),
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
            propPath: _.clone(propPath),
          });
        },
      );
      return prop;
    } else {
      const prop = partialProp as Extract<ExpandedPropSpec, { kind: "string" }>;
      prop.kind = "string";
      prop.data!.widgetKind = "Text";

      return prop;
    }
  }

  if (!normalizedCfData.type && normalizedCfData.description == "") {
    return undefined;
  }

  if (!normalizedCfData.type && normalizedCfData.title) {
    return undefined;
  }

  // console.log(cfProp);
  console.log(normalizedCfData);
  throw new Error(
    `no matching kind in prop with path: ${propPath}`,
  );
}

function setCreateOnlyProp(data: PropSpecData) {
  data.widgetOptions.push({
    label: CREATE_ONLY_PROP_LABEL,
    value: "true",
  });
}

export type DefaultPropType = "domain" | "secrets" | "resource_value";

export function createDefaultProp(
  type: DefaultPropType,
): Extract<ExpandedPropSpec, { kind: "object" }> {
  return createObjectProp(type, ["root"]);
}

export function createObjectProp(
  name: string,
  parentPath: string[],
): Extract<ExpandedPropSpec, { kind: "object" }> {
  const data: PropSpecData = {
    name,
    validationFormat: null,
    defaultValue: null,
    funcUniqueId: null,
    inputs: [],
    widgetKind: "Header",
    widgetOptions: [],
    hidden: null,
    docLink: null,
    documentation: null,
  };

  const prop: ExpandedPropSpec = {
    kind: "object",
    data,
    name,
    entries: [],
    uniqueId: ulid(),
    metadata: {
      createOnly: false,
      readOnly: false,
      writeOnly: false,
      primaryIdentifier: false,
      propPath: [...parentPath, name],
    },
  };

  return prop;
}

export function createScalarProp(
  name: string,
  kind: "number" | "string" | "boolean",
  parentPath: string[],
): ExpandedPropSpec {
  let widgetKind: PropSpecWidgetKind;
  switch (kind) {
    case "number":
    case "string":
      widgetKind = "Text";
      break;
    case "boolean":
      widgetKind = "Checkbox";
      break;
  }

  const data: PropSpecData = {
    name,
    validationFormat: null,
    defaultValue: null,
    funcUniqueId: null,
    inputs: [],
    widgetKind,
    widgetOptions: null,
    hidden: null,
    docLink: null,
    documentation: null,
  };

  const prop: ExpandedPropSpec = {
    kind,
    data,
    name,
    uniqueId: ulid(),
    metadata: {
      createOnly: false,
      readOnly: false,
      writeOnly: false,
      primaryIdentifier: false,
      propPath: [...parentPath, name],
    },
  };

  return prop;
}

export function bfsPropTree(
  prop: PropSpec,
  callback: (prop: PropSpec, parents: PropSpec[]) => unknown,
  options?: { skipTypeProps: boolean },
) {
  const queue = [{ prop, parents: [] }];

  while (queue.length > 0) {
    const queueItem = queue.pop();
    if (!queueItem) break;

    callback(queueItem.prop, queueItem.parents);

    const thisProp = queueItem.prop;
    const parents = _.clone(queueItem.parents);
    parents.unshift(thisProp);

    switch (thisProp.kind) {
      case "string":
      case "boolean":
      case "json":
      case "number":
        break;
      case "array":
      case "map":
        if (options?.skipTypeProps !== true) {
          queue.push({ prop: thisProp.typeProp, parents });
        }
        break;
      case "object": {
        const entries: PropSpec[] = _.sortBy(thisProp.entries, [
          "name",
          "kind",
        ]);
        entries.forEach((prop) => {
          queue.push({ prop, parents });
        });
        break;
      }
    }
  }
}

export function copyPropWithNewIds(
  sourceProp: ExpandedPropSpec,
): ExpandedPropSpec {
  const newProp = _.cloneDeep(sourceProp);

  bfsPropTree(newProp, (prop) => {
    prop.uniqueId = ulid();
  });

  return newProp;
}

export function generatePropHash(prop: PropSpec): string {
  const hasher = createHash("sha256");
  bfsPropTree(prop, (p) => {
    hasher.update(p.name);
    hasher.update(p.kind);
  });

  return hasher.digest("hex");
}
