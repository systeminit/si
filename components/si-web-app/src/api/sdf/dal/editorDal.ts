import { NodeKind, Node } from "@/api/sdf/model/node";
import { Entity } from "@/api/sdf/model/entity";
import { SDFError } from "@/api/sdf";
import { System } from "@/api/sdf/model/system";
import Bottle from "bottlejs";
import { NodeCreatedEvent } from "@/api/partyBus/NodeCreatedEvent";

export interface INodeCreateForApplicationRequest extends INodeCreateRequest {
  applicationId: string;
}

export interface INodeCreateRequest {
  name?: string;
  kind: NodeKind;
  objectType: string;
  workspaceId: string;
  changeSetId: string;
  editSessionId: string;
  systemId: string;
}

export interface INodeObjectEntity {
  entity: Entity;
  system?: never;
}

export interface INodeObjectSystem {
  entity?: never;
  system: System;
}

export type INodeObject = INodeObjectEntity | INodeObjectSystem;

export interface INodeCreateReplySuccess {
  node: Node;
  object: INodeObject;
  error?: never;
}

export interface INodeCreateReplyFailure {
  node?: never;
  object?: never;
  error: SDFError;
}

export type INodeCreateReply =
  | INodeCreateReplySuccess
  | INodeCreateReplyFailure;

async function nodeCreateForApplication(
  request: INodeCreateForApplicationRequest,
): Promise<INodeCreateReply> {
  let bottle = Bottle.pop("default");
  let sdf = bottle.container.SDF;

  const reply: INodeCreateReply = await sdf.post(
    "editorDal/nodeCreateForApplication",
    request,
  );

  if (!reply.error) {
    new NodeCreatedEvent(reply).publish();
  }

  return reply;
}

export const EditorDal = {
  nodeCreateForApplication,
};
