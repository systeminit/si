import {
  ObjectTypes,
  BaseObjectConstructor,
  SystemObject,
  BaseObject,
  ComponentObject,
  EntityObject,
  ComponentAndEntityObject,
  ComponentAndEntityObjectConstructor,
} from "./systemComponent";
import { Props, PropAction } from "./attrList";
import { camelCase } from "change-case";
import _ from "lodash";
import { promises as fs } from "fs";

export interface PropLookup {
  typeName: string;
  names?: string[];
}

export class Registry {
  objects: ObjectTypes[];

  constructor() {
    this.objects = [];
  }

  async serialize(): Promise<void> {
    await fs.writeFile(
      "/tmp/registry.json",
      JSON.stringify(this.objects, null, 2),
    );
  }

  inputTypesFor(typeName: string): EntityObject[] {
    // @ts-ignore
    const results: EntityObject[] = _.filter(this.objects, o => {
      if (o.kind() == "entityObject") {
        const eo = o as EntityObject;
        if (_.find(eo.inputTypes, ["typeName", typeName])) {
          return true;
        } else {
          return false;
        }
      } else {
        return false;
      }
    });
    return results;
  }

  get(typeName: string): ObjectTypes {
    const result = this.objects.find(v => v.typeName == camelCase(typeName));
    if (result) {
      return result;
    } else {
      throw new Error(`Cannot get object named ${typeName} in the registry`);
    }
  }

  serviceNames(): string[] {
    const names = new Set();
    for (const object of this.objects) {
      if (object.serviceName) {
        names.add(object.serviceName);
      }
    }
    const arrayNames = [];
    for (const name of names.values()) {
      arrayNames.push(`${name}`);
    }
    return arrayNames;
  }

  getObjectsForServiceName(serviceName: string): ObjectTypes[] {
    const results = [];
    for (const object of this.objects) {
      if (object.serviceName == serviceName) {
        results.push(object);
      }
    }
    return results;
  }

  // Find a property!
  lookupProp(lookup: PropLookup): Props {
    const foundObject = this.objects.find(c => c.typeName == lookup.typeName);
    if (!foundObject) {
      throw new Error(`Cannot find object: ${foundObject}`);
    }
    if (!lookup.names) {
      return foundObject.rootProp;
    }
    const firstName = lookup.names[0];
    let returnProp = foundObject.fields.getEntry(firstName);
    if (!returnProp) {
      throw new Error(
        `Cannot find prop on object ${foundObject.typeName}: ${firstName}`,
      );
    }
    if (returnProp.kind() != "object" && lookup.names.length > 1) {
      throw new Error(
        `You asked for sub-properties of a non-object type on ${foundObject.typeName} property ${firstName}`,
      );
    }
    for (let i = 1; i < lookup.names.length; i++) {
      const lookupName = lookup.names[i];
      // @ts-ignore
      const lookupResult = returnProp["properties"].getEntry(lookupName);
      if (!lookupResult) {
        throw new Error(
          `Cannot find prop "${lookupName}" on ${returnProp.name}`,
        );
      }

      if (i != lookup.names.length - 1 && lookupResult.kind() != "object") {
        console.log({
          i,
          length: lookup.names.length,
          lookupName,
          lookupResult,
        });
        throw new Error(
          `Cannot look up a sub-property of a non object Prop: ${
            foundObject.typeName
          } property ${lookupName} is ${lookupResult.kind()}`,
        );
      }

      returnProp = lookupResult;
    }
    return returnProp;
  }

  listEntities(): EntityObject[] {
    const results: EntityObject[] = [];
    for (const object of this.objects) {
      if (object instanceof EntityObject) {
        results.push(object);
      }
    }
    return results;
  }

  listActions(): { [key: string]: string[] } {
    const results: { [key: string]: string[] } = {};
    for (const entity of this.objects) {
      results[entity.typeName] = [];
      if (entity instanceof EntityObject) {
        for (const attr of entity.methods.attrs) {
          if (attr instanceof PropAction) {
            results[entity.typeName].push(attr.name);
          }
        }
      }
    }
    return results;
  }

  // These are "basic" objects - they don't have any extra behavior or
  // automatic fields. They just store the fields you give them.
  base(constructorArgs: BaseObjectConstructor): BaseObject {
    const compy = new BaseObject(constructorArgs);
    this.objects.push(compy);
    if (constructorArgs.options) {
      constructorArgs.options(compy);
    }
    return compy;
  }

  // These are "system" objects - they have what is needed to be an object
  // inside our system. They come with things like types, IDs, tenancy,
  // etc.
  system(constructorArgs: BaseObjectConstructor): SystemObject {
    const compy = new SystemObject(constructorArgs);
    this.objects.push(compy);
    if (constructorArgs.options) {
      constructorArgs.options(compy);
    }
    return compy;
  }

  component(constructorArgs: BaseObjectConstructor): ComponentObject {
    const compy = new ComponentObject(constructorArgs);
    this.objects.push(compy);
    if (constructorArgs.options) {
      constructorArgs.options(compy);
    }
    return compy;
  }

  entity(constructorArgs: BaseObjectConstructor): EntityObject {
    const compy = new EntityObject(constructorArgs);
    this.objects.push(compy);
    if (constructorArgs.options) {
      constructorArgs.options(compy);
    }
    return compy;
  }

  componentAndEntity(
    constructorArgs: ComponentAndEntityObjectConstructor,
  ): ComponentAndEntityObject {
    const compy = new ComponentAndEntityObject(constructorArgs);
    this.objects.push(compy.component);
    this.objects.push(compy.entity);
    this.objects.push(compy.entityEvent);
    if (constructorArgs.options) {
      constructorArgs.options(compy);
    }
    return compy;
  }
}

export const registry = new Registry();
