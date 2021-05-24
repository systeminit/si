import { OpSource } from "si-entity/dist/siEntity";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import { setPropertyFromEntity } from "./inferShared";

function inferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const entity = request.entity;
  const context = request.context;

  entity.set({
    source: OpSource.Inferred,
    system: "baseline",
    path: ["name"],
    value: request.entity.name,
  });

  setPropertyFromEntity({
    context,
    entityType: "awsLocation",
    fromPath: ["location"],
    toEntity: entity,
    toPath: ["location"],
  });

  return { entity: request.entity };
}

export default { inferProperties };
