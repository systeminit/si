import {
  type JSONSchema,
} from "https://deno.land/x/json_schema_typed@v8.0.0/draft_07.ts";
import $RefParser from "npm:@apidevtools/json-schema-ref-parser";
import _logger from "./logger.ts";
import { ServiceMissing } from "./errors.ts";
import { JSONSchemaObject } from "@apidevtools/json-schema-ref-parser";
import _ from "npm:lodash";

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
  | ["string", "array"]
  | ["integer", "string"];

export type CfProperty =
  & ({
    "type":
      | "boolean"
      | StringPair;
  } | {
    "type": "string";
    "enum"?: string[];
  } | {
    "type": "number" | "integer";
    "enum"?: number[];
  } | {
    "type": "array";
    "items": CfProperty;
  } | {
    "type": "object";
    "properties"?: Record<string, CfProperty>;
    "patternProperties"?: Record<string, CfProperty>;
  } | {
    "type": undefined;
    "oneOf"?: CfProperty[]; // TODO: this should be a qualification
    "anyOf"?: CfProperty[]; // TODO: this should be a qualification
    "properties"?: Record<string, CfProperty>;
    "patternProperties"?: Record<string, CfProperty>;
    "$ref"?: string;
  })
  & CfPropertyStatic;

export function normalizePropertyType(prop: CfProperty): CfProperty {
  if (!prop.type && (prop.properties || prop.patternProperties)) {
    return { ...prop, type: "object" };
  }

  if (!prop.type || !Array.isArray(prop.type)) {
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
      // If it's an object we make it a string so the user can pass JSON to it
      return { ...prop, type: "string" };
    case "array": {
      // When we get something that is string/array, the items object should already there
      const finalProp = { ...prop, type: "array" } as Extract<CfProperty, {
        type: "array";
      }>;
      if (!finalProp.items) {
        throw new Error("array typed prop includes array but has no items");
      }

      return finalProp;
    }
    default:
      console.log(prop);
      throw new Error("unhandled array type");
  }
}

export function normalizeAnyOfAndOneOfTypes(prop: CfProperty): CfProperty {
  if (prop.type) {
    return prop;
  }

  if (prop.oneOf) {
    const newProp: Extract<CfProperty, { type: "object" }> = {
      description: prop.description,
      type: "object",
      properties: {},
    };

    for (const ofMember of prop.oneOf) {
      if (!newProp.properties) {
        throw new Error("unexpected oneOf");
      }

      if (
        (ofMember.type === undefined || ofMember.type === "object") &&
        ofMember.properties
      ) {
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
      if (!ofMember.type && ofMember.properties) {
        isObject = true;

        if (!ofMember.title) {
          console.log(prop);
          throw new Error("anyOf of objects without title");
        }

        properties[ofMember.title] = {
          ...ofMember.properties[ofMember.title],
        };
      } else if (
        ofMember.type === "object" && ofMember.patternProperties
      ) {
        isObject = true;

        if (!ofMember.title) {
          console.log(prop);
          throw new Error("anyOf of objects without title");
        }

        properties[ofMember.title] = ofMember;
      } else {
        isObject = false;
        break;
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
        // false &&
        ![
          "AWS::EC2::Subnet",
          "AWS::EC2::SecurityGroup",
          "AWS::ElasticLoadBalancingV2::LoadBalancer",
          "AWS::ECS::Service",
          "AWS::EC2::SecurityGroupIngress",
          "AWS::EC2::SecurityGroupEgress",
          "AWS::EC2::SecurityGroupVpcAssociation",
          "AWS::EC2::Instance",
          "AWS::EC2::KeyPair",
          "AWS::EC2::VPC",
          "AWS::EC2::Subnet",
          "AWS::EC2::Route",
          "AWS::EC2::RouteTable",
          "AWS::EC2::SubnetRouteTableAssociation",
          "AWS::EC2::NatGateway",
          "AWS::EC2::InternetGateway",
          "AWS::EC2::EIP",
          "AWS::EC2::EIPAssociation",
          "AWS::EC2::VPCGatewayAttachment",
          "AWS::ElasticLoadBalancingV2::Listener",
          "AWS::ElasticLoadBalancingV2::ListenerRule",
          "AWS::ElasticLoadBalancingV2::TargetGroup",
          "AWS::ECS::CapacityProvider",
          "AWS::ECS::Cluster",
          "AWS::ECS::ClusterCapacityProviderAssociations",
          "AWS::ECS::TaskDefinition",
          "AWS::IAM::Policy",
          "AWS::IAM::Role",
          "AWS::IAM::InstanceProfile",
          "AWS::IAM::RolePolicy",
          "AWS::IAM::ManagedPolicy",
        ].includes(typeName)
      ) continue;

      logger.debug(`Loaded ${typeName}`);
      try {
        const expandedSchema = await $RefParser.dereference(data, {
          dereference: {
            circular: "ignore",
            onDereference: (path: string, ref: JSONSchemaObject) => {
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
