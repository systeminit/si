import { registry, PropLink, PropObject } from "si-registry";
import { Entity } from "@/api/sdf/model/entity";
import _ from "lodash";

export interface RegistryProperty {
  id: string;
  path: string[];
  prop: any;
  name: string;
  label: string;
  required: boolean;
  repeated: boolean;
  kind: string;
  hidden: boolean;
}

export interface PropEntry {
  prop: any;
  path: string[];
}

export function propertyList(
  entity: Entity,
  changeSetId?: string,
): RegistryProperty[] {
  let registryObject;
  if (entity.siStorable.typeName == "entity") {
    // @ts-ignore
    registryObject = registry.get(entity.objectType);
  } else {
    registryObject = registry.get("system");
  }

  const properties = registryObject.fields.getEntry("properties") as PropObject;
  const objectProperties: PropEntry[] = properties.properties.attrs.map(
    prop => {
      return { prop, path: ["properties", "__baseline"] };
    },
  );
  const result: RegistryProperty[] = [];

  for (const propEntry of objectProperties) {
    let path = propEntry.path;
    let prop = propEntry.prop;
    path.push(prop.name);

    if (prop.kind() == "link") {
      let cprop = prop as PropLink;
      const realProp = cprop.lookupMyself();

      result.push({
        id: `${entity.id}-${path.join("-")}-${changeSetId}-${
          entity.siStorable.updateClock.epoch
        }-${entity.siStorable.updateClock.updateCount}`,
        name: prop.name,
        label: prop.label,
        path,
        prop: realProp,
        required: prop.required,
        repeated: prop.repeated,
        kind: realProp.kind(),
        hidden: prop.hidden,
      });
      if (realProp.kind() == "object" && prop.repeated == false) {
        const rProp = realProp as PropObject;
        let newProps = rProp.properties.attrs.map(prop => {
          return { prop, path: _.clone(path) };
        });
        for (let nProp of newProps) {
          objectProperties.push(nProp);
        }
      }
    } else {
      if (prop.kind() == "object" && prop.repeated == false) {
        const rProp = prop as PropObject;
        let newProps = rProp.properties.attrs.map(prop => {
          return { prop, path: _.clone(path) };
        });
        for (let nProp of newProps) {
          objectProperties.push(nProp);
        }
      }
      result.push({
        id: `${entity.id}-${path.join("-")}-${changeSetId}-${
          entity.siStorable.updateClock.epoch
        }-${entity.siStorable.updateClock.updateCount}`,
        name: prop.name,
        label: prop.label,
        path,
        prop,
        required: prop.required,
        repeated: prop.repeated,
        kind: prop.kind(),
        hidden: prop.hidden,
      });
    }
  }
  // This groups things according to their nesting, so we can just
  // walk the results and have everything in the proper order.
  const grouped = _.groupBy(result, value => {
    if (value.kind == "object") {
      return value.path;
    } else {
      return value.path.slice(0, -1);
    }
  });
  return _.flatten(Object.values(grouped));
}

export function propertyListRepeated(
  entity: Entity,
  entityProperty: RegistryProperty,
  index: number,
): RegistryProperty[] {
  if (entityProperty.kind == "object") {
    let updateField = entityProperty.prop as PropObject;

    const objectProperties: PropEntry[] = updateField.properties.attrs.map(
      prop => {
        return { prop, path: _.clone(entityProperty.path) };
      },
    );
    const result: RegistryProperty[] = [];

    for (const propEntry of objectProperties) {
      let path = propEntry.path;
      let prop = propEntry.prop;
      path.push(`${index}`);
      path.push(prop.name);

      if (prop.kind() == "link") {
        let cprop = prop as PropLink;
        const realProp = cprop.lookupMyself();

        result.push({
          id: `${entity.id}-${path.join("-")}-${
            entity.siStorable.updateClock.epoch
          }-${entity.siStorable.updateClock.updateCount}`,
          name: prop.name,
          label: prop.label,
          path,
          prop: realProp,
          required: prop.required,
          repeated: prop.repeated,
          kind: realProp.kind(),
          hidden: prop.hidden,
        });
        if (realProp.kind() == "object" && prop.repeated == false) {
          const rProp = realProp as PropObject;
          let newProps = rProp.properties.attrs.map(prop => {
            return { prop, path: _.clone(path) };
          });
          for (let nProp of newProps) {
            objectProperties.push(nProp);
          }
        }
      } else {
        if (prop.kind() == "object" && prop.repeated == false) {
          const rProp = prop as PropObject;
          let newProps = rProp.properties.attrs.map(prop => {
            return { prop, path: _.clone(path) };
          });
          for (let nProp of newProps) {
            objectProperties.push(nProp);
          }
        }
        result.push({
          id: `${entity.id}-${path.join("-")}-${
            entity.siStorable.updateClock.epoch
          }-${entity.siStorable.updateClock.updateCount}`,
          name: prop.name,
          label: prop.label,
          path,
          prop,
          required: prop.required,
          repeated: prop.repeated,
          kind: prop.kind(),
          hidden: prop.hidden,
        });
      }
    }
    // This groups things according to their nesting, so we can just
    // walk the results and have everything in the proper order.
    const grouped = _.groupBy(result, value => {
      if (value.kind == "object") {
        return value.path;
      } else {
        return value.path.slice(0, -1);
      }
    });
    return _.flatten(Object.values(grouped));
  } else {
    let result: RegistryProperty[] = [];
    let path = entityProperty.path;
    path.push(`${index}`);
    result.push({
      id: `${entity.id}-${path.join("-")}-${
        entity.siStorable.updateClock.epoch
      }-${entity.siStorable.updateClock.updateCount}`,
      name: entityProperty.name,
      label: entityProperty.label,
      path,
      prop: entityProperty.prop,
      required: entityProperty.required,
      repeated: entityProperty.repeated,
      kind: entityProperty.kind,
      hidden: entityProperty.hidden,
    });
    return result;
  }
}

export function actionList(entity: Entity): string[] {
  let actions = registry.listActions();
  return actions[entity.objectType] || [];
}

export const registryProperty = {
  propertyList,
  propertyListRepeated,
  actionList,
};
