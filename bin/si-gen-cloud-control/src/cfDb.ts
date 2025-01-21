import { type JSONSchema } from "https://deno.land/x/json_schema_typed@v8.0.0/draft_07.ts";
import $RefParser from "npm:@apidevtools/json-schema-ref-parser";
import _logger from "./logger.ts";
import { ServiceMissing } from "./errors.ts";

const logger = _logger.ns("cfDb").seal();

type JSONPointer = string;

interface CfPropertyStatic {
  description?: string;
  title?: string;
}

type StringPair =
  | ["string", "object"]
  | ["string", "boolean"]
  | ["string", "number"]
  | ["string", "integer"]
  | ["object", "string"]
  | ["boolean", "string"]
  | ["number", "string"]
  | ["integer", "string"];

export type CfProperty =
  & ({
    "type":
      | "integer"
      | "number"
      | "boolean"
      | StringPair;
  } | {
    "type": "string";
    "enum"?: string[];
  } | {
    "type": "array";
    "items": CfProperty;
  } | {
    "type": "object";
    "properties"?: Record<string, CfProperty>;
    "patternProperties"?: Record<string, CfProperty>;
  } | {
    "type": undefined;
    "oneOf": CfProperty[]; // TODO: this should be a quialification
  })
  & CfPropertyStatic;

export function normalizePropertyType(prop: CfProperty): CfProperty {
  if (!Array.isArray(prop.type)) {
    return prop;
  }
  const nonStringType = prop.type.find((t) => t !== "string");

  switch (nonStringType) {
    case "boolean":
      return { ...prop, type: "boolean" };
    case "integer":
    case "number":
      return { ...prop, type: "integer" };
    case "object":
      return { ...prop, type: "string" };
    default:
      console.log(prop);
      throw new Error("unhandled array type");
  }
}

export function normalizeAnyOfAndOneOfTypes(prop: CfProperty): CfProperty {
  if (prop.type || !prop.oneOf) {
    return prop;
  }

  const newProp: CfProperty = {
    description: prop.description,
    properties: {} as Record<string, CfProperty>,
    type: "object",
  };

  for (const oneOf of prop.oneOf) {
    if (!oneOf.title || !newProp.properties) {
      throw new Error("unexpected oneOf");
    }
    newProp.properties[oneOf.title] = oneOf;
  }

  return newProp;
}

export interface CfSchema extends JSONSchema.Interface {
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
}

type CfDb = Record<string, CfSchema>;
const DB: CfDb = {};

export async function loadCfDatabase(
  path: string = "./cloudformation-schema",
): Promise<CfDb> {
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

      if (
        ![
          "AWS::EC2::Subnet",
          "AWS::EC2::VPC",
          "AWS::WAF::WebACL"
        ].includes(typeName)
      ) continue;

      logger.debug(`Loaded ${typeName}`);
      try {
        const expandedSchema = await $RefParser.dereference(data) as CfSchema;
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
