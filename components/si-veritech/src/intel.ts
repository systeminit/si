import leftHandPath from "./intel/leftHandPath";
import torture from "./intel/torture";
import dockerImage, { CheckQualificationCallbacks } from "./intel/dockerImage";
import k8sDeployment from "./intel/k8sDeployment";
import k8sNamespace from "./intel/k8sNamespace";
import awsEksCluster from "./intel/awsEksCluster";
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
  awsEksCluster,
};

export default intel;
