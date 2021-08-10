import leftHandPath from "./intel/leftHandPath";
import torture from "./intel/torture";
import dockerImage, { CheckQualificationCallbacks } from "./intel/dockerImage";
import k8sDeployment from "./intel/k8sDeployment";
import k8sIngress from "./intel/k8sIngress";
import k8sNamespace from "./intel/k8sNamespace";
import k8sSecret from "./intel/k8sSecret";
import k8sService from "./intel/k8sService";
import awsEksCluster from "./intel/awsEksCluster";
import azureResourceGroup from "./intel/azureResourceGroup";
import azureAksCluster from "./intel/azureAksCluster";
import azureServicePrincipal from "./intel/azureServicePrincipal";
import azure from "./intel/azure";
import aws from "./intel/aws";
import awsEks from "./intel/awsEks";
import azureAks from "./intel/azureAks";
import cloudProvider from "./intel/cloudProvider";
import kubernetesCluster from "./intel/kubernetesCluster";
import kubernetesService from "./intel/kubernetesService";
import service from "./intel/service";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "./controllers/inferProperties";
import { RunCommandCallbacks } from "./controllers/runCommand";
import { SyncResourceCallback } from "./controllers/syncResource";
import { DiscoveryCallback } from "./controllers/discover";

export interface Intel {
  inferProperties?(request: InferPropertiesRequest): InferPropertiesReply;
  checkQualifications?: CheckQualificationCallbacks;
  runCommands?: RunCommandCallbacks;
  syncResource?: SyncResourceCallback;
  discover?: DiscoveryCallback;
}

const intel: Record<string, Intel> = {
  leftHandPath,
  torture,
  dockerImage,
  k8sDeployment,
  k8sIngress,
  k8sNamespace,
  k8sSecret,
  k8sService,
  awsEksCluster,
  azureResourceGroup,
  azureAksCluster,
  azureServicePrincipal,
  azure,
  aws,
  cloudProvider,
  awsEks,
  azureAks,
  kubernetesCluster,
  kubernetesService,
  service,
};

export default intel;
