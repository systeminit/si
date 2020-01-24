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
    logger.log("info", "the prefix result is :", {
      graphqlTypePrefixResult,
      protobufItem,
      protobufName,
      protobufFullName,
    });
    const graphqlTypePrefix = graphqlTypePrefixResult.graphqlTypePrefix;

    logger.log("info", "the prefix is :", { graphqlTypePrefix });

    logger.log("info", "we have: ", {
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
      if (fieldData.repeated) {
        gt = t.list;
      } else {
        gt = t;
      }
      // If it is an "id" field, then we should announce that
      // to GraphQL, so that client side caching can be smart.
      if (field == "id") {
        gt.id(field, fieldConfig);

        // Protobuf floats or doubles live in GraphQL floats.
      } else if (fieldData.type == "float" || fieldData.type == "double") {
        gt.float(field, fieldConfig);
        // Protobuf integers map to GraphQL integers.
      } else if (
        fieldData.type == "int32" ||
        fieldData.type == "sint32" ||
        fieldData.type == "uint32" ||
        fieldData.type == "sint32" ||
        fieldData.type == "fixed32" ||
        fieldData.type == "sfixed32"
      ) {
        gt.int(field, fieldConfig);
      } else if (fieldData.type == "bool") {
        gt.boolean(field, fieldConfig);
      } else if (
        fieldData.type == "string" ||
        fieldData.type == "int64" ||
        fieldData.type == "uint64" ||
        fieldData.type == "sint64" ||
        fieldData.type == "fixed64" ||
        fieldData.type == "sfixed64"
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
            logger.log("warn", "I have the cheese", { result: result, hint });
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
                req = new g.Request(grpcMethodName, input).withMetadata(
                  metadata,
                );
              } else {
                req = new g.Request(grpcMethodName, {}).withMetadata(metadata);
              }
              const result = await req.exec();
              return result.response;
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
              const metadata = new Metadata();
              metadata.add("authenticated", `${user.authenticated}`);
              metadata.add("userId", user["userId"] || "");
              metadata.add("billingAccountId", user["billingAccountId"] || "");
              const g = grpc.service(grpcServiceName);
              const req = new g.Request(grpcMethodName, input).withMetadata(
                metadata,
              );
              const result = await req.exec();
              return result.response;
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
        logger.log("info", "Skipping type", { name });
        return;
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
    if (current === undefined) {
      this.protoLoader.root.resolve();
      current = this.protoLoader.root;
    }

    // If this is a protobuf Type, then translate that to a GraphQL Type
    if (current instanceof protobuf.Type) {
      logger.log("info", "Loading type", { name: current.name });
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
