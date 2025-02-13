import { CfProperty, normalizeProperty } from "../cfDb.ts";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import { PropSpecData } from "../bindings/PropSpecData.ts";
import { PropSpecWidgetKind } from "../bindings/PropSpecWidgetKind.ts";
import _ from "npm:lodash";
import ImportedJoi from "joi";
import { Extend } from "../extend.ts";
const { createHash } = await import("node:crypto");

export const CREATE_ONLY_PROP_LABEL = "si_create_only_prop";

export type OnlyProperties = {
  createOnly: string[];
  readOnly: string[];
  writeOnly: string[];
  primaryIdentifier: string[];
};

// PropSpecFor["object"], etc.
export type PropSpecFor = {
  boolean: Extract<PropSpec, { kind: "boolean" }>;
  json: Extract<PropSpec, { kind: "json" }>;
  number: Extract<PropSpec, { kind: "number" }>;
  string: Extract<PropSpec, { kind: "string" }>;
  array: Extract<PropSpec, { kind: "array" }>;
  map: Extract<PropSpec, { kind: "map" }>;
  object: Extract<PropSpec, { kind: "object" }>;
};
export type ExpandedPropSpecFor = {
  boolean: Extend<PropSpecFor["boolean"], PropSpecOverrides>;
  json: Extend<PropSpecFor["json"], PropSpecOverrides>;
  number: Extend<PropSpecFor["number"], PropSpecOverrides>;
  string: Extend<PropSpecFor["string"], PropSpecOverrides>;
  array: Extend<
    PropSpecFor["array"],
    PropSpecOverrides & { typeProp: ExpandedPropSpec }
  >;
  map: Extend<
    PropSpecFor["map"],
    PropSpecOverrides & { typeProp: ExpandedPropSpec }
  >;
  object: Extend<
    PropSpecFor["object"],
    PropSpecOverrides & { entries: ExpandedPropSpec[] }
  >;
};

export type ExpandedPropSpec = ExpandedPropSpecFor[keyof ExpandedPropSpecFor];

interface PropSpecOverrides {
  data: Extend<
    PropSpecData,
    { widgetOptions: { label: string; value: string }[] | null }
  >;
  enum?: string[] | number[];
  metadata: {
    createOnly: boolean;
    readOnly: boolean;
    writeOnly: boolean;
    primaryIdentifier: boolean;
    propPath: string[];
  };
  joiValidation?: string;
}

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
  typeName: string,
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
      typeName,
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

function createDocLink(typeName: string, propName: string): string | null {
  const typeNameSnake = typeName.split("::").slice(1).join("-").toLowerCase();
  return `https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-${typeNameSnake}.html#cfn-${typeNameSnake}-${propName.toLowerCase()}`;
}

function createPropFromCfInner(
  name: string,
  cfProp: CfProperty,
  onlyProperties: OnlyProperties,
  typeName: string,
  propPath: string[],
  queue: CreatePropQueue,
): ExpandedPropSpec | undefined {
  const propUniqueId = ulid();
  const data: ExpandedPropSpec["data"] = {
    name,
    validationFormat: null,
    defaultValue: null,
    funcUniqueId: null,
    inputs: [],
    widgetKind: null,
    widgetOptions: [],
    hidden: false,
    docLink: createDocLink(typeName, name),
    documentation: cfProp.description ?? null,
  };
  propPath.push(name);
  const partialProp: Partial<ExpandedPropSpec> = {
    name,
    data,
    uniqueId: propUniqueId,
    metadata: {
      // cfProp,
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

  const normalizedCfProp = normalizeProperty(cfProp);

  if (
    normalizedCfProp.type === "integer" || normalizedCfProp.type === "number"
  ) {
    const prop = partialProp as ExpandedPropSpecFor["number"];
    prop.kind = "number";
    if (normalizedCfProp.enum) {
      prop.data.widgetKind = "ComboBox";
      for (const val of normalizedCfProp.enum) {
        const valString = val.toString();
        prop.data.widgetOptions!.push({
          label: valString,
          value: valString,
        });
      }
    } else {
      prop.data.widgetKind = "Text";
    }

    // Add validation
    let validation = "";
    if (normalizedCfProp.type === "integer") {
      validation += ".integer()";
    }
    if (normalizedCfProp.minimum !== undefined) {
      validation += `.min(${normalizedCfProp.minimum})`;
    }
    if (normalizedCfProp.maximum !== undefined) {
      validation += `.max(${normalizedCfProp.maximum})`;
    }
    if (normalizedCfProp.exclusiveMinimum !== undefined) {
      validation += `.greater(${normalizedCfProp.exclusiveMinimum})`;
    }
    if (normalizedCfProp.exclusiveMaximum !== undefined) {
      validation += `.less(${normalizedCfProp.exclusiveMaximum})`;
    }
    if (normalizedCfProp.multipleOf !== undefined) {
      validation += `.multiple(${normalizedCfProp.multipleOf})`;
    }
    switch (normalizedCfProp.format) {
      case "int64":
      case "double":
        // These formats are inherent to JS
        break;
      case undefined:
        break;
      default:
        throw new Error(
          `Unsupported number format: ${normalizedCfProp.format}`,
        );
    }
    if (validation) {
      setJoiValidation(prop, `Joi.number()${validation}`);
    }

    return prop;
  } else if (normalizedCfProp.type === "boolean") {
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "boolean" }>;
    prop.kind = "boolean";
    prop.data.widgetKind = "Checkbox";

    return prop;
  } else if (normalizedCfProp.type === "string") {
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "string" }>;
    prop.kind = "string";
    if (normalizedCfProp.enum) {
      prop.data.widgetKind = "ComboBox";
      for (const val of normalizedCfProp.enum) {
        prop.data.widgetOptions!.push({
          label: val,
          value: val,
        });
      }
    } else {
      prop.data.widgetKind = "Text";
    }

    // Add validation
    if (
      normalizedCfProp.format === "date-time" ||
      normalizedCfProp.format === "timestamp"
    ) {
      prop.joiValidation = "Joi.date().iso()";
    } else {
      let validation = "";

      // https://json-schema.org/understanding-json-schema/reference/type#built-in-formats
      switch (normalizedCfProp.format) {
        case "uri":
          validation += ".uri()";
          break;
        case "json-pointer":
          // https://tools.ietf.org/html/rfc6901
          // We don't validate the whole thing there, but we at least check that it starts with slash!
          validation += `.pattern(/^\\//)`;
          break;
        case "string":
          // This seems meaningless (actually, the two fields that use it, QuickSight::DataSet::CreatedAt
          // and QuickSight::DataSet::LastUpdatedAt, are both number types in the actual API, so
          // it's not clear why these are strings in the first place)
          break;
        // This is a special case (and seems likely wrong), but may as well support it
        case "(^arn:[a-z\\d-]+:rekognition:[a-z\\d-]+:\\d{12}:collection\\/([a-zA-Z0-9_.\\-]+){1,255})":
          validation += `.pattern(new RegExp(${
            JSON.stringify(normalizedCfProp.format)
          }))`;
          break;
        case undefined:
          break;
        default:
          throw new Error(
            `Unsupported format: ${normalizedCfProp.format}`,
          );
      }
      if (normalizedCfProp.minLength !== undefined) {
        validation += `.min(${normalizedCfProp.minLength})`;
      }
      if (normalizedCfProp.maxLength !== undefined) {
        validation += `.max(${normalizedCfProp.maxLength})`;
      }
      if (normalizedCfProp.pattern !== undefined) {
        if (
          !shouldIgnorePattern(typeName, normalizedCfProp.pattern)
        ) {
          validation += `.pattern(new RegExp(${
            JSON.stringify(normalizedCfProp.pattern)
          }))`;
        }
      }
      if (validation) {
        try {
          setJoiValidation(prop, `Joi.string()${validation}`);
        } catch (e) {
          if (normalizedCfProp.pattern !== undefined) {
            console.log(
              `If this is a regex syntax error, add this to IGNORE_PATTERNS:
                ${JSON.stringify(typeName)}: [ ${
                JSON.stringify(normalizedCfProp.pattern)
              }, ],
              `,
            );
          }
          throw e;
        }
      }
    }
    return prop;
  } else if (normalizedCfProp.type === "json") {
    // TODO if this is gonna be json we should really check that it's valid json ...
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "string" }>;
    prop.kind = "string";
    prop.data.widgetKind = "TextArea";

    return prop;
  } else if (normalizedCfProp.type === "array") {
    const prop = partialProp as Extract<ExpandedPropSpec, { kind: "array" }>;
    prop.kind = "array";
    prop.data.widgetKind = "Array";

    queue.push({
      addTo: (data: ExpandedPropSpec) => {
        prop.typeProp = data;
      },
      name: `${name}Item`,
      cfProp: normalizedCfProp.items,
      propPath: _.clone(propPath),
    });

    return prop;
  } else if (normalizedCfProp.type === "object") {
    if (normalizedCfProp.patternProperties) {
      const prop = partialProp as Extract<ExpandedPropSpec, { kind: "map" }>;
      prop.kind = "map";
      prop.data.widgetKind = "Map";

      const patternProps = Object.entries(normalizedCfProp.patternProperties);

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
    } else if (normalizedCfProp.properties) {
      const prop = partialProp as Extract<ExpandedPropSpec, { kind: "object" }>;
      prop.kind = "object";
      prop.data.widgetKind = "Header";
      prop.entries = [];

      Object.entries(normalizedCfProp.properties).forEach(
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
      prop.data.widgetKind = "Text";

      return prop;
    }
  }

  if (!normalizedCfProp.type && normalizedCfProp.description == "") {
    return undefined;
  }

  if (!normalizedCfProp.type && normalizedCfProp.title) {
    return undefined;
  }

  // console.log(cfProp);
  console.log(normalizedCfProp);
  throw new Error(`no matching kind in prop with path: ${propPath}`);
}

function setJoiValidation(prop: ExpandedPropSpec, joiValidation: string) {
  prop.joiValidation = joiValidation;
  // Used in the eval() below
  // deno-lint-ignore no-unused-vars
  const Joi = ImportedJoi;
  try {
    prop.data.validationFormat = JSON.stringify(eval(joiValidation).describe());
  } catch (e) {
    console.error(joiValidation);
    throw e;
  }
}

// Some patterns in cloudformation are broken and need to be ignored
const IGNORE_PATTERNS = {
  "AWS::Amplify::App": ["(?s).+", "(?s).*"],
  "AWS::Amplify::Branch": ["(?s).+", "(?s).*"],
  "AWS::Amplify::Domain": ["(?s).+", "(?s).*"],
  "AWS::AppFlow::Flow": [
    "[\\u0020-\\uD7FF\\uE000-\\uFFFD\\uD800\\uDC00-\\uDBFF\\uDFFF\\t]*",
  ],
  "AWS::CloudFormation::GuardHook": [
    "^(?!(?i)aws)[A-Za-z0-9]{2,64}::[A-Za-z0-9]{2,64}::[A-Za-z0-9]{2,64}$",
  ],
  "AWS::CloudFormation::LambdaHook": [
    "^(?!(?i)aws)[A-Za-z0-9]{2,64}::[A-Za-z0-9]{2,64}::[A-Za-z0-9]{2,64}$",
  ],
  "AWS::CloudTrail::Dashboard": ["(?s).*"],
  "AWS::FinSpace::Environment": [
    "^[a-zA-Z-0-9-:\\/]*{1,1000}$",
    "^[a-zA-Z-0-9-:\\/.]*{1,1000}$",
  ],
  "AWS::GameLift::GameServerGroup": ["[ -퟿-�𐀀-􏿿\r\n\t]*"],
  "AWS::Invoicing::InvoiceUnit": ["^(?! )[\\p{L}\\p{N}\\p{Z}-_]*(?<! )$"],
  "AWS::OpsWorksCM::Server": [
    "(?s)\\s*-----BEGIN CERTIFICATE-----.+-----END CERTIFICATE-----\\s*",
    "(?ms)\\s*^-----BEGIN (?-s:.*)PRIVATE KEY-----$.*?^-----END (?-s:.*)PRIVATE KEY-----$\\s*",
    "(?s).*",
  ],
  "AWS::SageMaker::FeatureGroup": [
    "[\\u0020-\\uD7FF\\uE000-\\uFFFD\\uD800\\uDC00-\\uDBFF\\uDFFF\t]*",
  ],
} as Record<string, string[]>;

function shouldIgnorePattern(
  typeName: string,
  pattern: string,
): boolean {
  return IGNORE_PATTERNS[typeName]?.indexOf(pattern) >= 0;
}

function setCreateOnlyProp(data: ExpandedPropSpec["data"]) {
  data.widgetOptions ??= [];
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
  const data: ExpandedPropSpec["data"] = {
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

  const data: ExpandedPropSpec["data"] = {
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
  prop: ExpandedPropSpec | ExpandedPropSpec[],
  callback: (prop: ExpandedPropSpec, parents: ExpandedPropSpec[]) => unknown,
  options?: { skipTypeProps: boolean },
) {
  if (Array.isArray(prop)) {
    for (const p of prop) {
      bfsPropTree(p, callback, options);
    }
    return;
  }

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
        const entries: typeof thisProp.entries = _.sortBy(thisProp.entries, [
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

export function copyPropWithNewIds<T extends ExpandedPropSpec>(
  sourceProp: T,
): T {
  const newProp: T = _.cloneDeep(sourceProp);

  bfsPropTree(newProp, (prop) => {
    prop.uniqueId = ulid();
  });

  return newProp;
}

export function generatePropHash(prop: ExpandedPropSpec): string {
  const hasher = createHash("sha256");
  bfsPropTree(prop, (p) => {
    hasher.update(p.name);
    hasher.update(p.kind);
  });

  return hasher.digest("hex");
}
