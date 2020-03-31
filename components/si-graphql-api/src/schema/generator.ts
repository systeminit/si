import { Metadata } from "grpc";
import * as protobuf from "protobufjs";
import { ServiceDescription } from "@/services";
import { ProtobufLoader } from "@/protobuf";
import {
  GraphqlHintLoader,
  GraphqlHintMessage,
  GraphqlHintMethod,
} from "@/graphql-hint";
import {
  arg,
  objectType,
  enumType,
  inputObjectType,
  queryType,
  extendType,
  mutationType,
} from "nexus";
import { logger } from "@/logger";
import { loginTypes } from "@/schema/login";
import { subscriptionTypes } from "@/schema/entityEvent";
import { Context } from "@/.";
import { AuthenticationError } from "apollo-server";

export class SchemaGenerator {
  types: any[];
  services: ServiceDescription[];
  protoLoader: ProtobufLoader;
  graphqlHintLoader: GraphqlHintLoader;
  seenTypes: {
    [key: string]: boolean;
  };

  constructor(
    services: ServiceDescription[],
    protoLoader: ProtobufLoader,
    graphqlHintLoader: GraphqlHintLoader,
  ) {
    this.services = services;
    this.protoLoader = protoLoader;
    this.graphqlHintLoader = graphqlHintLoader;
    this.types = [];

    // We must have a query and a mutation type
    const query = queryType({
      definition(_t) {},
    });

    const mutation = mutationType({
      definition(_t) {},
    });

    this.types.push(query);
    this.types.push(mutation);

    // All the non-generated types get added here
    this.types = this.types.concat(loginTypes);
    this.types = this.types.concat(subscriptionTypes);

    this.seenTypes = {};
  }

  // Given a protobuf method or type, fetch the grpc service that
  // corresponds to it.
  grpcServiceName(protobufItem: protobuf.Method | protobuf.Type): string {
    const protobufFullName = protobufItem.fullName;
    return this.services.find(s =>
      protobufFullName.startsWith(`.${s.protoPackageName}`),
    ).grpcServiceName;
  }

  // Given a protobuf type, return the generator hints for it.
  graphqlHintMessage(protobufItem: protobuf.Type): GraphqlHintMessage {
    const protobufFullName = protobufItem.fullName;
    const hint = this.graphqlHintLoader.hints.find(h =>
      protobufFullName.startsWith(`.${h.protoPackageName}`),
    );
    if (hint && hint.message && hint.message[protobufItem.name]) {
      return hint.message[protobufItem.name];
    } else {
      // @ts-ignore - I do not know how to make this work
      return {};
    }
  }

  graphqlHintMethod(
    serviceName: string,
    protobufItem: protobuf.Method,
  ): GraphqlHintMethod {
    const protobufFullName = protobufItem.fullName;
    const hint = this.graphqlHintLoader.hints.find(h =>
      protobufFullName.startsWith(`.${h.protoPackageName}`),
    );
    if (
      hint &&
      hint.service &&
      hint.service[serviceName] &&
      hint.service[serviceName][protobufItem.name]
    ) {
      return hint.service[serviceName][protobufItem.name];
    } else {
      return {};
    }
  }

  // Given a protobuf item, figure out what its real GraphQL Type Name
  // should be. We prefix the entity/component domains.
  graphqlTypeName(
    protobufItem: protobuf.Type | protobuf.Enum | protobuf.Method,
    protobufName?: string,
  ): string {
    protobufName = protobufName || protobufItem.name;

    let isReferenceType = false;

    let protobufFullName = protobufItem.fullName;
    // If it starts with si, then the full name is the name of the item - it's
    // a foreign reference, essentially.
    if (protobufName.startsWith("si.")) {
      isReferenceType = true;
      protobufFullName = `.${protobufName}`;
    }

    // The protobuf full name is `si.account.User`, for example
    //
    // So find the service that uses that same prefix, and then
    // return the graphqlTypePrefix for that service.
    const graphqlTypePrefixResult = this.services.find(s =>
      protobufFullName.startsWith(`.${s.protoPackageName}`),
    );
    logger.log("debug", "the prefix result is :", {
      graphqlTypePrefixResult,
      protobufItem,
      protobufName,
      protobufFullName,
    });
    const graphqlTypePrefix = graphqlTypePrefixResult.graphqlTypePrefix;

    logger.log("debug", "the prefix is :", { graphqlTypePrefix });

    logger.log("debug", "we have: ", {
      protobufItem,
      protobufName,
      protobufFullName,
    });

    if (isReferenceType) {
      const realName = protobufName.split(".").pop();
      return `${graphqlTypePrefix}${realName}`;
    } else {
      return `${graphqlTypePrefix}${protobufName}`;
    }
  }

  // Define all the fields for a top level type. May be either
  // an Input or Object type in GraphQL - makes no difference to
  // this function.
  protobufToType(protobufType: protobuf.Type, t: any): void {
    // First - grab all the generator hints for this type.
    const hints = this.graphqlHintMessage(protobufType);

    // If we implement an interface, go ahead and do that now.
    if (hints["implementsInterfaceTypeHint"]) {
      for (const implementsType of hints["implementsInterfaceTypeHint"]) {
        t.implements(implementsType);
      }
    }

    // Second, iterate over every field, and generate its definition.
    for (const field in protobufType.fields) {
      const fieldData = protobufType.fields[field];

      // We either have hints for this field, or we just use an empty
      // object.
      const hint = hints[field] || {};

      // If we said we should skip this field - we skip the field, duh.
      if (hint["skip"]) {
        continue;
      }

      // If a comment was supplied, we add it here.
      const fieldConfig = {};
      if (fieldData.comment) {
        fieldConfig["description"] = fieldData.comment;
      } else {
        fieldConfig["description"] = `${field}`;
      }

      // If the field is "repeated" in the protobuf, then it
      // is a list type. Otherwise, we can use the passed in
      // type template.
      let gt: typeof t | typeof t.list;
      if (fieldData.repeated || fieldData.map) {
        gt = t.list;
      } else {
        gt = t;
      }
      // If it is an "id" field, then we should announce that
      // to GraphQL, so that client side caching can be smart.
      if (field == "id") {
        gt.id(field, fieldConfig);
      } else if (fieldData.map) {
        // NOTE: This only works with map<string, string> in protobuf.
        //
        // If we need to support more complex types, you'll need to
        // resolve the protobuf field
        const typeName = this.graphqlTypeName(protobufType);
        const mapFieldType = field.charAt(0).toUpperCase() + field.slice(1);
        const mapTypeName = `${typeName}${mapFieldType}Map`;
        gt.field(field, { type: mapTypeName, ...fieldConfig });
      } else if (
        // Protobuf floats or doubles live in GraphQL floats.
        fieldData.type == "float" ||
        fieldData.type == "double" ||
        fieldData.type == "DoubleValue"
      ) {
        gt.float(field, fieldConfig);
        // Protobuf integers map to GraphQL integers.
      } else if (
        fieldData.type == "int32" ||
        fieldData.type == "sint32" ||
        fieldData.type == "uint32" ||
        fieldData.type == "sint32" ||
        fieldData.type == "fixed32" ||
        fieldData.type == "sfixed32" ||
        fieldData.type == "google.protobuf.UInt32Value"
      ) {
        gt.int(field, fieldConfig);
      } else if (
        fieldData.type == "bool" ||
        fieldData.type == "google.protobuf.BoolValue"
      ) {
        gt.boolean(field, fieldConfig);
      } else if (
        fieldData.type == "string" ||
        fieldData.type == "int64" ||
        fieldData.type == "uint64" ||
        fieldData.type == "sint64" ||
        fieldData.type == "fixed64" ||
        fieldData.type == "sfixed64" ||
        fieldData.type == "bytes" ||
        fieldData.type == "google.protobuf.StringValue"
      ) {
        gt.string(field, fieldConfig);
      } else {
        // Otherwise, this field references another type. So we look
        // up the type name and use it.
        const referenceType = this.graphqlTypeName(
          protobufType,
          fieldData.type,
        );
        // @ts-ignore
        gt.field(field, { type: referenceType, ...fieldConfig });
      }

      // If the hints say this field has_one, then it should become
      // two fields - one the original field, and the second a field
      // that resolves to the grpc endpoint that retrieves the ID.
      //
      // The corresponding field must be identical to the field
      // in the request object.
      if (hint["has_one"]) {
        gt.field(hint["has_one"]["to"], {
          type: hint["has_one"]["type"],
          async resolve(obj, _attrs, { dataSources: { grpc }, user }: Context) {
            const metadata = new Metadata();
            metadata.add("authenticated", `${user.authenticated}`);
            metadata.add("userId", user["userId"] || "");
            metadata.add("billingAccountId", user["billingAccountId"] || "");

            const grpcServiceName = hint["has_one"]["grpcServiceName"];
            const g = grpc.service(grpcServiceName);
            const input = {};
            input[field] = obj[field];
            const req = new g.Request(
              hint["has_one"]["method"],
              input,
            ).withMetadata(metadata);
            const result = await req.exec();
            return result.response[hint["has_one"]["to"]];
          },
          ...fieldConfig,
        });
      }
      if (hint["in_list"]) {
        for (const thisHint of hint["in_list"]) {
          t.field(thisHint["to"], {
            type: thisHint["type"],
            // @ts-ignore - we know, this isn't strongly typed
            args: { input: arg({ type: thisHint["inputType"] }) },
            async resolve(
              obj,
              attrs,
              { dataSources: { grpc }, user }: Context,
            ) {
              const metadata = new Metadata();
              metadata.add("authenticated", `${user.authenticated}`);
              metadata.add("userId", user["userId"] || "");
              metadata.add("billingAccountId", user["billingAccountId"] || "");

              const grpcServiceName = thisHint["grpcServiceName"];
              const g = grpc.service(grpcServiceName);
              const expressionList = [];
              expressionList.push({
                expression: {
                  field: thisHint["listField"],
                  fieldType: "STRING",
                  comparison: "CONTAINS",
                  value: obj.id,
                },
              });
              const input = attrs["input"] || {};
              if (input["query"]) {
                const userQuery = input["query"];
                input["query"] = {
                  items: [
                    { query: userQuery },
                    {
                      query: {
                        items: expressionList,
                      },
                    },
                  ],
                  booleanTerm: "AND",
                };
              } else {
                input["query"] = {
                  items: expressionList,
                };
              }
              // I think this might not work forever, but it's going to work as
              // long as everything you are lookikng for is scoped by billing id! :)
              input["scopeByTenantId"] = obj["billing_account_id"];
              const req = new g.Request(thisHint["method"], input).withMetadata(
                metadata,
              );
              const result = await req.exec();
              logger.log("error", "**** I have the in list", {
                result: result,
                thisHint,
              });
              return result.response;
            },
            ...fieldConfig,
          });
        }
      }
      if (hint["has_list"]) {
        const thisHint = hint["has_list"];
        t.field(thisHint["to"], {
          type: thisHint["type"],
          // @ts-ignore - we know, this isn't strongly typed
          args: { input: arg({ type: thisHint["inputType"] }) },
          async resolve(obj, attrs, { dataSources: { grpc }, user }: Context) {
            const metadata = new Metadata();
            metadata.add("authenticated", `${user.authenticated}`);
            metadata.add("userId", user["userId"] || "");
            metadata.add("billingAccountId", user["billingAccountId"] || "");

            const grpcServiceName = thisHint["grpcServiceName"];
            const g = grpc.service(grpcServiceName);
            const expressionList = [];
            for (const itemId of obj[field]) {
              expressionList.push({
                expression: {
                  field: "id",
                  fieldType: "STRING",
                  comparison: "EQUALS",
                  value: itemId,
                },
              });
            }
            const input = attrs["input"] || {};
            if (input["query"]) {
              const userQuery = input["query"];
              input["query"] = {
                items: [
                  { query: userQuery },
                  {
                    query: {
                      items: expressionList,
                      booleanTerm: "OR",
                    },
                  },
                ],
                booleanTerm: "AND",
              };
            } else {
              input["query"] = {
                items: expressionList,
                booleanTerm: "OR",
              };
            }
            // I think this might not work forever, but it's going to work as
            // long as everything you are lookikng for is scoped by billing id! :)
            input["scopeByTenantId"] = obj["billing_account_id"];
            const req = new g.Request(thisHint["method"], input).withMetadata(
              metadata,
            );
            const result = await req.exec();
            logger.log("error", "**** I have the has list", {
              result: result,
              thisHint,
            });
            return result.response;
          },
          ...fieldConfig,
        });
      }
      if (hint["has_many"]) {
        for (const thisHint of hint["has_many"]) {
          t.field(thisHint["to"], {
            type: thisHint["type"],
            // @ts-ignore - we know, this isn't strongly typed
            args: { input: arg({ type: thisHint["inputType"] }) },
            async resolve(
              obj,
              attrs,
              { dataSources: { grpc }, user }: Context,
            ) {
              const metadata = new Metadata();
              metadata.add("authenticated", `${user.authenticated}`);
              metadata.add("userId", user["userId"] || "");
              metadata.add("billingAccountId", user["billingAccountId"] || "");

              const grpcServiceName = thisHint["grpcServiceName"];
              const g = grpc.service(grpcServiceName);
              const input = attrs["input"] || {};
              logger.log("error", "I have sad times", { attrs });
              logger.log("error", "I have good times", { obj });
              input["scopeByTenantId"] = obj["id"];
              const req = new g.Request(thisHint["method"], input).withMetadata(
                metadata,
              );
              const result = await req.exec();
              logger.log("error", "I have the has many", {
                result: result,
                thisHint,
              });
              return result.response;
            },
            ...fieldConfig,
          });
        }
      }
    }
  }

  // Translate a protobuf method to graphql. Takes the service name
  // and the method itself.
  protobufMethodToGraphql(
    serviceName: string,
    protobufItem: protobuf.Method,
  ): void {
    const nameAsType = this.graphqlTypeName(protobufItem);
    const methodName = nameAsType[0].toLowerCase() + nameAsType.slice(1);

    const responseType = this.graphqlTypeName(
      protobufItem,
      protobufItem.responseType,
    );
    const requestType = this.graphqlTypeName(
      protobufItem,
      protobufItem.requestType,
    );

    const grpcServiceName = this.grpcServiceName(protobufItem);
    const grpcMethodName = protobufItem.name;

    const hint = this.graphqlHintMethod(serviceName, protobufItem);

    // If this is a query, we extend the query type by resolving
    // this with an input object that maps to the protobuf request,
    // and the response.
    if (hint["query"]) {
      const graphqlType = extendType({
        type: "Query",
        definition(t) {
          t.field(methodName, {
            // @ts-ignore
            type: responseType,
            args: {
              // @ts-ignore
              input: arg({ type: requestType }),
            },
            async resolve(
              _root,
              { input },
              { dataSources: { grpc }, user }: Context,
            ) {
              if (user.authenticated == false && hint["skipauth"] != true) {
                throw new AuthenticationError("Must be logged in");
              }
              const metadata = new Metadata();
              metadata.add("authenticated", `${user.authenticated}`);
              metadata.add("userId", user["userId"] || "");
              metadata.add("billingAccountId", user["billingAccountId"] || "");
              const g = grpc.service(grpcServiceName);
              let req;
              if (input) {
                const grpcInput = transformInputMethod(input, protobufItem);
                req = new g.Request(grpcMethodName, grpcInput).withMetadata(
                  metadata,
                );
              } else {
                req = new g.Request(grpcMethodName, {}).withMetadata(metadata);
              }
              let result = await req.exec();
              result = transformOutputMethod(result.response, protobufItem);
              return result;
            },
          });
        },
      });
      this.seenTypes[methodName] = true;
      this.types.push(graphqlType);
    } else if (hint["mutation"]) {
      /// Mutations are identical for us
      const graphqlType = extendType({
        type: "Mutation",
        definition(t) {
          t.field(methodName, {
            // @ts-ignore
            type: responseType,
            args: {
              // @ts-ignore
              input: arg({ type: requestType }),
            },
            async resolve(
              _root,
              { input },
              { dataSources: { grpc }, user }: Context,
            ) {
              if (user.authenticated == false && hint["skipauth"] != true) {
                throw new AuthenticationError("Must be logged in");
              }
              const grpcInput = transformInputMethod(input, protobufItem);
              const metadata = new Metadata();
              metadata.add("authenticated", `${user.authenticated}`);
              metadata.add("userId", user["userId"] || "");
              metadata.add("billingAccountId", user["billingAccountId"] || "");
              const g = grpc.service(grpcServiceName);
              const req = new g.Request(grpcMethodName, grpcInput).withMetadata(
                metadata,
              );
              let result = await req.exec();
              result = transformOutputMethod(result.response, protobufItem);
              return result;
            },
          });
        },
      });
      this.seenTypes[methodName] = true;
      this.types.push(graphqlType);
    } else {
      // If it is not present in the list, then it shoudn't be
      // exposed.
      logger.log("debug", "Keeping method private", {
        serviceName,
        methodName,
      });
    }
  }

  protobufEnumToGraphql(protobufItem: protobuf.Enum): void {
    const name = this.graphqlTypeName(protobufItem);
    if (this.seenTypes[name] != true) {
      const graphqlType = enumType({
        name: name,
        members: protobufItem.values,
      });
      this.seenTypes[name] = true;
      this.types.push(graphqlType);
    }
  }

  // Given a protobuf type, translate it into a GraphQL type.
  protobufTypeToGraphql(protobufType: protobuf.Type): void {
    const name = this.graphqlTypeName(protobufType);

    // Make sure we only convert types one time. ;)
    if (!this.seenTypes[name]) {
      const hints = this.graphqlHintMessage(protobufType);
      if (hints["skip"]) {
        logger.log("debug", "Skipping type", { name });
        return;
      }

      // If this is a map type, we need to generate the map
      // object here. We can't do it down below, because then
      // we are in the definition, and the types won't be
      // available.
      for (const field in protobufType.fields) {
        const fieldData = protobufType.fields[field];
        if (fieldData.map) {
          const typeName = this.graphqlTypeName(protobufType);
          const mapFieldType = field.charAt(0).toUpperCase() + field.slice(1);
          const mapTypeName = `${typeName}${mapFieldType}Map`;
          if (name.endsWith("Request") || hints["inputType"]) {
            const mapType = inputObjectType({
              name: mapTypeName,
              definition(mapt) {
                mapt.string("key");
                mapt.string("value");
              },
            });
            this.seenTypes[mapTypeName] = true;
            this.types.push(mapType);
          } else {
            const mapType = objectType({
              name: mapTypeName,
              definition(mapt) {
                mapt.string("key");
                mapt.string("value");
              },
            });
            this.seenTypes[mapTypeName] = true;
            this.types.push(mapType);
          }
        }
      }

      // If the type ends with Request, it is a type defined
      // for a method request/response. All Request methods
      // need to become Graphql Input types.
      //
      // Either way, the definition for the field is delayed
      // until schema generation. So we delegate it to the
      // protobufToType method, which is actually going to
      // define our type.
      if (name.endsWith("Request") || hints["inputType"]) {
        const graphqlType = inputObjectType({
          //@ts-ignore
          name,
          definition: t => this.protobufToType(protobufType, t),
        });
        this.seenTypes[name] = true;
        this.types.push(graphqlType);
      } else {
        const graphqlType = objectType({
          name,
          definition: t => this.protobufToType(protobufType, t),
        });
        this.seenTypes[name] = true;
        this.types.push(graphqlType);
      }
    }
  }

  // Traverse all the loaded types, and generate their GraphQL
  // schema.
  traverseTypes(
    current?:
      | protobuf.Root
      | protobuf.Type
      | protobuf.Enum
      | protobuf.ReflectionObject,
  ): void {
    // If we do not have a current type, then we are at the root.
    // Resolve all the fields, and then load from there.
    //
    // But only in the `si` namespace. We don't want to load anybody
    // elses random stuff, except via reference.
    if (current === undefined) {
      this.protoLoader.root.resolve();
      const root = this.protoLoader.root;
      for (const nested of root.nestedArray) {
        if (nested.name == "si") {
          current = nested;
          break;
        }
      }
    }

    // If this is a protobuf Type, then translate that to a GraphQL Type
    if (current instanceof protobuf.Type) {
      logger.log("info", "Loading type", {
        name: current.name,
        current,
        typeFriend: typeof current,
      });
      this.protobufTypeToGraphql(current);
      // If this is a protobuf Enum, then translate that to a GraphQL Enum
    } else if (current instanceof protobuf.Enum) {
      logger.log("info", "Loading enum", { name: current.name });
      this.protobufEnumToGraphql(current);
      // If this is a protobuf Service, then translate all its methods
    } else if (current instanceof protobuf.Service) {
      logger.log("info", "Loading service", { name: current.name });
      for (const method of current.methodsArray) {
        logger.log("info", "Loading method", { name: method.name });
        this.protobufMethodToGraphql(current.name, method);
      }
    }

    // If there are nested items inside this one, convert them as well.
    if ("nestedArray" in current) {
      for (const nested of current.nestedArray) {
        this.traverseTypes(nested);
      }
    }
  }

  // Generate all the GraphQL types via nexus.
  generate(): void {
    // Walk all the types
    this.traverseTypes();
  }
}

export function transformOutputType(
  input: any,
  protobufType: protobuf.Type,
): any {
  protobufType.resolve();
  if (Array.isArray(input)) {
    for (const itemId in input) {
      for (const inputField in input[itemId]) {
        const inputFieldType = protobufType.fields[inputField];
        inputFieldType.resolve();
        input[itemId][inputField] = transformOutputField(
          input[itemId][inputField],
          inputFieldType,
        );
      }
    }
  } else {
    for (const inputField in input) {
      const inputFieldType = protobufType.fields[inputField];
      input[inputField] = transformOutputField(
        input[inputField],
        inputFieldType,
      );
    }
  }
  return input;
}

export function transformOutputField(
  inputValue: any,
  protobufField: protobuf.Field,
): any {
  const protobufScalars = [
    "double",
    "float",
    "int32",
    "int64",
    "uint32",
    "uint64",
    "sint32",
    "sint64",
    "fixed32",
    "fixed64",
    "sfixed32",
    "sfixed64",
    "bool",
    "string",
    "bytes",
  ];
  const googleWellKnownTypeWrappers = [
    "google.protobuf.DoubleValue",
    "google.protobuf.FloatValue",
    "google.protobuf.Int64Value",
    "google.protobuf.UInt64Value",
    "google.protobuf.Int32Value",
    "google.protobuf.UInt32Value",
    "google.protobuf.BoolValue",
    "google.protobuf.StringValue",
    "google.protobuf.BytesValue",
  ];
  if (protobufField.map) {
    const arrayMap = [];
    for (const [key, value] of Object.entries(inputValue)) {
      if (protobufScalars.includes(protobufField.type)) {
        arrayMap.push({ key: key, value: value });
      } else if (googleWellKnownTypeWrappers.includes(protobufField.type)) {
        arrayMap.push({ key: key, value: value["value"] });
      } else {
        protobufField.resolve();
        arrayMap.push({
          key: key,
          value: transformOutputType(
            value,
            protobufField.resolvedType as protobuf.Type,
          ),
        });
        //map[entry.key] = transformInputType(
        //  entry.value,
        //  protobufField.resolvedType as protobuf.Type,
        //);
      }
    }
    return arrayMap;
  } else if (protobufScalars.includes(protobufField.type)) {
    return inputValue;
  } else if (googleWellKnownTypeWrappers.includes(protobufField.type)) {
    if (inputValue === null) {
      return null;
    } else {
      return inputValue["value"];
    }
  } else {
    protobufField.resolve();
    return transformOutputType(
      inputValue,
      protobufField.resolvedType as protobuf.Type,
    );
  }
}

export function transformOutputMethod(
  input: any,
  protobufItem: protobuf.Method,
): any {
  protobufItem.resolve();
  const responseType = protobufItem.resolvedResponseType;
  return transformOutputType(input, responseType);
}

export function transformInputField(
  inputValue: any,
  protobufField: protobuf.Field,
): any {
  const protobufScalars = [
    "double",
    "float",
    "int32",
    "int64",
    "uint32",
    "uint64",
    "sint32",
    "sint64",
    "fixed32",
    "fixed64",
    "sfixed32",
    "sfixed64",
    "bool",
    "string",
    "bytes",
  ];
  const googleWellKnownTypeWrappers = [
    "google.protobuf.DoubleValue",
    "google.protobuf.FloatValue",
    "google.protobuf.Int64Value",
    "google.protobuf.UInt64Value",
    "google.protobuf.Int32Value",
    "google.protobuf.UInt32Value",
    "google.protobuf.BoolValue",
    "google.protobuf.StringValue",
    "google.protobuf.BytesValue",
  ];

  if (Array.isArray(inputValue) && protobufField.map) {
    const map = {};
    for (const entry of inputValue) {
      if (protobufScalars.includes(protobufField.type)) {
        map[entry.key] = entry.value;
      } else if (googleWellKnownTypeWrappers.includes(protobufField.type)) {
        map[entry.key] = { value: entry.value };
      } else {
        protobufField.resolve();
        map[entry.key] = transformInputType(
          entry.value,
          protobufField.resolvedType as protobuf.Type,
        );
      }
    }
    return map;
  } else if (protobufScalars.includes(protobufField.type)) {
    return inputValue;
  } else if (googleWellKnownTypeWrappers.includes(protobufField.type)) {
    return { value: inputValue };
  } else {
    protobufField.resolve();
    return transformInputType(
      inputValue,
      protobufField.resolvedType as protobuf.Type,
    );
  }
}

export function transformInputType(
  input: any,
  protobufType: protobuf.Type,
): any {
  protobufType.resolve();
  if (Array.isArray(input)) {
    for (const itemId in input) {
      for (const inputField in input[itemId]) {
        const inputFieldType = protobufType.fields[inputField];
        inputFieldType.resolve();
        input[itemId][inputField] = transformInputField(
          input[itemId][inputField],
          inputFieldType,
        );
      }
    }
  } else {
    for (const inputField in input) {
      const inputFieldType = protobufType.fields[inputField];
      input[inputField] = transformInputField(
        input[inputField],
        inputFieldType,
      );
    }
  }
  return input;
}

export function transformInputMethod(
  input: any,
  protobufItem: protobuf.Method,
): any {
  // Resolve the whole item, so we have all the fields and attributes
  //
  // Steps:
  //
  // 1) Get the request type from the method
  // 2) Look up the request type
  // 3) Walk the top level properties of the input, and look for fields that match the request type. If any of those fields are well known types or maps, convert them to the right structure.
  // 4) If we find a type, resolve that type.
  // 5) Return the results.
  protobufItem.resolve();
  const requestType = protobufItem.resolvedRequestType;
  return transformInputType(input, requestType);
}
