import leftHandPath from "./intel/leftHandPath";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "./controllers/inferProperties";

export interface Intel {
  inferProperties?(request: InferPropertiesRequest): InferPropertiesReply;
}

const intel: Record<string, Intel> = {
  leftHandPath,
};

export default intel;
