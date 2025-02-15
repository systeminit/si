import {
  type JSONSchema,
} from "https://deno.land/x/json_schema_typed@v8.0.0/draft_07.ts";
import $RefParser from "npm:@apidevtools/json-schema-ref-parser";
import _logger from "./logger.ts";
import { ServiceMissing } from "./errors.ts";
import _ from "npm:lodash";
import { Extend } from "./extend.ts";

const logger = _logger.ns("cfDb").seal();

type JSONPointer = string;

const CF_PROPERTY_TYPES = [
  "boolean",
  "string",
  "number",
  "integer",
  "object",
  "array",
  "json",
] as const;
export type CfPropertyType = typeof CF_PROPERTY_TYPES[number];

export type CfProperty =
  | Extend<CfBooleanProperty, { type: "boolean" }>
  | Extend<CfStringProperty, { type: "string" }>
  | Extend<CfNumberProperty, { type: "number" }>
  | Extend<CfIntegerProperty, { type: "integer" }>
  | Extend<CfArrayProperty, { type: "array" }>
  | CfObjectProperty // We may infer object-ness if type is undefined but other props are there
  | Omit<JSONSchema.String, "type"> & { type: "json" }
  | CfMultiTypeProperty
  // Then we have this mess of array typed properties
  | Extend<JSONSchema.Interface, {
    properties?: Record<string, CfProperty>;
    type: ["string", CfPropertyType] | [
      CfPropertyType,
      "string",
    ];
  }>;

export type CfBooleanProperty = JSONSchema.Boolean;

export type CfStringProperty = JSONSchema.String;

export type CfNumberProperty = JSONSchema.Number & {
  format?: string;
};

export type CfIntegerProperty = JSONSchema.Integer & {
  format?: string;
};

export type CfArrayProperty = Extend<JSONSchema.Array, {
  // For properties of type array, defines the data structure of each array item.
  // Contains a single schema. A list of schemas is not allowed.
  items: CfProperty;
  // For properties of type array, set to true to specify that the order in which array items are specified must be honored, and that changing the order of the array will indicate a change in the property.
  // The default is true.
  insertionOrder?: boolean;
}>;

export type CfObjectProperty = Extend<JSONSchema.Object, {
  properties?: Record<string, CfProperty>;
  // e.g. patternProperties: { "^[a-z]+": { type: "string" } }
  patternProperties?: Record<string, CfProperty>;
  // Any properties that are required if this property is specified.
  dependencies?: Record<string, string[]>;
  oneOf?: CfObjectProperty[];
  anyOf?: CfObjectProperty[];
  allOf?: CfObjectProperty[];
}>;

type CfMultiTypeProperty =
  & Pick<JSONSchema.Interface, "$ref" | "$comment" | "title" | "description">
  & {
    type?: undefined;
    oneOf?: CfProperty[];
    anyOf?: CfProperty[];
  };

type StringPair =
  | ["string", "object"]
  | ["string", "boolean"]
  | ["string", "number"]
  | ["string", "integer"]
  | ["object", "string"]
  | ["boolean", "string"]
  | ["number", "string"]
  | ["string", "array"]
  | ["integer", "string"];

export function normalizeProperty(
  prop: CfProperty,
): CfProperty {
  const normalizedCfData = normalizePropertyType(prop);
  return normalizeAnyOfAndOneOfTypes(normalizedCfData);
}

function normalizePropertyType(
  prop: CfProperty,
): CfProperty {
  // If it already has a single type, return the prop as-is
  if (typeof prop.type === "string") return prop;

  // Infer type when there is none.
  if (prop.type === undefined) {
    // Some props have no type but we can duck type them to objects
    if (isCfObjectProperty(prop)) {
      return { ...prop, type: "object" } as CfObjectProperty;
    }

    // TODO we really need to look inside the ref here rather than assuming string ...
    if (prop.$ref) {
      return { ...prop, type: "string" } as CfProperty;
    }

    // If it's a multi-type thing, return it--we don't really handle these yet.
    return prop;
  }

  // The only remaining possible type is array.

  // If the cf type is an array, it's always string+something, and we use that something
  // to guess the best type we should use
  const nonStringType = prop.type.find((t) => t !== "string");

  let type: CfPropertyType;
  switch (nonStringType) {
    case "boolean":
    case "integer":
    case "number":
      type = nonStringType;
      break;
    case "object":
      // If it's an object we make it a json type, which will become a string type + textArea widget
      type = "json";
      break;
    case "array": {
      // When we get something that is string/array, the items object should already there
      if (!("items" in prop)) {
        throw new Error("array typed prop includes array but has no items");
      }
      type = "array";
      break;
    }
    default:
      console.log(prop);
      throw new Error("unhandled array type");
  }
  return { ...prop, type } as CfProperty;
}

function normalizeAnyOfAndOneOfTypes(
  prop: CfProperty,
): CfProperty {
  if (prop.type) return prop;

  if (prop.oneOf) {
    const newProp: CfObjectProperty = {
      description: prop.description,
      type: "object",
      properties: {},
    };

    for (const ofMember of prop.oneOf) {
      if (!newProp.properties) {
        throw new Error("unexpected oneOf");
      }

      if (ofMember.type === "object" && ofMember.properties) {
        for (const title of _.keys(ofMember.properties)) {
          newProp.properties[title] = ofMember.properties[title];
        }
      } else if (ofMember.type === "array" && ofMember.items) {
        const title = ofMember.title ??
          `${prop.title}${_.capitalize(ofMember.type)}`;
        if (!title) {
          console.log(prop);
          throw new Error(
            `oneOf array without title`,
          );
        }

        newProp.properties[title] = ofMember;
      } else if (ofMember.type === "object") {
        // If its of type object with no properties, we treat it as a string
        const title = ofMember.title ??
          `${prop.title}JSON`;

        newProp.properties[title] = {
          title,
          description: ofMember.description,
          type: "string",
        };
      } else {
        console.log(ofMember);
        throw new Error(
          `attempted to process oneOf as not an object or array: ${ofMember}`,
        );
      }
    }

    return newProp;
  }

  if (prop.anyOf) {
    let isObject;
    const properties = {} as Record<string, CfProperty>;

    for (const ofMember of prop.anyOf) {
      if (!isCfObjectProperty(ofMember)) {
        isObject = false;
        break;
      }
      isObject = true;

      if (!ofMember.title) {
        console.log(prop);
        throw new Error("anyOf of objects without title");
      }

      if (ofMember.properties) {
        isObject = true;

        properties[ofMember.title] = {
          ...ofMember.properties[ofMember.title],
        };
      } else if (ofMember.patternProperties) {
        isObject = true;

        if (!ofMember.title) {
          console.log(prop);
          throw new Error("anyOf of objects without title");
        }

        properties[ofMember.title] = ofMember;
      }
    }

    if (isObject) {
      return {
        description: prop.description,
        type: "object",
        properties,
      };
    } else {
      return {
        description: prop.description,
        type: "string",
      };
    }
  }

  return prop;
}

// Tells whether this can be treated like an object (even if it doesn't have type = object)
function isCfObjectProperty(prop: CfProperty): prop is CfObjectProperty {
  return prop.type === "object" || "properties" in prop ||
    "patternProperties" in prop;
}

export type CfSchema = Extend<CfObjectProperty, {
  typeName: string;
  description: string;
  primaryIdentifier: JSONPointer[];
  sourceUrl?: string;
  documentationUrl?: string;
  replacementStrategy?: "create_then_delete" | "delete_then_create";
  taggable?: boolean;
  tagging?: {
    taggable: boolean;
    tagOnCreate?: boolean;
    tagUpdatable?: boolean;
    cloudFormationSystemTags?: boolean;
    tagProperty?: string;
  };
  handlers?: {
    create: string[];
    read: string[];
    update: string[];
    delete: string[];
    list: string[];
  };
  remote?: Record<
    string,
    {
      "$comment": string;
      properties: JSONSchema.Interface["properties"];
      definitions: JSONSchema.Interface["definitions"];
    }
  >;
  definitions?: Record<string, CfProperty>;
  properties: Record<string, CfProperty>;
  readOnlyProperties?: JSONPointer[];
  writeOnlyProperties?: JSONPointer[];
  conditionalCreateOnlyProperties?: JSONPointer[];
  nonPublicProperties?: JSONPointer[];
  nonPublicDefinitions?: JSONPointer[];
  createOnlyProperties?: JSONPointer[];
  deprecatedProperties?: JSONPointer[];
  additionalIdentifiers?: JSONPointer[];
  resourceLink?: {
    "$comment": JSONSchema.Interface["$comment"];
    templateUri: string;
    mappings: Record<string, JSONPointer>;
  };
  propertyTransform?: Record<string, string>;
}>;

type CfDb = Record<string, CfSchema>;
const DB: CfDb = {};
const DEFAULT_PATH = "./cloudformation-schema";

export async function loadCfDatabase(
  { path, services }: {
    path?: string;
    services?: string[];
  },
): Promise<CfDb> {
  path ??= DEFAULT_PATH;
  if (Object.keys(DB).length === 0) {
    const fullPath = Deno.realPathSync(path);
    logger.debug("Loading database from Cloudformation schema", { fullPath });
    for (const dirEntry of Deno.readDirSync(fullPath)) {
      if (
        dirEntry.name.startsWith(".") ||
        dirEntry.name.indexOf("definition.schema") !== -1
      ) {
        continue;
      }

      const filename = `${fullPath}/${dirEntry.name}`;

      const rawData = await import(filename, {
        with: { type: "json" },
      });
      const data = rawData.default as CfSchema;

      logger.verbose("Loading schema", { name: dirEntry.name, data });

      const typeName: string = data.typeName;

      if (services && !services.some((service) => typeName.match(service))) {
        continue;
      }

      logger.debug(`Loaded ${typeName}`);
      try {
        const expandedSchema = await $RefParser.dereference(data, {
          dereference: {
            circular: "ignore",
            onDereference: (path: string, ref: JSONSchema.Object) => {
              const name = path.split("/").pop();
              ref.title = ref.title ?? name;
            },
          },
        }) as CfSchema;
        DB[typeName] = expandedSchema;
      } catch (e) {
        logger.error(`failed to expand ${typeName}`, e);
        DB[typeName] = data;
      }
    }
  }

  return DB;
}

export function getServiceByName(serviceName: string): CfSchema {
  const result = DB[serviceName];
  if (result) {
    return result;
  } else {
    throw new ServiceMissing(serviceName);
  }
}

export function getPropertiesForService(
  serviceName: string,
): CfSchema["properties"] {
  const service = getServiceByName(serviceName);
  return service.properties;
}

export function* allCfProps(root: CfProperty) {
  const queue = [{ cfProp: root, cfPropPath: "" }];
  while (queue.length > 0) {
    const prop = queue.shift()!;
    yield prop;
    const { cfProp, cfPropPath } = prop;
    if ("properties" in cfProp && cfProp.properties) {
      queue.push(
        ...Object.entries(cfProp.properties).map(([name, child]) => ({
          cfProp: child,
          cfPropPath: `${cfPropPath}/${name}`,
        })),
      );
    }
    if ("patternProperties" in cfProp && cfProp.patternProperties) {
      queue.push(
        ...Object.values(cfProp.patternProperties).map((child) => ({
          cfProp: child,
          cfPropPath,
        })),
      );
    }
    if ("typeProp" in cfProp && cfProp.typeProp) {
      queue.push({ cfProp: cfProp.typeProp, cfPropPath });
    }
  }
}
