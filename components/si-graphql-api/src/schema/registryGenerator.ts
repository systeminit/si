import { registry, Component, Props } from "si-registry";
import {
  arg,
  objectType,
  enumType,
  inputObjectType,
  queryType,
  extendType,
  mutationType,
  queryField,
} from "nexus";
import { logger } from "@/logger";
import { loginTypes } from "@/schema/login";
import { NexusObjectTypeDef, ObjectDefinitionBlock } from "nexus/dist/core";

export class SiRegistryGenerator {
  // eslint-disable-next-line
  //
  // This is any because it is any in nexus
  types: any[];

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
    for (const component of registry.components) {
      this.generateComponent(component);
    }
  }

  propAsType(prop: Props, nexusTypeDef?: ObjectDefinitionBlock<string>): void {
    // yes, this is a binding of this. I need it. Shut your face.
    // eslint-disable-next-line
    const thisGenerator = this;
    const fieldConfig = {
      description: prop.label,
    };
    if (prop.kind() == "object") {
      // @ts-ignore
      for (const p of prop.properties.attrs) {
        if (p.kind() == "object") {
          thisGenerator.propAsType(p);
        } else if (p.kind() == "enum") {
          thisGenerator.propAsType(p);
        }
      }
      const ot = objectType({
        name: prop.graphqlTypeName(),
        definition(t) {
          // @ts-ignore
          for (const p of prop.properties.attrs) {
            if (p.kind() == "object") {
              if (p.repeated) {
                t.list.field(p.graphqlFieldName(), {
                  type: p.graphqlTypeName(),
                  ...fieldConfig,
                });
              } else {
                t.field(p.graphqlFieldName(), {
                  type: p.graphqlTypeName(),
                  ...fieldConfig,
                });
              }
            } else {
              thisGenerator.propAsType(p, t);
            }
          }
        },
      });
      this.types.push(ot);
    } else if (prop.kind() == "text" || prop.kind() == "code") {
      if (nexusTypeDef) {
        if (prop.repeated) {
          nexusTypeDef.list.string(prop.graphqlFieldName(), fieldConfig);
        } else {
          nexusTypeDef.string(prop.graphqlFieldName(), fieldConfig);
        }
      }
    } else if (prop.kind() == "number") {
      if (nexusTypeDef) {
        if (prop.repeated) {
          nexusTypeDef.list.int(prop.graphqlFieldName(), fieldConfig);
        } else {
          nexusTypeDef.int(prop.graphqlFieldName(), fieldConfig);
        }
      }
    } else if (prop.kind() == "bool") {
      if (nexusTypeDef) {
        if (prop.repeated) {
          nexusTypeDef.list.boolean(prop.graphqlFieldName(), fieldConfig);
        } else {
          nexusTypeDef.boolean(prop.graphqlFieldName(), fieldConfig);
        }
      }
    } else if (prop.kind() == "enum") {
      if (nexusTypeDef) {
        if (prop.repeated) {
          nexusTypeDef.list.field(prop.graphqlFieldName(), {
            // @ts-ignore
            type: prop.graphqlTypeName(),
            ...fieldConfig,
          });
        } else {
          nexusTypeDef.list.field(prop.graphqlFieldName(), {
            // @ts-ignore
            type: prop.graphqlTypeName(),
            ...fieldConfig,
          });
        }
      } else {
        const et = enumType({
          name: prop.graphqlTypeName(),
          description: prop.label,
          // @ts-ignore
          members: prop.variants,
        });
        this.types.push(et);
      }
    }
  }

  // PropSelect
  // PropObject
  // PropMap
  // PropComponent
  // PropEntity
  // PropEntityEvent
  // PropConstraints
  // PropProperties
  // PropLink;

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
      this.propAsType(prop);
    }
  }

  generateComponentStandard(component: Component): void {
    this.propAsType(component.asComponent());
    this.propAsType(component.asEntity());
    this.propAsType(component.asEntityEvent());
  }

  generateComponent(component: Component): void {
    if (component.noStd) {
      this.generateComponentNoStandard(component);
    } else {
      this.generateComponentStandard(component);
    }
  }
}
