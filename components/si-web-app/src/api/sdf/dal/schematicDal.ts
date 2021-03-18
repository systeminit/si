import { EdgeKind, Edge } from "@/api/sdf/model/edge";
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

/*
 * Nodes Connection
 */

export type ConnectionKind = EdgeKind;

export interface ConnectionNodeReference {
  nodeId: string;
  socketId: string;
  nodeKind: string;
}

export interface Connection {
  kind: String;
  source: ConnectionNodeReference;
  destination: ConnectionNodeReference;
  systemId: String;
}

export interface ConnectionCreateRequest {
  connection: Connection;
  workspaceId: string;
  changeSetId: string;
  editSessionId: string;
  applicationId: string;
}

export interface ConnectionCreateReplySuccess {
  edge: Edge; // We don't need this...
  error?: never;
}

export interface ConnectionCreateReplyFailure {
  edge?: Edge; // We don't need this...
  error: SDFError;
}

export type ConnectionCreateReply =
  | ConnectionCreateReplySuccess
  | ConnectionCreateReplyFailure;

async function connectionCreate(
  request: ConnectionCreateRequest,
): Promise<ConnectionCreateReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: ConnectionCreateReply = await sdf.post(
    "schematicDal/connectionCreate",
    request,
  );
  return reply;
}

export const SchematicDal = {
  getApplicationSystemSchematic,
  connectionCreate,
};
