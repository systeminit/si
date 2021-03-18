import { Module } from "vuex";
import { EntityObject } from "si-registry/lib/systemComponent";
import {
  INodeCreateReply,
  INodeUpdatePositionReply,
  IEntitySetPropertyRequest,
  IEntitySetPropertyReply,
  IEntitySetNameRequest,
  IEntitySetNameReply,
  IEntitySetPropertyBulkRequest,
  IEntitySetPropertyBulkReply,
} from "@/api/sdf/dal/editorDal";

import {
  ConnectionCreateReply,
  SchematicDal,
  Connection,
  ConnectionNodeReference,
} from "@/api/sdf/dal/schematicDal";

import { PanelEventBus } from "@/atoms/PanelEventBus";
import { SessionStore } from "@/store/modules/session";
import { ChangeSet } from "@/api/sdf/model/changeSet";
import { PartyBus } from "@/api/partyBus";
import { EditSession } from "@/api/sdf/model/editSession";
import Bottle from "bottlejs";
import { CurrentChangeSetEvent } from "@/api/partyBus/currentChangeSetEvent";
import { EditSessionCurrentSetEvent } from "@/api/partyBus/editSessionCurrentSetEvent";
import { EditorDal } from "@/api/sdf/dal/editorDal";
import { NodeKind } from "@/api/sdf/model/node";
import { NodeCreatedEvent } from "@/api/partyBus/NodeCreatedEvent";
import { NodeUpdatedEvent } from "@/api/partyBus/NodeUpdatedEvent";
import { ConnectionCreatedEvent } from "@/api/partyBus/ConnectionCreatedEvent";
import { EntityPropertySetEvent } from "@/api/partyBus/EntityPropertySetEvent";
import { EntitySetNameEvent } from "@/api/partyBus/EntitySetNameEvent";

import { Cg2dCoordinate } from "@/api/sicg";

export type IEditorContext = IEditorContextApplication;

export interface IEditorContextApplication {
  applicationId: string;
  contextType: "applicationSystem";
}

export interface EditorStore {
  version: number;
  context: IEditorContext | null;
  currentChangeSet: ChangeSet | null;
  currentEditSession: EditSession | null;
}

export interface ConnectionCreatePayload {
  connection: Connection;
}

export interface NodeCreatePayload {
  entityObject: EntityObject;
  sourcePanelId: string;
}

export interface NodeUpdatePositionePayload {
  nodeId: string;
  contextId: string;
  position: Cg2dCoordinate;
}

export function setupEditor() {
  const bottle = Bottle.pop("default");
  const partyBus: PartyBus = bottle.container.PartyBus;
  partyBus.subscribeToEvents("editor", undefined, [
    CurrentChangeSetEvent,
    EditSessionCurrentSetEvent,
  ]);
}

export interface TransientEdgeRemovalEvent {
  remove: boolean;
}

export const editor: Module<EditorStore, any> = {
  namespaced: true,
  state: {
    version: 1,
    currentChangeSet: null,
    currentEditSession: null,
    context: null,
  },
  getters: {
    inEditable(state): boolean {
      if (state.currentChangeSet && state.currentEditSession && state.context) {
        return true;
      } else {
        return false;
      }
    },
  },
  mutations: {
    setCurrentChangeSet(state, payload: EditorStore["currentChangeSet"]) {
      state.currentChangeSet = payload;
    },
    setCurrentEditSession(state, payload: EditorStore["currentEditSession"]) {
      state.currentEditSession = payload;
    },
    setContext(state, payload: EditorStore["context"]) {
      state.context = payload;
    },
  },
  actions: {
    async onCurrentChangeSet({ commit }, event: CurrentChangeSetEvent) {
      commit("setCurrentChangeSet", event.changeSet);
    },
    async onEditSessionCurrentSet(
      { commit },
      event: EditSessionCurrentSetEvent,
    ) {
      commit("setCurrentEditSession", event.editSession);
    },
    async setContext({ commit }, context: EditorStore["context"]) {
      commit("setContext", context);
    },
    async entitySetName(
      {},
      request: IEntitySetNameRequest,
    ): Promise<IEntitySetNameReply> {
      let reply = await EditorDal.entitySetName(request);
      if (!reply.error) {
        new EntitySetNameEvent({
          entity: reply.object,
          entitySetNameRequest: request,
        }).publish();
      }
      return reply;
    },
    async entitySetProperty(
      {},
      request: IEntitySetPropertyRequest,
    ): Promise<IEntitySetPropertyReply> {
      request.path = request.path.slice(2);
      let reply = await EditorDal.entitySetProperty(request);
      if (!reply.error) {
        console.log("set it", {
          entity: reply.object,
          entitySetPropertyRequest: request,
        });
        new EntityPropertySetEvent({
          entity: reply.object,
          entitySetPropertyRequest: request,
        }).publish();
      } else {
        console.log("not sending entity property set event", { reply });
      }
      return reply;
    },
    async entitySetPropertyBulk(
      {},
      request: IEntitySetPropertyBulkRequest,
    ): Promise<IEntitySetPropertyBulkReply> {
      for (let property of request.properties) {
        property.path = property.path.slice(2);
      }
      let reply = await EditorDal.entitySetPropertyBulk(request);
      if (!reply.error) {
        for (let property of request.properties) {
          let entitySetPropertyRequest: IEntitySetPropertyRequest = {
            ...request,
            ...property,
          };
          new EntityPropertySetEvent({
            entity: reply.object,
            entitySetPropertyRequest,
          }).publish();
        }
      } else {
        console.log("not sending entity property set bulk event", { reply });
      }
      return reply;
    },

    async nodeCreate(
      { state, rootState },
      payload: NodeCreatePayload,
    ): Promise<INodeCreateReply> {
      let currentWorkspace: SessionStore["currentWorkspace"] =
        rootState.session.currentWorkspace;
      let currentSystem: SessionStore["currentSystem"] =
        rootState.session.currentSystem;
      if (
        !currentWorkspace ||
        !currentSystem ||
        !state.currentEditSession ||
        !state.currentChangeSet ||
        !state.context
      ) {
        throw new Error(
          "Cannot call nodeCreate without a workspace, system, changeSet and editSession or EditContext! bug!",
        );
      }
      let reply: INodeCreateReply;
      if (state.context.applicationId) {
        reply = await EditorDal.nodeCreateForApplication({
          kind: NodeKind.Entity,
          objectType: payload.entityObject.typeName,
          workspaceId: currentWorkspace.id,
          changeSetId: state.currentChangeSet.id,
          editSessionId: state.currentEditSession.id,
          systemId: currentSystem.id,
          applicationId: state.context.applicationId,
        });
      } else {
        throw new Error("cannot create without an editor context");
      }
      if (!reply.error) {
        // reply.creatorId = "me!"
        new NodeCreatedEvent(reply, payload.sourcePanelId).publish();
      }

      return reply;
    },

    async connectionCreate(
      { state, rootState },
      payload: ConnectionCreatePayload,
    ): Promise<ConnectionCreateReply> {
      let currentWorkspace: SessionStore["currentWorkspace"] =
        rootState.session.currentWorkspace;
      let currentSystem: SessionStore["currentSystem"] =
        rootState.session.currentSystem;
      if (
        !currentWorkspace ||
        !currentSystem ||
        !state.currentEditSession ||
        !state.currentChangeSet ||
        !state.context
      ) {
        throw new Error(
          "Cannot call nodeCreate without a workspace, system, changeSet and editSession or EditContext! bug!",
        );
      }

      let reply: ConnectionCreateReply;
      if (state.context.applicationId) {
        reply = await SchematicDal.connectionCreate({
          connection: payload.connection,
          workspaceId: currentWorkspace.id,
          changeSetId: state.currentChangeSet.id,
          editSessionId: state.currentEditSession.id,
          applicationId: state.context.applicationId,
        });
      } else {
        throw new Error("cannot create without an editor context");
      }
      if (!reply.error) {
        new ConnectionCreatedEvent(reply).publish();
      } else {
      }
      return reply;
    },

    async nodeSetPosition(
      { state, rootState },
      payload: NodeUpdatePositionePayload,
    ): Promise<INodeUpdatePositionReply> {
      let currentWorkspace: SessionStore["currentWorkspace"] =
        rootState.session.currentWorkspace;
      if (!currentWorkspace || !state.context) {
        throw new Error(
          "Cannot call nodeCreate without a workspace, system, changeSet and editSession or EditContext! bug!",
        );
      }
      let reply: INodeUpdatePositionReply;
      if (state.context.applicationId) {
        reply = await EditorDal.nodeUpdatePosition({
          nodeId: payload.nodeId,
          contextId: payload.contextId,
          x: String(payload.position.x),
          y: String(payload.position.y),
          workspaceId: currentWorkspace.id,
        });
      } else {
        throw new Error("cannot create without an editor context");
      }
      if (!reply.error) {
        // @ts-ignore
        new NodeUpdatedEvent(reply).publish();
      }
      return reply;
    },
  },
};
