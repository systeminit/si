import leftHandPath from "./intel/leftHandPath";
import torture from "./intel/torture";
import dockerImage, { CheckQualificationCallbacks } from "./intel/dockerImage";
import k8sDeployment from "./intel/k8sDeployment";
import k8sNamespace from "./intel/k8sNamespace";
import k8sService from "./intel/k8sService";
import awsEksCluster from "./intel/awsEksCluster";
import azureResourceGroup from "./intel/azureResourceGroup";
import azureAksCluster from "./intel/azureAksCluster";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "./controllers/inferProperties";
import { RunCommandCallbacks } from "./controllers/runCommand";

export interface Intel {
  inferProperties?(request: InferPropertiesRequest): InferPropertiesReply;
  checkQualifications?: CheckQualificationCallbacks;
  runCommands?: RunCommandCallbacks;
}

const intel: Record<string, Intel> = {
  leftHandPath,
  torture,
  dockerImage,
  k8sDeployment,
  k8sNamespace,
  k8sService,
  awsEksCluster,
  azureResourceGroup,
  azureAksCluster,
};

export default intel;
