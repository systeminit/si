import leftHandPath from "./intel/leftHandPath";
import torture from "./intel/torture";
import dockerImage, { CheckQualificationCallbacks } from "./intel/dockerImage";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "./controllers/inferProperties";

export interface Intel {
  inferProperties?(request: InferPropertiesRequest): InferPropertiesReply;
  checkQualifications?: CheckQualificationCallbacks;
}

const intel: Record<string, Intel> = {
  leftHandPath,
  torture,
  dockerImage,
};

export default intel;
