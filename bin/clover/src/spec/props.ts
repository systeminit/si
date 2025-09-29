import { normalizeProperty } from "../cfDb.ts";
import { ulid } from "ulid";
import { PropSpecWidgetKind } from "../bindings/PropSpecWidgetKind.ts";
import { PropSpecData } from "../bindings/PropSpecData.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import _ from "lodash";
import ImportedJoi from "joi";
import { Extend } from "../extend.ts";
const { createHash } = await import("node:crypto");
import { cfPcreToRegexp } from "../pcre.ts";
import { ExpandedPkgSpec } from "./pkgs.ts";
import {
  CfObjectProperty,
  CfProperty,
  SuperSchema,
} from "../pipelines/types.ts";

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
  float: Extract<PropSpec, { kind: "float" }>;
  string: Extract<PropSpec, { kind: "string" }>;
  array: Extract<PropSpec, { kind: "array" }>;
  map: Extract<PropSpec, { kind: "map" }>;
  object: Extract<PropSpec, { kind: "object" }>;
};

export type ExpandedPropSpecFor = {
  boolean: Extend<PropSpecFor["boolean"], PropSpecOverrides>;
  json: Extend<PropSpecFor["json"], PropSpecOverrides>;
  number: Extend<PropSpecFor["number"], PropSpecOverrides>;
  float: Extend<PropSpecFor["float"], PropSpecOverrides>;
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

export interface PropSuggestion {
  schema: string;
  prop: string;
}

interface PropSpecOverrides {
  data: Extend<
    PropSpecData,
    {
      widgetOptions: { label: string; value: string }[] | null;
      widgetKind: PropSpecWidgetKind | null;
    }
  >;
  enum?: string[];
  metadata: {
    createOnly: boolean;
    readOnly: boolean;
    writeOnly: boolean;
    primaryIdentifier: boolean;
    propPath: string[];
    required: boolean;
  };
  joiValidation?: string;
  cfProp:
    | (CfProperty & {
      // The name of the definition this property lives under
      defName?: string;
    })
    | undefined;
}

export type CreatePropArgs = {
  // The path to this prop, e.g. ["root", "domain"]
  propPath: string[];
  // The definition for this prop in the schema
  cfProp: CfProperty & { defName?: string };
  // The parent prop's definition
  parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined;
  // A handler to add the prop to its parent after it has been created
  addTo?: (data: ExpandedPropSpec) => undefined;
};

export type CreatePropQueue = {
  superSchema: SuperSchema;
  onlyProperties: OnlyProperties;
  queue: CreatePropArgs[];
};

const MAX_PROP_DEPTH = 30;

export function findPropByName(
  objPropSpec: ExpandedPropSpecFor["object"],
  propName: string,
): ExpandedPropSpec | undefined {
  if (!objPropSpec.entries) {
    throw Error("findPropByName must be used on objects");
  }

  return objPropSpec.entries.find(
    (p) => p.name.toLowerCase() === propName.toLowerCase(),
  );
}

// Create top-level prop such as domain, resource_value, secrets, etc.
export function createDefaultPropFromCf(
  name: DefaultPropType,
  properties: Record<string, CfProperty>,
  superSchema: SuperSchema,
  onlyProperties: OnlyProperties,
): ExpandedPropSpecFor["object"] {
  // Enqueue the root prop only, and then iterate over its children
  let rootProp: ExpandedPropSpecFor["object"] | undefined;
  const queue: CreatePropQueue = {
    superSchema,
    onlyProperties,
    queue: [
      {
        propPath: ["root", name],
        // Pretend the prop only has the specified properties (since we split it up)
        cfProp: { ...superSchema, properties },
        parentProp: undefined,
        addTo: (prop: ExpandedPropSpec) => {
          if (prop.kind !== "object") {
            throw new Error(`${name} prop is not an object`);
          }
          // Set "rootProp" before returning it
          rootProp = prop;
        },
      },
    ],
  };

  while (queue.queue.length > 0) {
    const propArgs = queue.queue.shift()!;
    if (propArgs.propPath.length > MAX_PROP_DEPTH) {
      throw new Error(
        `Prop tree loop detected: Tried creating prop more than ${MAX_PROP_DEPTH} levels deep in the prop tree: ${propArgs.propPath}`,
      );
    }

    const prop = createPropFromCf(
      propArgs,
      queue,
      createDocLink,
      childIsRequired,
    );
    if (!prop) continue;
    if (propArgs.addTo) propArgs.addTo(prop);
  }

  if (!rootProp) {
    throw new Error(
      `createProp for ${superSchema.typeName} did not generate a ${name} prop`,
    );
  }

  // Top level prop doesn't actually get the generated doc; we add that to the schema instead
  rootProp.data.inputs = null;
  rootProp.data.widgetOptions = null;
  rootProp.data.hidden = false;
  rootProp.data.documentation = null;

  return rootProp;
}

export function createDefaultProp(
  name: DefaultPropType,
  cfProp: CfObjectProperty | undefined,
  required: boolean,
): ExpandedPropSpecFor["object"] {
  return createObjectProp(name, ["root"], cfProp, required);
}

export type DocFn = (
  { typeName }: SuperSchema,
  defName: string | undefined,
  propName?: string,
) => string;
export function createDocLink(
  { typeName }: SuperSchema,
  defName: string | undefined,
  propName?: string,
): string {
  // Figure out the snake case name of the resource to link to

  // AWS::EC2::SecurityGroup -> aws, ec2-securitygroup
  const [topLevelRef, ...typeRefParts] = typeName.toLowerCase().split("::");
  let kebabRef = typeRefParts.join("-");

  let docLink =
    "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide";

  // If the document refers to a definition, the link is a little different
  if (defName) {
    // AWS::EC2::SecurityGroup #/definitions/Ingress -> /aws-properties-ec2-securitygroup-ingress
    kebabRef += `-${defName.toLowerCase()}`;
    docLink += `/${topLevelRef}-properties-${kebabRef}.html`;
  } else {
    docLink += `/${topLevelRef}-resource-${kebabRef}.html`;
  }

  // If a property name is provided, reference the property with a fragment
  if (propName) {
    docLink += `#cfn-${kebabRef}-${propName.toLowerCase()}`;
  }
  return docLink;
}

export function createPropFromCf(
  { propPath, cfProp, parentProp }: CreatePropArgs,
  { superSchema, onlyProperties, queue }: CreatePropQueue,
  docFn: DocFn,
  requiredFn: requiredFn,
): ExpandedPropSpec | undefined {
  const name = propPath[propPath.length - 1];
  const required = requiredFn(superSchema, parentProp, name);
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
    docLink: parentProp?.kind === "object"
      ? docFn(superSchema, cfProp.defName, name)
      : null,
    documentation: cfProp.description ?? null,
    uiOptionals: null,
  };
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
      required,
    },
    cfProp,
  };

  if (partialProp.metadata?.createOnly) {
    setCreateOnlyProp(data);
  }

  if (!cfProp.title) {
    cfProp.title = name;
  }

  const normalizedCfProp = normalizeProperty(cfProp);

  if (
    normalizedCfProp.type === "integer" ||
    normalizedCfProp.type === "number"
  ) {
    let prop;
    if (normalizedCfProp.type === "integer") {
      prop = partialProp as ExpandedPropSpecFor["number"];
      prop.kind = "number";
    } else {
      prop = partialProp as ExpandedPropSpecFor["float"];
      prop.kind = "float";
    }
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
    if (required) validation += ".required()";
    if (validation) setJoiValidation(prop, `Joi.number()${validation}`);

    return prop;
  } else if (normalizedCfProp.type === "boolean") {
    const prop = partialProp as ExpandedPropSpecFor["boolean"];
    prop.kind = "boolean";
    prop.data.widgetKind = "Checkbox";

    // Add validation
    let validation = "";
    if (required) validation += ".required()";
    if (validation) setJoiValidation(prop, `Joi.boolean()${validation}`);

    return prop;
  } else if (normalizedCfProp.type === "string") {
    const prop = partialProp as ExpandedPropSpecFor["string"];
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
        case "iso-8601":
          // TODO
          break;
        case "decimal":
          // TODO
          break;
        case "string":
          // This seems meaningless (actually, the two fields that use it, QuickSight::DataSet::CreatedAt
          // and QuickSight::DataSet::LastUpdatedAt, are both number types in the actual API, so
          // it's not clear why these are strings in the first place)
          break;
        // This is a special case (and seems likely wrong), but may as well support it
        case "(^arn:[a-z\\d-]+:rekognition:[a-z\\d-]+:\\d{12}:collection\\/([a-zA-Z0-9_.\\-]+){1,255})":
          validation += `.pattern(new RegExp(${
            JSON.stringify(
              normalizedCfProp.format,
            )
          }))`;
          break;
        case undefined:
          break;
        default:
          throw new Error(`Unsupported format: ${normalizedCfProp.format}`);
      }
      if (normalizedCfProp.minLength !== undefined) {
        validation += `.min(${normalizedCfProp.minLength})`;
      }
      if (normalizedCfProp.maxLength !== undefined) {
        validation += `.max(${normalizedCfProp.maxLength})`;
      }
      if (normalizedCfProp.pattern !== undefined) {
        const toRegexp = cfPcreToRegexp(normalizedCfProp.pattern);
        if (toRegexp) {
          validation += `.pattern(new RegExp(${
            JSON.stringify(
              toRegexp.pattern,
            )
          }${toRegexp.flags ? `, ${JSON.stringify(toRegexp.flags)}` : ""}))`;
        }
      }
      if (required) validation += ".required()";
      if (validation) setJoiValidation(prop, `Joi.string()${validation}`);
    }
    return prop;
  } else if (normalizedCfProp.type === "json") {
    // TODO if this is gonna be json we should really check that it's valid json ...
    const prop = partialProp as ExpandedPropSpecFor["string"];
    prop.kind = "string";
    prop.data.widgetKind = "CodeEditor";

    // Add validation
    let validation = "";
    if (required) validation += ".required()";
    if (validation) setJoiValidation(prop, `Joi.string()${validation}`);

    return prop;
  } else if (normalizedCfProp.type === "array") {
    const prop = partialProp as ExpandedPropSpecFor["array"];
    prop.kind = "array";
    prop.data.widgetKind = "Array";

    queue.push({
      propPath: [...propPath, `${name}Item`],
      cfProp: normalizedCfProp.items,
      parentProp: prop,
      addTo: (data: ExpandedPropSpec) => {
        prop.typeProp = data;
      },
    });

    return prop;
  } else if (normalizedCfProp.type === "object") {
    if (normalizedCfProp.patternProperties) {
      const prop = partialProp as ExpandedPropSpecFor["map"];
      prop.kind = "map";
      prop.data.widgetKind = "Map";

      const patternProps = Object.entries(normalizedCfProp.patternProperties);

      let cfItemProp;
      if (patternProps.length === 1) {
        const [_thing, patternProp] = patternProps[0];
        cfItemProp = patternProp;
      } else if (patternProps.length === 2) {
        // If there is 2 pattern props, that means we have a validation for the key and another one for the value of the map.
        // We take the second one as the type of the value, since it's the thing we can store right now
        const [_thing, patternProp] = patternProps[1];
        cfItemProp = patternProp;
      } else {
        console.log(patternProps);
        throw new Error("too many pattern props you fool");
      }

      if (!cfItemProp) {
        throw new Error("could not extract type from pattern prop");
      }

      queue.push({
        cfProp: cfItemProp,
        propPath: [...propPath, `${name}Item`],
        parentProp: prop,
        addTo: (data: ExpandedPropSpec) => {
          prop.typeProp = data;
        },
      });

      return prop;
    } else if (normalizedCfProp.properties) {
      const prop = partialProp as ExpandedPropSpecFor["object"];
      prop.kind = "object";
      prop.data.widgetKind = "Header";
      prop.entries = [];
      for (
        const [name, childCfProp] of Object.entries(
          normalizedCfProp.properties,
        )
      ) {
        queue.push({
          cfProp: childCfProp,
          propPath: [...propPath, name],
          parentProp: prop,
          addTo: (childProp: ExpandedPropSpec) => {
            prop.entries.push(childProp);
          },
        });
      }
      return prop;
    } else {
      const prop = partialProp as ExpandedPropSpecFor["string"];
      prop.kind = "string";
      prop.data.widgetKind = "Text";

      // Add validation
      let validation = "";
      if (required) validation += ".required()";
      if (validation) setJoiValidation(prop, `Joi.string()${validation}`);

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

export type requiredFn = (
  superSchema: SuperSchema,
  parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
) => boolean;
function childIsRequired(
  superSchema: SuperSchema,
  parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
) {
  // If the parent is an object, then the child is required only if the parent is required
  // *and* the child is in the parent's "required" list
  if (parentProp?.kind === "object") {
    if (!parentProp?.metadata.required) return false;
    if (!parentProp.cfProp) return false;
    if (!("required" in parentProp.cfProp)) return false;
    return parentProp.cfProp.required?.includes(childName) ?? false;
  }
  // If the parent is the root prop, or an array or map, the child is required (i.e. if it
  // gets created then it must have a value).
  return true;
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

function setCreateOnlyProp(data: ExpandedPropSpec["data"]) {
  data.widgetOptions ??= [];
  data.widgetOptions.push({
    label: CREATE_ONLY_PROP_LABEL,
    value: "true",
  });
}

export type DefaultPropType =
  | "domain"
  | "secrets"
  | "secret_definition"
  | "resource_value";

export function createObjectProp(
  name: string,
  parentPath: string[],
  cfProp: CfObjectProperty | undefined,
  required: boolean,
): Extract<ExpandedPropSpec, { kind: "object" }> {
  const data: ExpandedPropSpec["data"] = {
    name,
    validationFormat: null,
    defaultValue: null,
    funcUniqueId: null,
    inputs: [],
    widgetKind: "Header",
    widgetOptions: [],
    hidden: false,
    docLink: null,
    documentation: null,
    uiOptionals: null,
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
      required,
    },
    cfProp,
  };

  return prop;
}

export function createScalarProp(
  name: string,
  kind: "number" | "string" | "boolean",
  parentPath: string[],
  required: boolean,
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
    hidden: false,
    docLink: null,
    documentation: null,
    uiOptionals: null,
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
      required,
    },
    cfProp: undefined,
  };

  return prop;
}

export function bfsPropTree(
  prop: ExpandedPropSpec | (ExpandedPropSpec | null)[],
  callback: (prop: ExpandedPropSpec, parents: ExpandedPropSpec[]) => unknown,
  options?: { skipTypeProps: boolean },
) {
  if (Array.isArray(prop)) {
    for (const p of prop) {
      if (p !== null) {
        bfsPropTree(p, callback, options);
      }
    }
    return;
  }

  const queue = [{ prop, parents: [] as ExpandedPropSpec[] }];

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

export function addPropSuggestSource(
  prop: ExpandedPropSpec,
  suggestion: PropSuggestion,
): ExpandedPropSpec {
  prop.data.uiOptionals ??= {};

  prop.data.uiOptionals.suggestSources = [
    ...(prop.data.uiOptionals.suggestSources ?? []),
    suggestion,
  ];
  return prop;
}

export function addPropSuggestAsSourceFor(
  prop: ExpandedPropSpec,
  suggestion: PropSuggestion,
): ExpandedPropSpec {
  prop.data.uiOptionals ??= {};

  prop.data.uiOptionals.suggestAsSourceFor = [
    ...(prop.data.uiOptionals.suggestAsSourceFor ?? []),
    suggestion,
  ];
  return prop;
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

export function propSuggestionFor(
  variant: ExpandedPkgSpec,
  prop: ExpandedPropSpec,
): PropSuggestion {
  return {
    schema: variant.name,
    // stripping /root out
    prop: "/" + prop.metadata.propPath.slice(1).join("/"),
  };
}
