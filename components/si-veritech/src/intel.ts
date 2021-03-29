import leftHandPath from "./intel/leftHandPath";
import torture from "./intel/torture";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "./controllers/inferProperties";

export interface Intel {
  inferProperties?(request: InferPropertiesRequest): InferPropertiesReply;
}

const intel: Record<string, Intel> = {
  leftHandPath,
  torture,
};

export default intel;
