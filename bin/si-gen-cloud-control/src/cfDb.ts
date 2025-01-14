import { type JSONSchema } from "https://deno.land/x/json_schema_typed@v8.0.0/draft_07.ts";
import $RefParser from "npm:@apidevtools/json-schema-ref-parser";
import _logger from "./logger.ts";

const logger = _logger.ns("cfDb").seal();

type JSONPointer = string;

interface CfPropertyStatic {
  description?: string;
}

export type CfProperty =
  & ({
    "type": "integer" | "boolean" | "string" ;
  } | {
    "type": "array";
    "items": CfProperty;
  } | {
    "type": "object";
    "properties": Record<string, CfProperty>;
  })
  & CfPropertyStatic;

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
await loadDatabase();

export async function loadDatabase(): Promise<CfDb> {
  if (Object.keys(DB).length === 0) {
    const fullPath = Deno.realPathSync("./cloudformation-schema");
    logger.debug("Loading database from Cloudformation schema", { fullPath });
    for (const dirEntry of Deno.readDirSync(fullPath)) {
      const rawData = await import(`${fullPath}/${dirEntry.name}`, {
        with: { type: "json" },
      });
      const data = rawData.default as CfSchema;

      logger.verbose("Loading schema", { name: dirEntry.name, data });

      const typeName: string = data.typeName;

      if (typeName !== "AWS::IAM::Role") continue;

      logger.debug(`Loaded ${typeName}`);
      try {
        const expandedSchema = await $RefParser.dereference(data);
        DB[typeName] = expandedSchema as CfSchema;
      } catch (e) {
        logger.error(`failed to expand ${typeName}`, e);
        DB[typeName] = data;
      }
    }
  }

  return DB;
}

export class ServiceMissing extends Error {
  constructor(serviceName: string) {
    super(
      `Attempt to find schema for service ${serviceName}, but it does not exist`,
    );
    this.name = "SchemaMissing";
  }
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
