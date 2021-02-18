import { SDFError } from "@/api/sdf";
import Bottle from "bottlejs";
import { Schematic } from "@/api/sdf/model/schematic";

export interface IGetSchematicRequest {
  workspaceId: string;
  rootObjectId: string;
  changeSetId?: string;
}

export interface IGetSchematicReplySuccess {
  schematic: Schematic;
  error?: never;
}

export interface IGetSchematicReplyFailure {
  schematic?: never;
  error: SDFError;
}

export type IGetSchematicReply =
  | IGetSchematicReplySuccess
  | IGetSchematicReplyFailure;

export interface IGetApplicationSystemSchematicRequest
  extends IGetSchematicRequest {
  systemId: string;
}

export async function getApplicationSystemSchematic(
  request: IGetApplicationSystemSchematicRequest,
): Promise<IGetSchematicReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: IGetSchematicReply = await sdf.get(
    "schematicDal/getApplicationSystemSchematic",
    request,
  );
  return reply;
}

export const SchematicDal = {
  getApplicationSystemSchematic,
};
