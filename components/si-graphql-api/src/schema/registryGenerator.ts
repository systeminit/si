import { Metadata } from "grpc";
import {
  registry,
  Component,
  Props,
  PropMethod,
  PropAction,
  PropObject,
  PropLink,
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
import { ObjectDefinitionBlock, InputDefinitionBlock } from "nexus/dist/core";
import { camelCase, pascalCase, constantCase } from "change-case";
import { Context } from "@/index";
import { AuthenticationError } from "apollo-server";

interface NexusBlockOptions {
  nexusTypeDef?: NexusTypeDefBlock;
  inputType: boolean;
}

type NexusTypeDefBlock =
  | ObjectDefinitionBlock<string>
  | InputDefinitionBlock<string>;

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
    for (const component of registry.components) {
      logger.debug(
        `*** Generating objects for component ${component.typeName} ***`,
      );
      this.generateComponent(component);
      logger.debug(
        `*** Generating methods for component ${component.typeName} ***`,
      );
      this.generateMethods(component);
    }
  }

  generateMethods(component: Component): void {
    for (const method of component.componentMethods.attrs) {
      if (method.kind() == "method" || method.kind() == "action") {
        // @ts-ignore
        this.generateQueryOrMutationField(method, component, method.mutation);
      }
    }
    for (const method of component.entityMethods.attrs) {
      if (method.kind() == "method" || method.kind() == "action") {
        // @ts-ignore
        this.generateQueryOrMutationField(method, component, method.mutation);
      }
    }
    for (const method of component.entityActions.attrs) {
      if (method.kind() == "method" || method.kind() == "action") {
        // @ts-ignore
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
        if (!this.typesCache[cp.graphqlTypeName(true)]) {
          this.typesCache[cp.graphqlTypeName(true)] = true;
          this.generateInputObjectTypes(cp);
        } else {
          this.typesCache[cp.graphqlTypeName(true)];
        }
      }
    }
  }

  generateInputTypes(
    prop: PropMethod | PropAction,
    _component: Component,
  ): void {
    for (const rp of prop.request.attrs) {
      if (rp.kind() == "object" || rp.kind() == "link") {
        // @ts-ignore
        this.generateInputObjectTypes(rp);
      }
    }
  }

  generateQueryOrMutationField(
    prop: PropMethod | PropAction,
    component: Component,
    mutation = false,
  ): void {
    // eslint-disable-next-line
    const thisGenerator = this;

    if (!this.typesCache[prop.graphqlTypeName(true)]) {
      this.generateInputTypes(prop, component);

      const request = inputObjectType({
        name: `${prop.graphqlTypeName(true)}`,
        description: `${prop.label} Request`,
        definition(t) {
          for (const rp of prop.request.attrs) {
            thisGenerator.propAsType(rp, { nexusTypeDef: t, inputType: true });
          }
        },
      });
      this.typesCache[prop.graphqlTypeName(true)] = true;
      this.types.push(request);
    }

    if (!this.typesCache[`${prop.graphqlTypeName()}Reply`]) {
      const reply = objectType({
        name: `${prop.graphqlTypeName()}Reply`,
        description: `${prop.label} Reply`,
        definition(t) {
          for (const rp of prop.reply.attrs) {
            thisGenerator.propAsType(rp, { nexusTypeDef: t, inputType: false });
          }
        },
      });
      this.typesCache[`${prop.graphqlTypeName()}Reply`] = true;
      this.types.push(reply);
    }

    let addField = queryField;
    if (mutation) {
      // @ts-ignore
      addField = mutationField;
    }
    // @ts-ignore
    const query = addField(camelCase(prop.graphqlTypeName()), {
      // @ts-ignore
      type: `${prop.graphqlTypeName()}Reply`,
      args: {
        // @ts-ignore
        input: arg({ type: prop.graphqlTypeName(true) }),
      },
      async resolve(
        _root,
        { input },
        { dataSources: { grpc }, user }: Context,
      ) {
        if (user.authenticated == false && prop.skipAuth != true) {
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
          req = new g.Request(pascalCase(prop.name), grpcInput).withMetadata(
            metadata,
          );
        } else {
          req = new g.Request(pascalCase(prop.name), {}).withMetadata(metadata);
        }
        let result = await req.exec();
        result = thisGenerator.transformGrpcToGraphql(
          result.response,
          prop,
          component,
        );
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
    component: Component,
    // eslint-disable-next-line
  ): Record<string, any> {
    for (const p of prop.request.attrs) {
      if (input[p.graphqlFieldName()] == undefined) {
        continue;
      }
      input[p.graphqlFieldName()] = this.transformGraphqlFieldToGrpc(
        input[p.graphqlFieldName()],
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
    component: Component,
    ignoreRepeated = false,
    // eslint-disable-next-line
  ): any {
    if (prop.repeated && !ignoreRepeated) {
      const newArrayValues = [];
      for (const fieldValue in input[prop.graphqlFieldName()]) {
        newArrayValues.push(
          this.transformGraphqlFieldToGrpc(fieldValue, prop, component, true),
        );
      }
      return newArrayValues;
    } else if (
      prop.kind() == "text" ||
      prop.kind() == "number" ||
      prop.kind() == "bool" ||
      prop.kind() == "code"
    ) {
      return {
        value: input,
      };
    } else if (prop.kind() == "map") {
      const newMap = {};
      if (!Array.isArray(input)) {
        throw `Cannot generate GRPC call; map type value is not an array ${prop.graphqlFieldName()}: ${JSON.stringify(
          input,
        )}`;
      }
      for (const entry of input) {
        newMap[entry.key] = { value: entry.value };
      }
      return newMap;
    } else if (prop.kind() == "object") {
      // @ts-ignore
      for (const internalProp of prop.properties.attrs) {
        if (input[internalProp.graphqlFieldName()] != undefined) {
          input[
            internalProp.graphqlFieldName()
          ] = this.transformGraphqlFieldToGrpc(
            input[internalProp.graphqlFieldName()],
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
    } else if (prop.kind() == "constraints") {
      return this.transformGraphqlFieldToGrpc(
        input,
        component.asConstraints(),
        component,
      );
    } else if (prop.kind() == "properties") {
      return this.transformGraphqlFieldToGrpc(
        input,
        component.asProperties(),
        component,
      );
    } else if (prop.kind() == "enum") {
      return input;
    } else {
      console.log(
        `I don't know what you are: ${prop.graphqlFieldName()}, kind: ${prop.kind()}`,
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
    component: Component,
    // eslint-disable-next-line
  ): Record<string, any> {
    for (const p of prop.reply.attrs) {
      if (input[p.graphqlFieldName()] == undefined) {
        continue;
      }
      input[p.graphqlFieldName()] = this.transformGrpcFieldToGraphql(
        input[p.graphqlFieldName()],
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
    component: Component,
    ignoreRepeated = false,
    // eslint-disable-next-line
  ): any {
    if (prop.repeated && !ignoreRepeated) {
      const newArrayValues = [];
      for (const fieldValue in input[prop.graphqlFieldName()]) {
        newArrayValues.push(
          this.transformGrpcFieldToGraphql(fieldValue, prop, component, true),
        );
      }
      return newArrayValues;
    } else if (
      prop.kind() == "text" ||
      prop.kind() == "number" ||
      prop.kind() == "bool" ||
      prop.kind() == "code"
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
        if (input[internalProp.graphqlFieldName()] != undefined) {
          input[
            internalProp.graphqlFieldName()
          ] = this.transformGrpcFieldToGraphql(
            input[internalProp.graphqlFieldName()],
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
    } else if (prop.kind() == "constraints") {
      return this.transformGrpcFieldToGraphql(
        input,
        component.asConstraints(),
        component,
      );
    } else if (prop.kind() == "properties") {
      return this.transformGrpcFieldToGraphql(
        input,
        component.asProperties(),
        component,
      );
    } else if (prop.kind() == "entity") {
      return this.transformGrpcFieldToGraphql(
        input,
        component.asEntity(),
        component,
      );
    } else if (prop.kind() == "entityEvent") {
      return this.transformGrpcFieldToGraphql(
        input,
        component.asEntityEvent(),
        component,
      );
    } else if (prop.kind() == "enum") {
      return input;
    } else {
      console.log(
        `I don't know what you are: ${prop.graphqlFieldName()}, kind: ${prop.kind()}`,
      );
    }
    return input;
  }

  stringField(prop: Props, { nexusTypeDef }: NexusBlockOptions): void {
    const fieldConfig = {
      description: prop.label,
    };
    if (nexusTypeDef) {
      if (prop.repeated) {
        if (prop.graphqlFieldName() == "id") {
          nexusTypeDef.list.id(prop.graphqlFieldName(), fieldConfig);
        } else {
          nexusTypeDef.list.string(prop.graphqlFieldName(), fieldConfig);
        }
      } else {
        if (prop.graphqlFieldName() == "id") {
          nexusTypeDef.id(prop.graphqlFieldName(), fieldConfig);
        } else {
          nexusTypeDef.string(prop.graphqlFieldName(), fieldConfig);
        }
      }
    }
  }

  intField(prop: Props, { nexusTypeDef }: NexusBlockOptions): void {
    const fieldConfig = {
      description: prop.label,
    };
    if (nexusTypeDef) {
      if (prop.repeated) {
        nexusTypeDef.list.int(prop.graphqlFieldName(), fieldConfig);
      } else {
        nexusTypeDef.int(prop.graphqlFieldName(), fieldConfig);
      }
    }
  }

  transformLinkField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = {
      description: prop.label,
    };
    // @ts-ignore
    const realProp = prop.lookupMyself();
    if (nexusTypeDef) {
      // @ts-ignore
      if (realProp.kind() == "object") {
        if (prop.repeated) {
          // @ts-ignore
          nexusTypeDef.list.field(prop.graphqlFieldName(), {
            // @ts-ignore
            type: realProp.graphqlTypeName(inputType),
            ...fieldConfig,
          });
        } else {
          // @ts-ignore
          nexusTypeDef.list.field(prop.graphqlFieldName(), {
            // @ts-ignore
            type: realProp.graphqlTypeName(inputType),
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
          nexusTypeDef.list.field(prop.graphqlFieldName(), {
            type: realProp.graphqlTypeName(),
            ...fieldConfig,
          });
        } else {
          // @ts-ignore
          nexusTypeDef.field(prop.graphqlFieldName(), {
            type: realProp.graphqlTypeName(),
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
        if (this.loopDetector[realProp.graphqlTypeName(inputType)]) {
          return;
        }
        this.loopDetector[realProp.graphqlTypeName(inputType)] = true;
        this.propAsType(realProp, { inputType });
      }
    }
  }

  booleanField(prop: Props, { nexusTypeDef }: NexusBlockOptions): void {
    const fieldConfig = {
      description: prop.label,
    };
    if (nexusTypeDef) {
      if (prop.repeated) {
        nexusTypeDef.list.boolean(prop.graphqlFieldName(), fieldConfig);
      } else {
        nexusTypeDef.boolean(prop.graphqlFieldName(), fieldConfig);
      }
    }
  }

  enumField(prop: Props, { nexusTypeDef }: NexusBlockOptions): void {
    const fieldConfig = {
      description: prop.label,
    };
    if (nexusTypeDef) {
      if (prop.repeated) {
        // @ts-ignore
        nexusTypeDef.list.field(prop.graphqlFieldName(), {
          // @ts-ignore
          type: prop.graphqlTypeName(),
          ...fieldConfig,
        });
      } else {
        // @ts-ignore
        nexusTypeDef.field(prop.graphqlFieldName(), {
          // @ts-ignore
          type: prop.graphqlTypeName(),
          ...fieldConfig,
        });
      }
    } else {
      if (this.typesCache[prop.graphqlTypeName()]) {
        return;
      }
      this.typesCache[prop.graphqlTypeName()] = true;
      const et = enumType({
        name: prop.graphqlTypeName(),
        description: prop.label,
        // @ts-ignore
        members: prop.variants.map((v: string) => constantCase(v)),
      });
      this.types.push(et);
    }
  }

  objectField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = {
      description: prop.label,
    };

    if (nexusTypeDef) {
      if (prop.repeated) {
        // @ts-ignore
        nexusTypeDef.list.field(prop.graphqlFieldName(), {
          // @ts-ignore
          type: prop.graphqlTypeName(inputType),
          ...fieldConfig,
        });
      } else {
        // @ts-ignore
        nexusTypeDef.field(prop.graphqlFieldName(), {
          // @ts-ignore
          type: prop.graphqlTypeName(inputType),
          ...fieldConfig,
        });
      }
    }

    if (this.typesCache[prop.graphqlTypeName(inputType)]) {
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

    if (!this.typesCache[prop.graphqlTypeName(inputType)]) {
      let createType = objectType;
      if (inputType) {
        // @ts-ignore
        createType = inputObjectType;
      }
      const ot = createType({
        name: prop.graphqlTypeName(inputType),
        definition(t) {
          // @ts-ignore
          for (const p of prop.properties.attrs) {
            if (p.kind() == "object") {
              if (p.repeated) {
                t.list.field(p.graphqlFieldName(), {
                  type: p.graphqlTypeName(inputType),
                  ...fieldConfig,
                });
              } else {
                t.field(p.graphqlFieldName(), {
                  type: p.graphqlTypeName(inputType),
                  ...fieldConfig,
                });
              }
            } else {
              thisGenerator.propAsType(p, { nexusTypeDef: t, inputType });
            }
          }
        },
      });
      this.typesCache[prop.graphqlTypeName(inputType)] = true;
      this.types.push(ot);
    }
  }

  transformMapField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = {
      description: prop.label,
    };

    if (nexusTypeDef) {
      if (prop.repeated) {
        // @ts-ignore
        nexusTypeDef.list.field(prop.graphqlFieldName(), {
          // @ts-ignore
          type: prop.graphqlTypeName(inputType),
          ...fieldConfig,
        });
      } else {
        // @ts-ignore
        nexusTypeDef.field(prop.graphqlFieldName(), {
          // @ts-ignore
          type: prop.graphqlTypeName(inputType),
          ...fieldConfig,
        });
      }
    } else {
      if (!this.typesCache[prop.graphqlTypeName(inputType)]) {
        // eslint-disable-next-line
        let typeFun: any;
        if (inputType) {
          typeFun = inputObjectType;
        } else {
          typeFun = objectType;
        }
        const mapType = typeFun({
          name: prop.graphqlTypeName(inputType),
          description: prop.label,
          // eslint-disable-next-line
          definition(mapt: any) {
            mapt.string("key");
            mapt.string("value");
          },
        });
        this.typesCache[prop.graphqlTypeName(inputType)] = true;
        this.types.push(mapType);
      }
    }
  }

  propAsType(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    // yes, this is a binding of this. I need it. Shut your face.
    // eslint-disable-next-line
    if (prop.kind() == "object") {
      this.objectField(prop, { nexusTypeDef, inputType });
    } else if (prop.kind() == "text" || prop.kind() == "code") {
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
    } else if (
      prop.kind() == "constraints" ||
      prop.kind() == "entity" ||
      prop.kind() == "entityEvent" ||
      prop.kind() == "component" ||
      prop.kind() == "properties"
    ) {
      this.existingField(prop, { nexusTypeDef, inputType });
    }
  }

  existingField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = {
      description: prop.label,
    };

    if (nexusTypeDef) {
      if (prop.repeated) {
        // @ts-ignore
        nexusTypeDef.list.field(prop.graphqlFieldName(), {
          type: prop.graphqlTypeName(inputType),
          ...fieldConfig,
        });
      } else {
        // @ts-ignore
        nexusTypeDef.field(prop.graphqlFieldName(), {
          type: prop.graphqlTypeName(inputType),
          ...fieldConfig,
        });
      }
    }
  }

  entityField(
    prop: Props,
    { nexusTypeDef, inputType }: NexusBlockOptions,
  ): void {
    const fieldConfig = {
      description: prop.label,
    };

    if (nexusTypeDef) {
      if (prop.repeated) {
        // @ts-ignore
        nexusTypeDef.list.field(prop.graphqlFieldName(), {
          type: prop.graphqlTypeName(inputType),
          ...fieldConfig,
        });
      } else {
        // @ts-ignore
        nexusTypeDef.field(prop.graphqlFieldName(), {
          type: prop.graphqlTypeName(inputType),
          ...fieldConfig,
        });
      }
    }
  }

  generateComponentNoStandard(component: Component): void {
    for (const prop of component.internalOnly.attrs) {
      if (prop.kind() != "object") {
        logger.log(
          "warn",
          "Found top level prop whose kind was not object; skipping it!",
          { prop },
        );
        continue;
      }
      this.propAsType(prop, { inputType: false });
    }
  }

  generateComponentStandard(component: Component): void {
    // First, you have to catch any objects or enums defined - because if you
    // don't, they might only be linked in the entity later, and then won't
    // appear in the API. Sad clown!
    for (const prop of component.properties.attrs) {
      if (prop.kind() == "object" || prop.kind() == "enum") {
        this.propAsType(prop, { inputType: false });
      }
    }
    this.propAsType(component.asComponent(), { inputType: false });
    this.propAsType(component.asConstraints(), { inputType: true });
    this.propAsType(component.asProperties(), { inputType: true });
    this.propAsType(component.asEntity(), { inputType: false });
    this.propAsType(component.asEntityEvent(), { inputType: false });
  }

  generateComponent(component: Component): void {
    if (component.noStd) {
      this.generateComponentNoStandard(component);
    } else {
      this.generateComponentStandard(component);
    }
  }
}
