import _ from "lodash";

import { PropLink, PropObject } from "./components/prelude";

import { registry } from "./registry";

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

export function valiatePropertyList(registryObject: any): void {
  const properties = registryObject.fields.getEntry("properties") as PropObject;
  const objectProperties: PropEntry[] = properties.properties.attrs.map(
    prop => {
      return { prop, path: [] };
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
}
