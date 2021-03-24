import { OpSource } from "si-entity/dist/siEntity";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";

function inferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const entity = request.entity;

  entity.set({
    source: OpSource.Inferred,
    system: "baseline",
    path: ["allLayers"],
    value: "a",
  });
  entity.set({
    source: OpSource.Manual,
    system: "baseline",
    path: ["allLayers"],
    value: "b",
  });

  return { entity: request.entity };
}

export default { inferProperties };
