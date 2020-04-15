import { Component, ComponentConstructor } from "./component";
import { Props } from "./attrList";

export interface PropLookup {
  component: string;
  propType:
    | "siProperties"
    | "constraints"
    | "properties"
    | "data"
    | "internalOnly";
  names: string[];
}

export class ComponentRegistry {
  components: Component[];

  constructor() {
    this.components = [];
  }

  // Find a property!
  lookupProp(lookup: PropLookup): Props {
    const component = this.components.find(c => c.typeName == lookup.component);
    if (!component) {
      throw `Cannot find component: ${component}`;
    }
    const firstName = lookup.names[0];
    let returnProp = component[lookup.propType].getEntry(firstName);
    if (!returnProp) {
      throw `Cannot find prop on component ${component.typeName}: ${firstName}`;
    }
    if (returnProp.kind() != "object" && lookup.names.length > 1) {
      throw `You asked for sub-properties of a non-object type on ${component.typeName} property ${firstName}`;
    }
    for (let i = 1; i < lookup.names.length; i++) {
      const lookupName = lookup.names[i];
      const lookupResult = returnProp["properties"].getEntry(lookupName);
      if (!lookupResult) {
        throw `Cannot find prop "${lookupName}" on ${returnProp.name}`;
      }

      if (i != lookup.names.length - 1 && lookupResult.kind() != "object") {
        console.log({
          i,
          length: lookup.names.length,
          lookupName,
          lookupResult,
        });
        throw `Cannot look up a sub-property of a non object Prop: ${
          component.typeName
        } property ${lookupName} is ${lookupResult.kind()}`;
      }

      returnProp = lookupResult;
    }
    return returnProp;
  }

  get(typeName: string): Component | undefined {
    return this.components.find(v => v.typeName == typeName);
  }

  component(constructorArgs: ComponentConstructor): Component {
    // Strong Bad Joke
    const compy = new Component(constructorArgs);
    // We need to be in the registry before we evaluate options -
    // thank god objects are always pointers!
    this.components.push(compy);
    if (compy.noStd == false) {
      compy.setDefaultValues();
    }
    if (constructorArgs.options) {
      constructorArgs.options(compy);
    }
    return compy;
  }
}

export const registry = new ComponentRegistry();
