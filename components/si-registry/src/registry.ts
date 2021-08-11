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
import k8sIngress from "./schema/kubernetes/k8sIngress";
import k8sNamespace from "./schema/kubernetes/k8sNamespace";
import k8sService from "./schema/kubernetes/k8sService";
import k8sSecret from "./schema/kubernetes/k8sSecret";
import kubernetesCluster from "./schema/kubernetes/kubernetes";
import awsEks from "./schema/aws/awsEks";
import cloudProvider from "./schema/si/cloudProvider";
import aws from "./schema/aws/aws";
import awsRegion from "./schema/aws/awsRegion";
import awsAccessKey from "./schema/aws/awsAccessKey";
import awsEksCluster from "./schema/aws/awsEksCluster";
import azure from "./schema/azure/azure";
import azureAks from "./schema/azure/azureAks";
import azureLocation from "./schema/azure/azureLocation";
import azureAksCluster from "./schema/azure/azureAksCluster";
import azureServicePrincipal from "./schema/azure/azureServicePrincipal";
import azureResourceGroup from "./schema/azure/azureResourceGroup";

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
  k8sIngress,
  k8sNamespace,
  k8sSecret,
  k8sService,
  kubernetesCluster,
  awsEks,
  cloudProvider,
  aws,
  awsRegion,
  awsAccessKey,
  awsEksCluster,
  azure,
  azureAks,
  azureAksCluster,
  azureLocation,
  azureServicePrincipal,
  azureResourceGroup,
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
