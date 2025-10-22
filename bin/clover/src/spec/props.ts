import util from "node:util";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";
import { PropSpecWidgetKind } from "../bindings/PropSpecWidgetKind.ts";
import { PropSpecData } from "../bindings/PropSpecData.ts";
import { PropSpec } from "../bindings/PropSpec.ts";
import type { JsonValue } from "../bindings/serde_json/JsonValue.ts";
import _ from "lodash";
import ImportedJoi from "npm:joi@17.13.3";
import { Extend } from "../extend.ts";
const { createHash } = await import("node:crypto");
import { cfPcreToRegexp } from "../pcre.ts";
import { ExpandedPkgSpec } from "./pkgs.ts";
import {
  CfObjectProperty,
  CfProperty,
  ProviderConfig,
  SuperSchema,
} from "../pipelines/types.ts";
import { JSONSchema } from "../pipelines/draft_07.ts";
import logger from "../logger.ts";

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

// Type guard to check if a JsonValue is a PropSuggestion array
function isPropSuggestionArray(value: unknown): value is PropSuggestion[] {
  return (
    Array.isArray(value) &&
    (value.length === 0 ||
      (typeof value[0] === "object" &&
        value[0] !== null &&
        "schema" in value[0] &&
        "prop" in value[0]))
  );
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

const MAX_PROP_DEPTH = 40;

export type PropPath = `/${"domain" | "resource_value"}/${string}`;

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
export function createDefaultPropFromJsonSchema(
  name: DefaultPropType,
  properties: Record<string, JSONSchema>,
  schema: SuperSchema,
  onlyProperties: OnlyProperties,
  docFn: DocFn,
  providerConfig: ProviderConfig,
): ExpandedPropSpecFor["object"] {
  const propsBeingCreated: JSONSchema[] = [];

  // Recursively create prop and all its children
  const rootProp = createPropFromJsonSchema(
    ["root", name],
    { ...schema, type: "object", properties },
    undefined,
  );

  if (rootProp?.kind !== "object") {
    throw new Error(
      `createProp for ${schema.typeName} did not generate a ${name} object prop`,
    );
  }

  // Top level prop doesn't actually get the generated doc; we add that to the schema instead
  rootProp.data.inputs = null;
  rootProp.data.widgetOptions = null;
  rootProp.data.hidden = false;
  rootProp.data.documentation = null;

  return rootProp;

  function createPropFromJsonSchema(
    // The path to this prop, e.g. ["root", "domain"]
    propPath: string[],
    // The definition for this prop in the schema
    schemaProp: JSONSchema & { defName?: string },
    // The parent prop's definition
    parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  ): ExpandedPropSpec | undefined {
    if (propPath.length > MAX_PROP_DEPTH) {
      throw new Error(
        `Prop tree loop detected: Tried creating prop more than ${MAX_PROP_DEPTH} levels deep in the prop tree: ${propPath.join(
          "/",
        )}`,
      );
    }

    if (propsBeingCreated.includes(schemaProp)) {
      logger.debug(`Circular property definition: ${propPath.join("/")}`);
      return undefined;
    }
    try {
      propsBeingCreated.push(schemaProp);

      // Apply provider-specific normalization
      const cfProp = providerConfig.normalizeProperty(schemaProp, {
        propPath,
        schema,
        parentProp,
      });

      const name = propPath[propPath.length - 1];
      const required = providerConfig.isChildRequired(schema, parentProp, name);
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
        docLink:
          parentProp?.kind === "object"
            ? docFn(schema, schemaProp.defName, name)
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

      if (cfProp.type === "integer" || cfProp.type === "number") {
        let prop;
        if (cfProp.type === "integer") {
          prop = partialProp as ExpandedPropSpecFor["number"];
          prop.kind = "number";
        } else {
          prop = partialProp as ExpandedPropSpecFor["float"];
          prop.kind = "float";
        }
        if (cfProp.enum) {
          prop.data.widgetKind = "ComboBox";
          for (const val of cfProp.enum) {
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
        if (cfProp.type === "integer") {
          validation += ".integer()";
        }
        if (cfProp.minimum !== undefined) {
          validation += `.min(${cfProp.minimum})`;
        }
        if (cfProp.maximum !== undefined) {
          validation += `.max(${cfProp.maximum})`;
        }
        if (cfProp.exclusiveMinimum !== undefined) {
          validation += `.greater(${cfProp.exclusiveMinimum})`;
        }
        if (cfProp.exclusiveMaximum !== undefined) {
          validation += `.less(${cfProp.exclusiveMaximum})`;
        }
        if (cfProp.multipleOf !== undefined) {
          validation += `.multiple(${cfProp.multipleOf})`;
        }
        switch (cfProp.format) {
          case "int64":
          case "double":
            // These formats are inherent to JS
            break;
          case undefined:
            break;
          default:
            throw new Error(`Unsupported number format: ${cfProp.format}`);
        }
        if (required) validation += ".required()";
        if (validation) setJoiValidation(prop, `Joi.number()${validation}`);

        return prop;
      } else if (cfProp.type === "boolean") {
        const prop = partialProp as ExpandedPropSpecFor["boolean"];
        prop.kind = "boolean";
        prop.data.widgetKind = "Checkbox";

        // Add validation
        let validation = "";
        if (required) validation += ".required()";
        if (validation) setJoiValidation(prop, `Joi.boolean()${validation}`);

        return prop;
      } else if (cfProp.type === "string") {
        const prop = partialProp as ExpandedPropSpecFor["string"];
        prop.kind = "string";
        if (cfProp.enum) {
          prop.data.widgetKind = "ComboBox";
          for (const val of cfProp.enum) {
            prop.data.widgetOptions!.push({
              label: val,
              value: val,
            });
          }
        } else {
          prop.data.widgetKind = "Text";
        }

        // Add validation
        if (cfProp.format === "date-time" || cfProp.format === "timestamp") {
          prop.joiValidation = "Joi.date().iso()";
        } else {
          let validation = "";

          // https://json-schema.org/understanding-json-schema/reference/type#built-in-formats
          switch (cfProp.format) {
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
              validation += `.pattern(new RegExp(${JSON.stringify(
                cfProp.format,
              )}))`;
              break;
            case "base64url":
              // TODO ADD VALIDATION FOR THIS
              break;
            case undefined:
              break;
            default:
              throw new Error(`Unsupported format: ${cfProp.format}`);
          }
          if (cfProp.minLength !== undefined) {
            validation += `.min(${cfProp.minLength})`;
          }
          if (cfProp.maxLength !== undefined) {
            validation += `.max(${cfProp.maxLength})`;
          }
          if (cfProp.pattern !== undefined) {
            const toRegexp = cfPcreToRegexp(cfProp.pattern);
            if (toRegexp) {
              validation += `.pattern(new RegExp(${JSON.stringify(
                toRegexp.pattern,
              )}${
                toRegexp.flags ? `, ${JSON.stringify(toRegexp.flags)}` : ""
              }))`;
            }
          }
          if (required) validation += ".required()";
          if (validation) setJoiValidation(prop, `Joi.string()${validation}`);
        }
        return prop;
      } else if (cfProp.type === "json") {
        // TODO if this is gonna be json we should really check that it's valid json ...
        const prop = partialProp as ExpandedPropSpecFor["string"];
        prop.kind = "string";
        prop.data.widgetKind = "CodeEditor";

        // Add validation
        let validation = "";
        if (required) validation += ".required()";
        if (validation) setJoiValidation(prop, `Joi.string()${validation}`);

        return prop;
      } else if (cfProp.type === "array") {
        const prop = partialProp as ExpandedPropSpecFor["array"];
        prop.kind = "array";
        prop.data.widgetKind = "Array";

        const typeProp = createPropFromJsonSchema(
          [...propPath, `${name}Item`],
          cfProp.items,
          prop,
        );
        if (!typeProp) {
          logger.warn(
            `Ignoring ${propPath.join("/")}: unable to create array item type`,
          );
          return undefined;
        }
        prop.typeProp = typeProp;

        return prop;
      } else if (cfProp.type === "object") {
        if (cfProp.patternProperties || cfProp.additionalProperties) {
          const prop = partialProp as ExpandedPropSpecFor["map"];
          prop.kind = "map";
          prop.data.widgetKind = "Map";

          const itemPath = [...propPath, `${name}Item`];
          let typeProp: ExpandedPropSpec | undefined;

          if (cfProp.patternProperties) {
            const patternProps = Object.entries(cfProp.patternProperties);

            if (patternProps.length === 1) {
              const [_thing, patternProp] = patternProps[0];
              typeProp = createPropFromJsonSchema(itemPath, patternProp, prop);
            } else if (patternProps.length === 2) {
              // If there is 2 pattern props, that means we have a validation for the key and another one for the value of the map.
              // We take the second one as the type of the value, since it's the thing we can store right now
              const [_thing, patternProp] = patternProps[1];
              typeProp = createPropFromJsonSchema(itemPath, patternProp, prop);
            } else {
              console.log(patternProps);
              throw new Error("too many pattern props you fool");
            }
          } else if (cfProp.additionalProperties) {
            // Use additionalProperties as the map value type
            typeProp = createPropFromJsonSchema(
              itemPath,
              cfProp.additionalProperties,
              prop,
            );
          }

          if (!typeProp) {
            logger.warn(
              `Ignoring ${propPath.join("/")}: unable to create map item type`,
            );
            return undefined;
          }
          prop.typeProp = typeProp;

          return prop;
        } else if (cfProp.properties) {
          const prop = partialProp as ExpandedPropSpecFor["object"];
          prop.kind = "object";
          prop.data.widgetKind = "Header";
          prop.entries = [];
          for (const [name, childCfProp] of Object.entries(cfProp.properties)) {
            const childProp = createPropFromJsonSchema(
              [...propPath, name],
              childCfProp,
              prop,
            );
            // Just skip props we don't understand
            if (childProp) prop.entries.push(childProp);
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

      if (!cfProp.type && cfProp.description == "") {
        logger.warn(
          `No type + empty description for top level prop at ${propPath.join(
            "/",
          )}: ${util.inspect(cfProp)}`,
        );
        return undefined;
      }

      if (!cfProp.type && cfProp.title) {
        logger.warn(
          `No type for top level prop at ${propPath.join("/")}: ${util.inspect(
            cfProp,
          )}`,
        );
        return undefined;
      }

      // console.log(cfProp);
      console.log(cfProp);
      throw new Error(
        `no matching kind in prop with path: ${propPath.join("/")}`,
      );
    } catch (e) {
      console.error(
        `Error creating prop for ${schema.typeName} at ${propPath.join("/")}`,
      );
      throw e;
    } finally {
      propsBeingCreated.pop();
    }
  }
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

type requiredFn = (
  superSchema: SuperSchema,
  parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
) => boolean;

function setJoiValidation(prop: ExpandedPropSpec, joiValidation: string) {
  prop.joiValidation = joiValidation;
  // Used in the eval() below
  // deno-lint-ignore no-unused-vars
  const Joi = ImportedJoi;
  try {
    prop.data.validationFormat = JSON.stringify(eval(joiValidation).describe());
  } catch (e) {
    logger.error(joiValidation);
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

  const existing = prop.data.uiOptionals.suggestSources;
  const existingArray = isPropSuggestionArray(existing) ? existing : [];
  prop.data.uiOptionals.suggestSources = [
    ...existingArray,
    suggestion,
  ] as unknown as JsonValue;
  return prop;
}

export function addPropSuggestAsSourceFor(
  prop: ExpandedPropSpec,
  suggestion: PropSuggestion,
): ExpandedPropSpec {
  prop.data.uiOptionals ??= {};

  const existing = prop.data.uiOptionals.suggestAsSourceFor;
  const existingArray = isPropSuggestionArray(existing) ? existing : [];
  prop.data.uiOptionals.suggestAsSourceFor = [
    ...existingArray,
    suggestion,
  ] as unknown as JsonValue;
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
    prop: propPathStr(prop),
  };
}

export function propPathStr(prop: ExpandedPropSpec): PropPath {
  return toPropPath(prop.metadata.propPath);
}

export function toPropPathArray(
  propPath: PropPath,
): ExpandedPropSpec["metadata"]["propPath"] {
  if (propPath[0] !== "/") {
    throw new Error(`propPath must start with /: ${propPath}`);
  }
  return ["root", ...propPath.split("/").slice(1)];
}

export function toPropPath(
  propPathArray: ExpandedPropSpec["metadata"]["propPath"],
): PropPath {
  if (propPathArray[0] !== "root") {
    throw new Error(`propPath array must start with root: ${propPathArray}`);
  }
  if (propPathArray[1] !== "domain" && propPathArray[1] !== "resource_value") {
    throw new Error(
      `propPath array must start with root/domain or resource_value: ${propPathArray}`,
    );
  }
  return ("/" + propPathArray.slice(1).join("/")) as PropPath;
}
