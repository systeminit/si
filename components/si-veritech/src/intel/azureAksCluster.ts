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
    path: ["name"],
    value: request.entity.name,
  });

  return { entity: request.entity };
}

export default { inferProperties };
