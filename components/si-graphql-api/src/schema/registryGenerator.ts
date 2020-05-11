import { Metadata } from "grpc";
import {
  registry,
  Props,
  PropMethod,
  PropAction,
  PropObject,
  PropLink,
  PropEnum,
  PropNumber,
  ObjectTypes,
} from "si-registry";
import {
  arg,
  objectType,
  enumType,
  inputObjectType,
  queryType,
  mutationType,
  queryField,
  mutationField,
} from "nexus";
import { logger } from "@/logger";
import {
  ObjectDefinitionBlock,
  InputDefinitionBlock,
  extendType,
} from "nexus/dist/core";
import { camelCase, pascalCase, constantCase } from "change-case";
import { Context } from "@/index";
import { AuthenticationError } from "apollo-server";
import traceApi from "@opentelemetry/api";

interface NexusBlockOptions {
  nexusTypeDef?: NexusTypeDefBlock;
  inputType: boolean;
}

type NexusTypeDefBlock =
  | ObjectDefinitionBlock<string>
  | InputDefinitionBlock<string>;

interface FieldConfig {
  description: string;
  nullable?: boolean;
}

export class SiRegistryGenerator {
  // eslint-disable-next-line
  //
  // This is any because it is any in nexus

  // eslint-disable-next-line
  types: any[];
  loopDetector: Record<string, true> = {};
  typesCache: Record<string, true> = {};

  constructor() {
    this.types = [
      // eslint-disable-next-line
      queryType({ definition(_t) {} }),
      // eslint-disable-next-line
      mutationType({ definition(_t) {} }),
    ];
  }

  // * Walk the components
  //  * Generate a type for each non-scalar item
  //    * Recurse against all nested things
  //  * Generate a Query or Mutation for each method
  //    * If generating a mutation, create the mapping input types
  generate(): void {
    logger.debug("*** Starting GraphQL API Generation ***");
    for (const systemObject of registry.objects) {
      logger.debug(
        `*** Generating GraphQL Types for ${systemObject.typeName} ***`,
      );
      this.generateComponent(systemObject);
      logger.debug(
        `*** Generating GraphQL Queries and Mutations for ${systemObject.typeName} ***`,
      );
      this.generateMethods(systemObject);
      logger.debug(
        `*** Generating Associations for ${systemObject.typeName} ***`,
      );
      this.generateAssociations(systemObject);
    }
  }

  generateAssociations(systemObject: ObjectTypes): void {
    if (systemObject.associations.all().length == 0) {
      return;
    }
    // eslint-disable-next-line
    const thisGenerator = this;
    const associationTypeName = `${pascalCase(
      systemObject.typeName,
    )}Associations`;
    if (!this.typesCache[associationTypeName]) {
      const assocType = objectType({
        name: associationTypeName,
        description: `${systemObject.displayTypeName} Associations`,
        definition(t) {
          for (const association of systemObject.associations.all()) {
            const returnType = registry.get(association.typeName);
            if (returnType == undefined) {
              throw "No return type defined for this association; bug!";
            }
            const associatedObject = registry.get(association.typeName);
            if (associatedObject == undefined) {
              throw "No associated object defined for this association; bug!";
            }
            const associatedProp = associatedObject.methods.attrs.find(m => {
              return m.name == association.methodName;
            });
            if (!associatedProp) {
              throw `Cannot find associated method for ${systemObject.typeName} assocation ${association.fieldName}`;
            }

            if (association.kind() == "belongsTo") {
              const methodName = `${pascalCase(
                association.typeName,
              )}${pascalCase(association.methodName)}`;

              // This is a map to the "get" method
              t.field(association.fieldName, {
                // @ts-ignore
                type: `${methodName}Reply`,
                description: returnType.displayTypeName,
                async resolve(_root, _input, context: any) {
                  const trace = traceApi.trace.getTracer("si-graphql-api");
                  const span = trace.startSpan(
                    `graphql.belongsTo ${association.fieldName}`,
                    { parent: trace.getCurrentSpan() },
                  );
                  span.setAttribute("graphql.resolver", true);
                  span.setAttribute("graphql.root", false);
                  span.setAttribute("graphql.mutation", false);
                  span.setAttribute("graphql.query", false);
                  span.setAttribute("graphql.association", true);
                  span.setAttribute("graphql.association.type", "belongsTo");
                  span.setAttribute("graphql.fieldName", association.fieldName);

                  const grpc = context.dataSources.grpc;
                  const user = context.user;
                  const associationParent = context.associationParent;
                  if (!associationParent) {
                    throw "Cannot load associations without a parent; bug in the root field resolver";
                  }
                  if (user.authenticated == false) {
                    throw new AuthenticationError("Must be logged in");
                  }
                  const metadata = new Metadata();
                  metadata.add("authenticated", `${user.authenticated}`);
                  metadata.add("userId", user["userId"] || "");
                  metadata.add(
                    "billingAccountId",
                    user["billingAccountId"] || "",
                  );
                  const g = grpc.service(systemObject.siPathName);

                  // Get the field from the associationParent
                  let lookupValue = associationParent;
                  for (const key of association.fromFieldPath) {
                    if (lookupValue[key] == undefined) {
                      throw `Cannot find field ${association.fromFieldPath.join(
                        ".",
                      )} on association lookup`;
                    }
                    lookupValue = lookupValue[key];
                  }

                  // eslint-disable-next-line
                  const methodName = `${pascalCase(
                    association.typeName,
                  )}${pascalCase(association.methodName)}`;

                  const reqInput: Record<string, any> = {};
                  reqInput[association.methodArgumentName] = {
                    value: lookupValue,
                  };

                  const req = new g.Request(methodName, reqInput)
                    .withMetadata(metadata)
                    .withRetry(0);

                  let result = await req.exec();
                  result = thisGenerator.transformGrpcToGraphql(
                    result.response,
                    associatedProp as PropMethod,
                    systemObject,
                  );

                  span.end();
                  return result;
                },
              });
            } else if (association.kind() == "hasMany") {
              const methodName = `${pascalCase(
                association.typeName,
              )}${pascalCase(association.methodName)}`;

              // This is a map to the list method
              t.field(association.fieldName, {
                // @ts-ignore
                type: `${methodName}Reply`,
                // @ts-ignore
                args: { input: arg({ type: `${methodName}Request` }) },
                description: returnType.displayTypeName,
                async resolve(_root, input, context: any) {
                  const trace = traceApi.trace.getTracer("si-graphql-api");
                  const span = trace.startSpan(
                    `graphql.hasMany ${association.fieldName}`,
                    { parent: trace.getCurrentSpan() },
                  );
                  span.setAttribute("graphql.resolver", true);
                  span.setAttribute("graphql.root", false);
                  span.setAttribute("graphql.mutation", false);
                  span.setAttribute("graphql.query", false);
                  span.setAttribute("graphql.association", true);
                  span.setAttribute("graphql.association.type", "hasMany");
                  span.setAttribute("graphql.fieldName", association.fieldName);

                  const grpc = context.dataSources.grpc;
                  const user = context.user;
                  const associationParent = context.associationParent;
                  if (!associationParent) {
                    throw "Cannot load associations without a parent; bug in the root field resolver";
                  }
                  if (user.authenticated == false) {
                    throw new AuthenticationError("Must be logged in");
                  }
                  const metadata = new Metadata();
                  metadata.add("authenticated", `${user.authenticated}`);
                  metadata.add("userId", user["userId"] || "");
                  metadata.add(
                    "billingAccountId",
                    user["billingAccountId"] || "",
                  );
                  const g = grpc.service(systemObject.siPathName);

                  // Get the field from the associationParent
                  let lookupValue = associationParent;
                  for (const key of association.fromFieldPath) {
                    if (lookupValue[key] == undefined) {
                      throw `Cannot find field ${association.fromFieldPath.join(
                        ".",
                      )} on association lookup`;
                    }
                    lookupValue = lookupValue[key];
                  }
                  input["scopeByTenantId"] = lookupValue;

                  const reqInput = thisGenerator.transformGraphqlToGrpc(
                    input,
                    associatedProp as PropMethod,
                    systemObject,
                  );

                  const req = new g.Request(methodName, reqInput)
                    .withMetadata(metadata)
                    .withRetry(0);

                  let result = await req.exec();
                  result = thisGenerator.transformGrpcToGraphql(
                    result.response,
                    associatedProp as PropMethod,
                    systemObject,
                  );

                  span.end();

                  return result;
                },
              });
            } else if (association.kind() == "hasList") {
              const methodName = `${pascalCase(
                association.typeName,
              )}${pascalCase(association.methodName)}`;

              // This is a map to the list method
              t.field(association.fieldName, {
                // @ts-ignore
                type: `${methodName}Reply`,
                // @ts-ignore
                args: { input: arg({ type: `${methodName}Request` }) },
                description: returnType.displayTypeName,
                async resolve(_root, input, context: any) {
                  const trace = traceApi.trace.getTracer("si-graphql-api");
                  const span = trace.startSpan(
                    `graphql.hasList ${association.fieldName}`,
                    { parent: trace.getCurrentSpan() },
                  );
                  span.setAttribute("graphql.resolver", true);
                  span.setAttribute("graphql.root", false);
                  span.setAttribute("graphql.mutation", false);
                  span.setAttribute("graphql.query", false);
                  span.setAttribute("graphql.association", true);
                  span.setAttribute("graphql.association.type", "hasList");
                  span.setAttribute("graphql.fieldName", association.fieldName);

                  const grpc = context.dataSources.grpc;
                  const user = context.user;
                  const associationParent = context.associationParent;
                  if (!associationParent) {
                    throw "Cannot load associations without a parent; bug in the root field resolver";
                  }
                  if (user.authenticated == false) {
                    throw new AuthenticationError("Must be logged in");
                  }
                  const metadata = new Metadata();
                  metadata.add("authenticated", `${user.authenticated}`);
                  metadata.add("userId", user["userId"] || "");
                  metadata.add(
                    "billingAccountId",
                    user["billingAccountId"] || "",
                  );
                  const g = grpc.service(systemObject.siPathName);

                  // Get the field from the associationParent
                  input["scopeByTenantId"] =
                    associationParent["siProperties"]["billingAccountId"];
                  let lookupValue = associationParent;
                  for (const key of association.fromFieldPath) {
                    if (lookupValue[key] == undefined) {
                      throw `Cannot find field ${association.fromFieldPath.join(
                        ".",
                      )} on association lookup`;
                    }
                    lookupValue = lookupValue[key];
                  }
                  if (!Array.isArray(lookupValue)) {
                    throw "Failed to lookup an array value during HasList association - this is a bug!";
                  }
                  const queryData: Record<string, any> = {
                    booleanTerm: "OR",
                    items: [],
                  };
                  for (const id of lookupValue) {
                    queryData.items.push({
                      expression: {
                        field: "id",
                        comparison: "EQUALS",
                        value: id,
                        fieldType: "STRING",
                      },
                    });
                  }
                  input["query"] = queryData;

                  const reqInput = thisGenerator.transformGraphqlToGrpc(
                    input,
                    associatedProp as PropMethod,
                    systemObject,
                  );

                  const req = new g.Request(methodName, reqInput)
                    .withMetadata(metadata)
                    .withRetry(0);

                  let result = await req.exec();
                  result = thisGenerator.transformGrpcToGraphql(
                    result.response,
                    associatedProp as PropMethod,
                    systemObject,
                  );

                  span.end();

                  return result;
                },
              });
            } else if (association.kind() == "inList") {
              const methodName = `${pascalCase(
                association.typeName,
              )}${pascalCase(association.methodName)}`;

              // This is a map to the list method
              t.field(association.fieldName, {
                // @ts-ignore
                type: `${methodName}Reply`,
                // @ts-ignore
                args: { input: arg({ type: `${methodName}Request` }) },
                description: returnType.displayTypeName,
                async resolve(_root, input, context: any) {
                  const trace = traceApi.trace.getTracer("si-graphql-api");
                  const span = trace.startSpan(
                    `graphql.inList ${association.fieldName}`,
                    { parent: trace.getCurrentSpan() },
                  );
                  span.setAttribute("graphql.resolver", true);
                  span.setAttribute("graphql.root", false);
                  span.setAttribute("graphql.mutation", false);
                  span.setAttribute("graphql.query", false);
                  span.setAttribute("graphql.association", true);
                  span.setAttribute("graphql.association.type", "hasList");
                  span.setAttribute("graphql.fieldName", association.fieldName);

                  const grpc = context.dataSources.grpc;
                  const user = context.user;
                  const associationParent = context.associationParent;
                  if (!associationParent) {
                    throw "Cannot load associations without a parent; bug in the root field resolver";
                  }
                  if (user.authenticated == false) {
                    throw new AuthenticationError("Must be logged in");
                  }
                  const metadata = new Metadata();
                  metadata.add("authenticated", `${user.authenticated}`);
                  metadata.add("userId", user["userId"] || "");
                  metadata.add(
                    "billingAccountId",
                    user["billingAccountId"] || "",
                  );
                  const g = grpc.service(systemObject.siPathName);

                  // Get the field from the associationParent
                  input["scopeByTenantId"] =
                    associationParent["siProperties"]["billingAccountId"];
                  let lookupValue = associationParent;
                  for (const key of association.fromFieldPath) {
                    if (lookupValue[key] == undefined) {
                      throw `Cannot find field ${association.fromFieldPath.join(
                        ".",
                      )} on association lookup`;
                    }
                    lookupValue = lookupValue[key];
                  }
                  const queryData: Record<string, any> = {
                    booleanTerm: "OR",
                    items: [],
                  };
                  queryData.items.push({
                    expression: {
                      // @ts-ignore
                      field: association.toFieldPath.join("."),
                      comparison: "CONTAINS",
                      value: lookupValue,
                      fieldType: "STRING",
                    },
                  });
                  input["query"] = queryData;

                  const reqInput = thisGenerator.transformGraphqlToGrpc(
                    input,
                    associatedProp as PropMethod,
                    systemObject,
                  );

                  const req = new g.Request(methodName, reqInput)
                    .withMetadata(metadata)
                    .withRetry(0);

                  let result = await req.exec();
                  result = thisGenerator.transformGrpcToGraphql(
                    result.response,
                    associatedProp as PropMethod,
                    systemObject,
                  );

                  span.end();

                  return result;
                },
              });
            }
          }
        },
      });
      const extendRealType = extendType({
        // @ts-ignore
        type: pascalCase(systemObject.typeName),
        definition(t) {
          t.field("associations", {
            // @ts-ignore
            type: associationTypeName,
            async resolve(root, _input, context: any) {
              const trace = traceApi.trace.getTracer("si-graphql-api");
              const span = trace.startSpan(
                `graphql.associations ${camelCase(systemObject.typeName)}`,
                { parent: trace.getCurrentSpan() },
              );
              span.setAttribute("graphql.resolver", true);
              span.setAttribute("graphql.root", false);
              span.setAttribute("graphql.mutation", false);
              span.setAttribute("graphql.query", false);
              span.setAttribute("graphql.association", false);
              span.setAttribute("graphql.fieldName", "associations");

              context.associationParent = root;

              span.end();
              return {};
            },
          });
        },
      });
      this.typesCache[associationTypeName] = true;
      this.typesCache[extendRealType] = true;
      this.types.push(assocType);
      this.types.push(extendRealType);
    }
  }

  generateMethods(component: ObjectTypes): void {
    for (const method of component.methods.attrs) {
      if (method instanceof PropMethod || method instanceof PropAction) {
        this.generateQueryOrMutationField(method, component, method.mutation);
      }
    }
  }

  generateInputObjectTypes(prop: PropObject | PropLink): void {
    // eslint-disable-next-line
    let iteratorField: any;

    if (prop.kind() == "object") {
      // @ts-ignore
      iteratorField = prop.properties.attrs;
      this.propAsType(prop, { inputType: true });
    } else if (prop.kind() == "link") {
      // @ts-ignore
      const realprop = prop.lookupMyself();
      if (realprop.kind() == "object") {
        this.propAsType(realprop, { inputType: true });
        iteratorField = realprop.properties.attrs;
      } else {
        return;
      }
    } else if (prop.kind() == "map") {
      this.propAsType(prop, { inputType: true });
      return;
    } else {
      return;
    }

    for (const cp of iteratorField) {
      if (cp.kind() == "object" || cp.kind() == "link") {
        if (!this.typesCache[this.graphqlTypeName(cp, true)]) {
          this.typesCache[this.graphqlTypeName(cp, true)] = true;
          this.generateInputObjectTypes(cp);
        } else {
          this.typesCache[this.graphqlTypeName(cp, true)];
        }
      }
    }
  }

  graphqlTypeName(prop: Props, inputType?: boolean): string {
    let request = "";
    if (inputType) {
      request = "Request";
    }
    return `${pascalCase(prop.parentName)}${pascalCase(prop.name)}${request}`;
  }

  graphqlFieldName(prop: Props): string {
    return `${camelCase(prop.name)}`;
  }

  generateInputTypes(
    prop: PropMethod | PropAction,
    _component: ObjectTypes,
  ): void {
    for (const rp of prop.request.properties.attrs) {
      if (rp.kind() == "object" || rp.kind() == "link") {
        // @ts-ignore
        this.generateInputObjectTypes(rp);
      }
    }
  }

  generateQueryOrMutationField(
    prop: PropMethod | PropAction,
    component: ObjectTypes,
    mutation = false,
  ): void {
    // eslint-disable-next-line
    const thisGenerator = this;

    if (prop.isPrivate) {
      return;
    }

    if (!this.typesCache[this.graphqlTypeName(prop, true)]) {
      this.generateInputTypes(prop, component);

      const request = inputObjectType({
        name: `${this.graphqlTypeName(prop, true)}`,
        description: `${prop.label} Request`,
        definition(t) {
          for (const rp of prop.request.properties.attrs) {
            thisGenerator.propAsType(rp, { nexusTypeDef: t, inputType: true });
          }
        },
      });
      this.typesCache[this.graphqlTypeName(prop, true)] = true;
      this.types.push(request);
    }

    if (!this.typesCache[`${this.graphqlTypeName(prop)}Reply`]) {
      const reply = objectType({
        name: `${this.graphqlTypeName(prop)}Reply`,
        description: `${prop.label} Reply`,
        definition(t) {
          for (const rp of prop.reply.properties.attrs) {
            thisGenerator.propAsType(rp, { nexusTypeDef: t, inputType: false });
          }
        },
      });
      this.typesCache[`${this.graphqlTypeName(prop)}Reply`] = true;
      this.types.push(reply);
    }

    let addField = queryField;
    if (mutation) {
      // @ts-ignore
      addField = mutationField;
    }
    // @ts-ignore
    const query = addField(camelCase(this.graphqlTypeName(prop)), {
      // @ts-ignore
      type: `${this.graphqlTypeName(prop)}Reply`,
      args: {
        input: arg({
          // @ts-ignore
          type: this.graphqlTypeName(prop, true),
          nullable: !prop.required,
        }),
      },
      async resolve(_root, { input }, context: any): Promise<any> {
        const trace = traceApi.trace.getTracer("si-graphql-api");
        const span = trace.startSpan(
          `graphql.resolver ${camelCase(thisGenerator.graphqlTypeName(prop))}`,
          { parent: trace.getCurrentSpan() },
        );
        span.setAttribute("graphql.resolver", true);
        span.setAttribute("graphql.root", true);
        if (mutation) {
          span.setAttribute("graphql.mutation", true);
        } else {
          span.setAttribute("graphql.query", true);
        }
        const grpc = context.dataSources.grpc;
        const user = context.user;
        if (user.authenticated == false && prop.skipAuth != true) {
          span.end();
          throw new AuthenticationError("Must be logged in");
        }
        const metadata = new Metadata();
        metadata.add("authenticated", `${user.authenticated}`);
        metadata.add("userId", user["userId"] || "");
        metadata.add("billingAccountId", user["billingAccountId"] || "");
        const g = grpc.service(component.siPathName);
        // eslint-disable-next-line
        let req: any;
        if (input) {
          const grpcInput = thisGenerator.transformGraphqlToGrpc(
            input,
            prop,
            component,
          );
          req = new g.Request(
            `${camelCase(component.typeName)}${pascalCase(prop.name)}`,
            grpcInput,
          )
            .withMetadata(metadata)
            .withRetry(0);
        } else {
          req = new g.Request(
            `${camelCase(component.typeName)}${pascalCase(prop.name)}`,
            {},
          )
            .withMetadata(metadata)
            .withRetry(0);
        }
        let result = await req.exec();
        result = thisGenerator.transformGrpcToGraphql(
          result.response,
          prop,
          component,
        );
        span.end();
        return result;
      },
    });
    this.types.push(query);
  }

  // Transform all well-known types and maps to be the format expected by
  // our GRPC protobuf call.
  transformGraphqlToGrpc(
    // eslint-disable-next-line
    input: Record<string, any>,
    prop: PropMethod | PropAction,
    component: ObjectTypes,
    // eslint-disable-next-line
  ): Record<string, any> {
    for (const p of prop.request.properties.attrs) {
      if (input[this.graphqlFieldName(p)] == undefined) {
        continue;
      }
      input[this.graphqlFieldName(p)] = this.transformGraphqlFieldToGrpc(
        input[this.graphqlFieldName(p)],
        p,
        component,
      );
    }
    return input;
  }

  transformGraphqlFieldToGrpc(
    // eslint-disable-next-line
    input: any,
    prop: Props,
    component: ObjectTypes,
    ignoreRepeated = false,
    // eslint-disable-next-line
  ): any {
    if (prop.repeated && !ignoreRepeated) {
      const newArrayValues = [];
      for (const fieldValue of input) {
        newArrayValues.push(
          this.transformGraphqlFieldToGrpc(fieldValue, prop, component, true),
        );
      }
      return newArrayValues;
    } else if (
      prop.kind() == "text" ||
      prop.kind() == "number" ||
      prop.kind() == "bool" ||
      prop.kind() == "code" ||
      prop.kind() == "password"
    ) {
      return {
        value: input,
      };
    } else if (prop.kind() == "map") {
      const newMap: Record<string, any> = {};
      if (!Array.isArray(input)) {
        throw `Cannot generate GRPC call; map type value is not an array ${this.graphqlFieldName(
          prop,
        )}: ${JSON.stringify(input)}`;
      }
      for (const entry of input) {
        newMap[entry.key] = { value: entry.value };
      }
      return newMap;
    } else if (prop.kind() == "object") {
      // @ts-ignore
      for (const internalProp of prop.properties.attrs) {
        if (input[this.graphqlFieldName(internalProp)] != undefined) {
          input[
            this.graphqlFieldName(internalProp)
          ] = this.transformGraphqlFieldToGrpc(
            input[this.graphqlFieldName(internalProp)],
            internalProp,
            component,
          );
        }
      }
      return input;
    } else if (prop.kind() == "link") {
      // Think this will work? Probably, as long as the fields never get renamed?
      return this.transformGraphqlFieldToGrpc(
        input,
        // @ts-ignore
        prop.lookupMyself(),
        component,
      );
    } else if (prop.kind() == "enum") {
      const enumProp = prop as PropEnum;
      const inputString = `${input}`;
      for (let index = 0; index < enumProp.variants.length; index++) {
        if (
          enumProp.variants[index].toLowerCase() == inputString.toLowerCase()
        ) {
          return index + 1;
        }
      }
      logger.log("warn", "Unknown enum", { prop, input });
      return 0;
    } else {
      console.log(
        `I don't know what you are: ${this.graphqlFieldName(
          prop,
        )}, kind: ${prop.kind()}`,
      );
    }
    return input;
  }

  // Transform all well-known types and maps to be the format expected by
  // our GRPC protobuf call.
  transformGrpcToGraphql(
    // eslint-disable-next-line
    input: Record<string, any>,
    prop: PropMethod | PropAction,
    component: ObjectTypes,
    // eslint-disable-next-line
  ): Record<string, any> {
    for (const p of prop.reply.properties.attrs) {
      if (input[this.graphqlFieldName(p)] == undefined) {
        continue;
      }
      input[this.graphqlFieldName(p)] = this.transformGrpcFieldToGraphql(
        input[this.graphqlFieldName(p)],
        p,
        component,
      );
    }
    return input;
  }

  transformGrpcFieldToGraphql(
    // eslint-disable-next-line
    input: any,
    prop: Props,
    component: ObjectTypes,
    ignoreRepeated = false,
    // eslint-disable-next-line
  ): any {
    if (prop.repeated && !ignoreRepeated) {
      const newArrayValues = [];
      for (const fieldValue of input) {
        newArrayValues.push(
          this.transformGrpcFieldToGraphql(fieldValue, prop, component, true),
        );
      }
      return newArrayValues;
    } else if (
      prop.kind() == "text" ||
      prop.kind() == "number" ||
      prop.kind() == "bool" ||
      prop.kind() == "code" ||
      prop.kind() == "password"
    ) {
      return input["value"];
    } else if (prop.kind() == "map") {
      const newMapArray = [];
      for (const key in input) {
        newMapArray.push({ key, value: input[key]["value"] });
      }
      return newMapArray;
    } else if (prop.kind() == "object") {
      // @ts-ignore
      for (const internalProp of prop.properties.attrs) {
        if (input[this.graphqlFieldName(internalProp)] != undefined) {
          input[
            this.graphqlFieldName(internalProp)
          ] = this.transformGrpcFieldToGraphql(
            input[this.graphqlFieldName(internalProp)],
            internalProp,
            component,
          );
        }
      }
      return input;
    } else if (prop.kind() == "link") {
      // Think this will work? Probably, as long as the fields never get renamed?
      return this.transformGrpcFieldToGraphql(
        input,
        // @ts-ignore
        prop.lookupMyself(),
        component,
      );
    } else if (prop.kind() == "enum") {
      return input;
    } else {
      console.log(
        `I don't know what you are: ${this.graphqlFieldName(
          prop,
        )}, kind: ${prop.kind()}`,
      );
    }
    return input;
  }

  stringField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = this.makeFieldConfig(prop, { inputType });

    if (nexusTypeDef) {
      if (prop.repeated) {
        if (this.graphqlFieldName(prop) == "id") {
          nexusTypeDef.list.id(this.graphqlFieldName(prop), fieldConfig);
        } else {
          nexusTypeDef.list.string(this.graphqlFieldName(prop), fieldConfig);
        }
      } else {
        if (this.graphqlFieldName(prop) == "id") {
          nexusTypeDef.id(this.graphqlFieldName(prop), fieldConfig);
        } else {
          nexusTypeDef.string(this.graphqlFieldName(prop), fieldConfig);
        }
      }
    }
  }

  intField(prop: Props, { nexusTypeDef, inputType }: NexusBlockOptions): void {
    const fieldConfig = this.makeFieldConfig(prop, { inputType });

    if (nexusTypeDef) {
      const numberProp = prop as PropNumber;
      if (numberProp.numberKind == "int32") {
        if (prop.repeated) {
          nexusTypeDef.list.int(this.graphqlFieldName(prop), fieldConfig);
        } else {
          nexusTypeDef.int(this.graphqlFieldName(prop), fieldConfig);
        }
      } else {
        if (prop.repeated) {
          nexusTypeDef.list.string(this.graphqlFieldName(prop), fieldConfig);
        } else {
          nexusTypeDef.string(this.graphqlFieldName(prop), fieldConfig);
        }
      }
    }
  }

  transformLinkField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = this.makeFieldConfig(prop, { inputType });

    // @ts-ignore
    const realProp = prop.lookupMyself();
    if (nexusTypeDef) {
      // @ts-ignore
      if (realProp.kind() == "object") {
        if (prop.repeated) {
          // @ts-ignore
          nexusTypeDef.list.field(this.graphqlFieldName(prop), {
            // @ts-ignore
            type: this.graphqlTypeName(realProp, inputType),
            ...fieldConfig,
          });
        } else {
          // @ts-ignore
          nexusTypeDef.field(this.graphqlFieldName(prop), {
            // @ts-ignore
            type: this.graphqlTypeName(realProp, inputType),
            ...fieldConfig,
          });
        }
      } else if (realProp.kind() == "text" || realProp.kind() == "code") {
        this.stringField(prop, { nexusTypeDef, inputType });
      } else if (realProp.kind() == "number") {
        this.intField(prop, { nexusTypeDef, inputType });
      } else if (realProp.kind() == "enum") {
        if (prop.repeated) {
          // @ts-ignore
          nexusTypeDef.list.field(this.graphqlFieldName(prop), {
            type: this.graphqlTypeName(realProp),
            ...fieldConfig,
          });
        } else {
          // @ts-ignore
          nexusTypeDef.field(this.graphqlFieldName(prop), {
            type: this.graphqlTypeName(realProp),
            ...fieldConfig,
          });
        }
      } else {
        console.log("um, not sure what to do for you", {
          realProp,
          prop,
          kind: realProp.kind(),
        });
      }
    } else if (inputType) {
      if (realProp.kind() == "object") {
        if (this.loopDetector[this.graphqlTypeName(realProp, inputType)]) {
          return;
        }
        this.loopDetector[this.graphqlTypeName(realProp, inputType)] = true;
        this.propAsType(realProp, { inputType });
      }
    }
  }

  booleanField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = this.makeFieldConfig(prop, { inputType });

    if (nexusTypeDef) {
      if (prop.repeated) {
        nexusTypeDef.list.boolean(this.graphqlFieldName(prop), fieldConfig);
      } else {
        nexusTypeDef.boolean(this.graphqlFieldName(prop), fieldConfig);
      }
    }
  }

  enumField(prop: Props, { nexusTypeDef, inputType }: NexusBlockOptions): void {
    const fieldConfig = this.makeFieldConfig(prop, { inputType });

    if (nexusTypeDef) {
      if (prop.repeated) {
        // @ts-ignore
        nexusTypeDef.list.field(this.graphqlFieldName(prop), {
          // @ts-ignore
          type: this.graphqlTypeName(prop),
          ...fieldConfig,
        });
      } else {
        // @ts-ignore
        nexusTypeDef.field(this.graphqlFieldName(prop), {
          // @ts-ignore
          type: this.graphqlTypeName(prop),
          ...fieldConfig,
        });
      }
    } else {
      if (this.typesCache[this.graphqlTypeName(prop)]) {
        return;
      }
      this.typesCache[this.graphqlTypeName(prop)] = true;
      const et = enumType({
        name: this.graphqlTypeName(prop),
        description: prop.label,
        // @ts-ignore
        members: prop.variants.map((v: string) => constantCase(v)),
      });
      this.types.push(et);
    }
  }

  makeFieldConfig(prop: Props, { inputType }: NexusBlockOptions): FieldConfig {
    const fieldConfig: FieldConfig = {
      description: prop.label,
    };
    if (inputType) {
      fieldConfig["nullable"] = !prop.required;
    }
    return fieldConfig;
  }

  objectField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = this.makeFieldConfig(prop, { inputType });

    if (nexusTypeDef) {
      if (prop.repeated) {
        // @ts-ignore
        nexusTypeDef.list.field(this.graphqlFieldName(prop), {
          // @ts-ignore
          type: this.graphqlTypeName(prop, inputType),
          ...fieldConfig,
        });
      } else {
        // @ts-ignore
        nexusTypeDef.field(this.graphqlFieldName(prop), {
          // @ts-ignore
          type: this.graphqlTypeName(prop, inputType),
          ...fieldConfig,
        });
      }
    }

    if (this.typesCache[this.graphqlTypeName(prop, inputType)]) {
      return;
    }

    // eslint-disable-next-line
    const thisGenerator = this;

    // @ts-ignore
    for (const p of prop.properties.attrs) {
      if (p.kind() == "object") {
        thisGenerator.propAsType(p, { inputType });
      } else if (p.kind() == "enum") {
        thisGenerator.propAsType(p, { inputType });
      } else if (p.kind() == "map") {
        thisGenerator.propAsType(p, { inputType });
      } else if (p.kind() == "link") {
        const realProp = p.lookupMyself();
        //if (
        //  !this.typesCache[realProp.graphqlTypeName(inputType)] &&
        //  realProp.kind() == "object"
        //) {
        //}
        if (realProp.kind() == "object") {
          thisGenerator.propAsType(p, { inputType });
          // TODO: the issue is that when we recurse for input types,
          // or any other kind for that matter, we can get in big fucking
          // trouble with loops, like the one we have in query. its bad.
          // but the fix is *right here somewhere*
        }
      }
    }

    if (!this.typesCache[this.graphqlTypeName(prop, inputType)]) {
      let createType = objectType;
      if (inputType) {
        // @ts-ignore
        createType = inputObjectType;
      }
      const ot = createType({
        name: this.graphqlTypeName(prop, inputType),
        definition(t) {
          // @ts-ignore
          for (const p of prop.properties.attrs) {
            if (p.kind() == "object") {
              if (p.repeated) {
                t.list.field(thisGenerator.graphqlFieldName(p), {
                  // @ts-ignore
                  type: thisGenerator.graphqlTypeName(p, inputType),
                  ...fieldConfig,
                });
              } else {
                t.field(thisGenerator.graphqlFieldName(p), {
                  // @ts-ignore
                  type: thisGenerator.graphqlTypeName(p, inputType),
                  ...fieldConfig,
                });
              }
            } else {
              thisGenerator.propAsType(p, { nexusTypeDef: t, inputType });
            }
          }
        },
      });
      this.typesCache[this.graphqlTypeName(prop, inputType)] = true;
      this.types.push(ot);
    }
  }

  transformMapField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = this.makeFieldConfig(prop, { inputType });

    if (nexusTypeDef) {
      if (prop.repeated) {
        // @ts-ignore
        nexusTypeDef.list.field(this.graphqlFieldName(prop), {
          // @ts-ignore
          type: this.graphqlTypeName(prop, inputType),
          ...fieldConfig,
        });
      } else {
        // @ts-ignore
        nexusTypeDef.field(this.graphqlFieldName(prop), {
          // @ts-ignore
          type: this.graphqlTypeName(prop, inputType),
          ...fieldConfig,
        });
      }
    } else {
      if (!this.typesCache[this.graphqlTypeName(prop, inputType)]) {
        // eslint-disable-next-line
        let typeFun: any;
        if (inputType) {
          typeFun = inputObjectType;
        } else {
          typeFun = objectType;
        }
        const mapType = typeFun({
          name: this.graphqlTypeName(prop, inputType),
          description: prop.label,
          // eslint-disable-next-line
          definition(mapt: any) {
            mapt.string("key");
            mapt.string("value");
          },
        });
        this.typesCache[this.graphqlTypeName(prop, inputType)] = true;
        this.types.push(mapType);
      }
    }
  }

  propAsType(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    if (prop.skip) {
      return;
    }
    // yes, this is a binding of this. I need it. Shut your face.
    // eslint-disable-next-line
    if (prop.kind() == "object") {
      this.objectField(prop, { nexusTypeDef, inputType });
    } else if (
      prop.kind() == "text" ||
      prop.kind() == "code" ||
      prop.kind() == "password"
    ) {
      this.stringField(prop, { nexusTypeDef, inputType });
    } else if (prop.kind() == "number") {
      this.intField(prop, { nexusTypeDef, inputType });
    } else if (prop.kind() == "bool") {
      this.booleanField(prop, { nexusTypeDef, inputType });
    } else if (prop.kind() == "link") {
      this.transformLinkField(prop, { nexusTypeDef, inputType });
    } else if (prop.kind() == "enum") {
      this.enumField(prop, { nexusTypeDef, inputType });
    } else if (prop.kind() == "map") {
      this.transformMapField(prop, { nexusTypeDef, inputType });
    } else {
      console.dir(prop);
      throw `Cannot transform this prop to a graphql type - bug: ${prop.kind()}`;
    }
  }

  entityField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = this.makeFieldConfig(prop, { inputType });

    if (nexusTypeDef) {
      if (prop.repeated) {
        // @ts-ignore
        nexusTypeDef.list.field(this.graphqlFieldName(prop), {
          type: this.graphqlTypeName(prop, inputType),
          ...fieldConfig,
        });
      } else {
        // @ts-ignore
        nexusTypeDef.field(this.graphqlFieldName(prop), {
          type: this.graphqlTypeName(prop, inputType),
          ...fieldConfig,
        });
      }
    }
  }

  //generateComponentNoStandard(component: Component): void {
  //  for (const prop of component.internalOnly.attrs) {
  //    if (prop.kind() != "object") {
  //      logger.log(
  //        "warn",
  //        "Found top level prop whose kind was not object; skipping it!",
  //        { prop },
  //      );
  //      continue;
  //    }
  //    this.propAsType(prop, { inputType: false });
  //  }
  //}

  //generateComponentStandard(component: Component): void {
  //  // First, you have to catch any objects or enums defined - because if you
  //  // don't, they might only be linked in the entity later, and then won't
  //  // appear in the API. Sad clown!
  //  for (const prop of component.properties.attrs) {
  //    if (prop.kind() == "object" || prop.kind() == "enum") {
  //      this.propAsType(prop, { inputType: false });
  //    }
  //  }
  //  this.propAsType(component.asComponent(), { inputType: false });
  //  this.propAsType(component.asConstraints(), { inputType: true });
  //  this.propAsType(component.asProperties(), { inputType: true });
  //  this.propAsType(component.asEntity(), { inputType: false });
  //  this.propAsType(component.asEntityEvent(), { inputType: false });
  //}

  generateComponent(systemObject: ObjectTypes): void {
    this.propAsType(systemObject.rootProp, { inputType: false });
  }
}

export const registryGenerator = new SiRegistryGenerator();
