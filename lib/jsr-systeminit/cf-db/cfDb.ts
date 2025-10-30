/**
 * Main module providing functionality to load and access CloudFormation resource types.
 *
 * This module defines the core types and functions for working with CloudFormation
 * schemas, including loading the database, accessing service definitions, and
 * normalizing property types.
 *
 * @module cfDb
 */

import util from "node:util";
import type {
  JSONSchema,
} from "./draft_07.ts";
import $RefParser from "npm:@apidevtools/json-schema-ref-parser@11.9.3";
import _logger from "./logger.ts";
import { ServiceMissing } from "./errors.ts";
import _ from "npm:lodash@4.17.21";
import rawCfSchema from "./cf-schema.json" with { type: "json" };
import { CfDb, CfObjectProperty, CfProperty, CfPropertyType, CfSchema, isCfPropertyType } from "../../../bin/clover/src/pipelines/types.ts";
import "./isArrayFix.ts"

export type { CfDb, CfObjectProperty, CfProperty, CfPropertyType, CfSchema };

const logger = _logger.ns("cfDb").seal();


/**
 * Normalizes a CloudFormation property to ensure consistent structure.
 *
 * This function standardizes the property type information and handles special
 * cases like anyOf and oneOf constructs to provide a uniform interface.
 *
 * @param prop The CloudFormation property to normalize
 * @returns The normalized property with consistent structure
 */
export function normalizeProperty(
  prop: JSONSchema,
): CfProperty {
  prop = normalizePropertyType(prop);
  prop = normalizeAnyOfAndOneOfTypes(prop);
  return prop as CfProperty;
}

/**
 * Normalizes property type information to ensure consistent representation.
 *
 * This internal function handles various type scenarios in CloudFormation properties:
 * - Properties with single types are returned unchanged
 * - Properties with no type are inferred based on their structure
 * - Properties with array types (multi-type) are normalized to a single appropriate type
 *
 * @param prop The CloudFormation property to normalize
 * @returns The property with normalized type information
 * @internal
 */
function normalizePropertyType(
  prop: JSONSchema,
) {
  if (typeof prop === "boolean") throw new Error("unexpected boolean type");

  // If it's already a single CfPropertyType, return it as-is
  if (isCfPropertyType(prop.type)) return prop;

  // If it's not a recognized type name, throw
  if (typeof prop.type === "string") throw new Error(`Unexpected prop type ${prop.type}`);
  if (typeof prop.type === "number") throw new Error(`Unexpected numeric prop type (${prop.type})`);

  // Infer type when there is none.
  if (prop.type === undefined) {
    // Some props have no type but we can duck type them to objects
    if (prop.properties || prop.patternProperties) return { ...prop, type: "object" };
    // TODO we really need to look inside the ref here rather than assuming string ...
    if (prop.$ref) return { ...prop, type: "string" };

    // If it's a multi-type thing, return it--we don't really handle these yet.
    return prop;
  }

  if (Array.isArray(prop.type)) {
    // If the cf type is an array, it's always string+something, and we use that something
    // to guess the best type we should use
    const nonStringType = prop.type.find((t) => t !== "string");
    // TODO we need to know whether there *was* a string type here; and we need to know if
    // there were more than one non-string type

    switch (nonStringType) {
      case "boolean":
      case "integer":
      case "number":
        return { ...prop, type: nonStringType };
      case "object":
        // If it's an object we make it a json type, which will become a string type + textArea widget
        return { ...prop, type: "json" };
      case "array": {
        // When we get something that is string/array, the items object should already there
        if (!("items" in prop)) {
          throw new Error("array typed prop includes array but has no items");
        }
        return { ...prop, type: "array" };
      }
      default:
        throw new Error("unhandled array type");
    }
  }

  // TODO handle other cases (looks like JSONSchema.Array)
  throw new Error("unexpected property type");
}

/**
 * Normalizes properties with anyOf and oneOf constructs.
 *
 * This internal function handles complex schema constructs like anyOf and oneOf
 * by converting them into a more uniform structure, typically an object with
 * properties.
 *
 * @param prop The CloudFormation property to normalize
 * @returns The property with normalized anyOf/oneOf structures
 * @internal
 */
function normalizeAnyOfAndOneOfTypes(
  prop: JSONSchema.Interface,
) {
  if (prop.type) return prop;

  if (prop.oneOf) {
    const mergedProp: (CfProperty & { type: "object" }) = {
      description: prop.description,
      type: "object",
      properties: {},
    };
    let jsonProp: (CfProperty & { type: "string" }) | undefined = undefined;
    let arrayProp: (CfProperty & { type: "array" }) | undefined = undefined;

    for (let ofMember of prop.oneOf) {
      if (!mergedProp.properties) {
        throw new Error(`unexpected oneOf: ${util.inspect(prop, { depth: 3 })}`);
      }

      ofMember = normalizePropertyType(ofMember);

      if (ofMember.type === "object" && ofMember.properties) {
        for (const title of _.keys(ofMember.properties)) {
          mergedProp.properties[title] = normalizeProperty(ofMember.properties[title]);
        }
      } else if (ofMember.type === "array" && ofMember.items) {
        const title = ofMember.title ?? prop.title;
        if (!title) { throw new Error(`oneOf array without title: ${util.inspect(prop, { depth: 3 })}`); }

        // we don't support this yet; throw an exception if it happens so we can decide
        if (Array.isArray(ofMember.items)) throw new Error("unexpected array as item type");

        arrayProp = {
          title,
          description: prop.description,
          type: "array",
          items: normalizeProperty(ofMember.items),
        };
      } else if (ofMember.type === "object") {
        // If its of type object with no properties, we treat it as a string
        const title = ofMember.title ?? prop.title;

        jsonProp = {
          title,
          description: prop.description,
          type: "string",
        }
      } else {
        throw new Error(
          `attempted to process oneOf as not an object or array: ${util.inspect(prop, { depth: 3 })}`,
        );
      }
    }

    // Array props take precedence over JSON props as well as explicit array props,
    // because we are assuming that props that can be either object or array are really just a
    // "one or many", i.e. T or T[]
    if (arrayProp) return arrayProp;
    // JSON prop is last resort, return the nicely typed one if there is one
    if (mergedProp.properties) return mergedProp;
    if (!jsonProp) throw new Error("Unexpected or empty oneOf");
    return jsonProp;
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
        throw new Error(`anyOf of objects without title: ${util.inspect(prop, { depth: 3 })}`);
      }

      if (ofMember.properties) {
        isObject = true;

        properties[ofMember.title] = {
          ...ofMember.properties[ofMember.title],
        };
      } else if (ofMember.patternProperties) {
        isObject = true;

        if (!ofMember.title) {
          throw new Error(`anyOf of objects without title: ${util.inspect(prop, { depth: 3 })}`);
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

/**
 * Type guard that determines if a property can be treated as an object property.
 *
 * This internal function checks whether a CloudFormation property has object-like
 * characteristics, even if it doesn't explicitly have type="object".
 *
 * @param prop The CloudFormation property to check
 * @returns True if the property can be treated as an object property
 * @internal
 */
// Tells whether this can be treated like an object (even if it doesn't have type = object)
function isCfObjectProperty(prop: JSONSchema): prop is CfObjectProperty {
  return typeof prop === "object" && (prop.type === "object" || "properties" in prop ||
    "patternProperties" in prop);
}

const DB: CfDb = {};

/**
 * Loads the CloudFormation database from schema files.
 *
 * This function reads CloudFormation schema files from disk, processes them,
 * and loads them into memory for use by other functions. It dereferences all
 * schema references to provide a fully resolved database.
 *
 * @param options Loading options
 * @param options.path Optional path to the directory containing schema files
 * @param options.services Optional array of service name patterns to filter schemas
 * @returns Promise resolving to the loaded CloudFormation database
 */
export async function loadCfDatabase(
  { services }: {
    services?: string[];
  },
): Promise<CfDb> {
  if (Object.keys(DB).length === 0) {
    for (const cfSchema of rawCfSchema) {
      const typeName: string = cfSchema.typeName;

      if (services && !services.some((service) => typeName.match(service))) {
        continue;
      }

      logger.debug(`Loaded ${typeName}`);

      // Mark all definition props with their enclosing name for doc link generation
      if (cfSchema.definitions) {
        for (const [defName, defProp] of Object.entries(cfSchema.definitions)) {
          // deno-lint-ignore no-explicit-any
          for (const cfProp of nestedCfProps(defProp as any)) {
            (cfProp as { defName?: string }).defName = defName;
          }
        }
      }

      // Dereference the schema
      const dereferencedSchema = await $RefParser.dereference(cfSchema, {
        dereference: {
          circular: "ignore",
          onDereference: (path: string, ref: JSONSchema.Object) => {
            const name = path.split("/").pop();
            ref.title = ref.title ?? name;
          },
        },
      }) as CfSchema;
      DB[typeName] = dereferencedSchema;
    }
  }

  return DB;
}

/**
 * Generator function that recursively yields all nested properties in a CloudFormation property.
 *
 * This internal function performs a depth-first traversal of a property, yielding
 * each property it encounters, including those in anyOf, oneOf, allOf constructs,
 * nested properties, patternProperties, and array items.
 *
 * @param prop The CloudFormation property to traverse
 * @yields Each nested property encountered during traversal
 * @internal
 */
function* nestedCfProps(prop: CfProperty): Generator<CfProperty> {
  yield prop;
  for (const p of prop.anyOf ?? []) yield* nestedCfProps(p as CfProperty);
  for (const p of prop.oneOf ?? []) yield* nestedCfProps(p as CfProperty);
  for (const p of prop.allOf ?? []) yield* nestedCfProps(p as CfProperty);
  if ("properties" in prop) {
    for (const p of Object.values(prop.properties ?? {})) {
      yield* nestedCfProps(p as CfProperty);
    }
  }
  if ("patternProperties" in prop) {
    for (const p of Object.values(prop.patternProperties ?? {})) {
      yield* nestedCfProps(p as CfProperty);
    }
  }
  if ("items" in prop) yield* nestedCfProps(prop.items as CfProperty);
}

/**
 * Gets a CloudFormation resource type schema by name.
 *
 * This function retrieves a specific CloudFormation resource type schema
 * from the loaded database. The database must be loaded first using loadCfDatabase().
 *
 * @param serviceName The full name of the CloudFormation resource type (e.g., "AWS::Lambda::Function")
 * @returns The CloudFormation schema for the requested service
 * @throws {ServiceMissing} If the requested service doesn't exist in the database
 */
export function getServiceByName(serviceName: string): CfSchema {
  const result = DB[serviceName];
  if (result) {
    return result;
  } else {
    throw new ServiceMissing(serviceName);
  }
}

/**
 * Gets all properties for a CloudFormation resource type.
 *
 * This function retrieves the properties object for a specific CloudFormation
 * resource type from the loaded database.
 *
 * @param serviceName The full name of the CloudFormation resource type (e.g., "AWS::Lambda::Function")
 * @returns A record of property names to property definitions
 * @throws {ServiceMissing} If the requested service doesn't exist in the database
 */
export function getPropertiesForService(
  serviceName: string,
): CfSchema["properties"] {
  const service = getServiceByName(serviceName);
  return service.properties;
}

/**
 * Generator function that traverses all properties in a CloudFormation schema.
 *
 * This function performs a breadth-first traversal of a CloudFormation property
 * tree, yielding each property along with its path. This is useful for processing
 * or analyzing the entire schema structure.
 *
 * @param root The root CloudFormation property to traverse
 * @yields An object containing the current property and its path
 */
export function* allCfProps(
  root: CfProperty,
): Generator<{ cfProp: CfProperty; cfPropPath: string }> {
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
