import _ from "lodash";
import { Prop, RegistryEntry } from "./registryEntry";

// TODO: Eventually, this needs to become a service that serves up the registry entries
// for a given organization/billing account - they should be customizable, etc etc.
import leftHandPath from "./schema/test/leftHandPath";
import noCallbacks from "./schema/test/noCallbacks";
import torture from "./schema/test/torture";
import system from "./schema/si/system";
import service from "./schema/si/service";
import application from "./schema/si/application";
import dockerImage from "./schema/docker/dockerImage";
import kubernetesService from "./schema/kubernetes/kubernetesService";
import k8sDeployment from "./schema/kubernetes/k8sDeployment";
import k8sNamespace from "./schema/kubernetes/k8sNamespace";

export const registry: { [entityType: string]: RegistryEntry } = {
  leftHandPath,
  noCallbacks,
  system,
  service,
  application,
  torture,
  dockerImage,
  kubernetesService,
  k8sDeployment,
  k8sNamespace,
};

export function findProp(path: string[]): Prop | undefined {
  if (path.length == 0) {
    return undefined;
  }
  const registryEntry = registry[path[0]];
  if (!registryEntry) {
    return undefined;
  }
  let properties = registryEntry.properties;
  for (let x = 1; x < path.length; x++) {
    const propName = path[x];
    if (!_.isNaN(_.toNumber(propName))) {
      continue;
    }
    const prop = _.find(properties, ["name", propName]);
    if (x == path.length - 1) {
      return prop;
    }
    if (prop && prop.type == "object") {
      properties = prop.properties;
    } else if (prop && prop.type == "array") {
      // if an array is the second to last path, and the next item
      // is an array index, we should return the current prop.
      if (x == path.length - 2) {
        const lookAheadPropName = path[x + 1];
        if (!_.isNaN(_.toNumber(lookAheadPropName))) {
          return prop;
        }
      }
      if (prop.itemProperty.type == "object") {
        properties = prop.itemProperty.properties;
      } else {
        return undefined;
      }
    } else {
      return undefined;
    }
  }
}
