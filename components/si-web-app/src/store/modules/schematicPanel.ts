import { Module } from "vuex";

import {
  Schematic,
  SchematicKind,
  ISchematicNode,
} from "@/api/sdf/model/schematic";

import {
  IGetApplicationSystemSchematicRequest,
  IGetSchematicReply,
  INodeCreateReply,
  SchematicDal,
  Connection,
  INodeUpdatePositionReply,
  ConnectionCreateReply,
} from "@/api/sdf/dal/schematicDal";

import { ConnectionCreatedEvent } from "@/api/partyBus/ConnectionCreatedEvent";
import { SchematicNodeSelectedEvent } from "@/api/partyBus/SchematicNodeSelectedEvent";
import { NodeCreatedEvent } from "@/api/partyBus/NodeCreatedEvent";
import { EntitySetNameEvent } from "@/api/partyBus/EntitySetNameEvent";
import { NodeUpdatedEvent } from "@/api/partyBus/NodeUpdatedEvent";
import { EditSessionCancelEvent } from "@/api/partyBus/EditSessionCancelEvent";

import { Cg2dCoordinate } from "@/api/sicg";
import { Edge } from "@/api/sdf/model/edge";
import { schematicSelectedEntityId$ } from "@/observables";

import _ from "lodash";

export type IEditorContext = IEditorContextApplication;

export interface TransientEdgeRemovalEvent {
  remove: boolean;
}
export interface IEditorContextApplication {
  applicationId: string;
  contextType: "applicationSystem";
}

export interface ConnectionCreatePayload {
  connection: Connection;
  workspaceId: string;
  changeSetId: string;
  editSessionId: string;
  applicationId: string;
}

export interface SchematicPanelStore {
  kind: SchematicKind | null;
  schematic: Schematic | null;
  rootObjectId: string | null;
  selectedNode: ISchematicNode | null;
  selectedNodeId: string | null;
  lastRequest: IGetApplicationSystemSchematicRequest | null;
}

export interface NodeCreatePayload {
  entityType: string;
  sourcePanelId: string;
  applicationId: string;
  workspaceId: string;
  changeSetId: string;
  editSessionId: string;
}

export interface NodeUpdatePositionePayload {
  nodeId: string;
  contextId: string;
  position: Cg2dCoordinate;
  workspaceId: string;
  applicationId: string;
}

export interface SetNodePositionPayload {
  nodeId: string;
  context: string;
  position: Cg2dCoordinate;
}

export interface NodeSelectWithIdPayload {
  nodeId: string;
  storeInstanceId: string;
}

export interface updateEdgeListPayload {
  edgeId: string;
  edge: Edge;
}

export const schematicPanelStoreSubscribeEvents = [
  NodeCreatedEvent,
  EntitySetNameEvent,
  ConnectionCreatedEvent,
  EditSessionCancelEvent,
];

export const schematicPanelStore: Module<SchematicPanelStore, any> = {
  namespaced: true,
  state(): SchematicPanelStore {
    return {
      kind: null,
      schematic: null,
      rootObjectId: null,
      selectedNode: null,
      selectedNodeId: null,
      lastRequest: null,
    };
  },
  mutations: {
    setRootObjectId(state, payload: SchematicPanelStore["rootObjectId"]) {
      state.rootObjectId = payload;
    },
    setSchematic(state, payload: SchematicPanelStore["schematic"]) {
      state.schematic = payload;
    },
    setSelectedNode(state, payload: SchematicPanelStore["selectedNode"]) {
      if (payload) {
        schematicSelectedEntityId$.next(payload.object.id);
      } else {
        schematicSelectedEntityId$.next("");
      }
      state.selectedNode = payload;
      if (payload && payload.node.id) {
        state.selectedNodeId = payload.node.id;
      }
    },
    setLastRequest(state, payload: SchematicPanelStore["lastRequest"]) {
      state.lastRequest = payload;
    },
    setNodePosition(state, payload: SetNodePositionPayload) {
      if (state.schematic) {
        const position = {
          x: String(payload.position.x),
          y: String(payload.position.y),
        };

        if (
          state.schematic &&
          state.schematic.nodes[payload.nodeId] &&
          state.schematic.nodes[payload.nodeId].node &&
          state.schematic.nodes[payload.nodeId].node.positions[payload.context]
        ) {
          state.schematic.nodes[payload.nodeId].node.positions[
            payload.context
          ] = position;
        } else {
          if (
            state.schematic &&
            state.schematic.nodes[payload.nodeId] &&
            state.schematic.nodes[payload.nodeId].node
          ) {
            state.schematic.nodes[payload.nodeId].node.positions = {
              [payload.context]: position,
            };
          }
        }
      }
    },
    addNode(state, node: ISchematicNode) {
      if (state.schematic) {
        state.schematic.nodes[node.node.id] = node;
      }
    },
    lastRequestRemoveEditSession(state) {
      if (state.lastRequest?.editSessionId) {
        delete state.lastRequest?.editSessionId;
      }
    },
  },
  actions: {
    async nodeCreate(
      { commit, dispatch, state },
      payload: NodeCreatePayload,
    ): Promise<INodeCreateReply> {
      if (
        !payload.applicationId ||
        !payload.workspaceId ||
        !payload.changeSetId ||
        !payload.editSessionId
      ) {
        throw new Error(
          "Cannot call nodeCreate without a workspace, system, changeSet and editSession or EditContext! bug!",
        );
      }
      let reply: INodeCreateReply;
      reply = await SchematicDal.nodeCreateForApplication({
        entityType: payload.entityType,
        workspaceId: payload.workspaceId,
        changeSetId: payload.changeSetId,
        editSessionId: payload.editSessionId,
        applicationId: payload.applicationId,
        returnSchematic: true,
      });
      if (!reply.error) {
        if (reply.schematic) {
          await commit("setSelectedNode", reply.node);
          await commit("setSchematic", reply.schematic);
        } else if (!reply.schematic && reply.node) {
          await commit("addNode", reply.node);
          await commit("setSelectedNode", reply.node);
        }

        new SchematicNodeSelectedEvent({
          schematicNode: reply.node,
        }).publish();

        new NodeCreatedEvent(
          _.cloneDeep(reply),
          payload.sourcePanelId,
        ).publish();
      }
      return reply;
    },
    async onNodeCreated({ state, dispatch }, event: NodeCreatedEvent) {
      if (state.lastRequest) {
        await dispatch("setApplicationSystemSchematic", event.schematic);
      }
    },
    async onNodeUpdated({ state, dispatch }, _event: NodeUpdatedEvent) {
      if (state.lastRequest) {
        await dispatch("loadApplicationSystemSchematic", state.lastRequest);
      }
    },
    async onEditSessionCancel(
      { state, dispatch, commit },
      _event: EditSessionCancelEvent,
    ) {
      if (state.lastRequest) {
        commit("lastRequestRemoveEditSession");
        await dispatch("loadApplicationSystemSchematic", state.lastRequest);
      }
    },
    async onConnectionCreated(
      { state, dispatch },
      event: ConnectionCreatedEvent,
    ) {
      if (state.lastRequest) {
        await dispatch("setApplicationSystemSchematic", event.schematic);
      }
    },
    async onEntitySetName({ state, dispatch }, _event: NodeCreatedEvent) {
      if (state.lastRequest) {
        await dispatch("loadApplicationSystemSchematic", state.lastRequest);
      }
    },
    setRootObjectId({ commit }, payload: SchematicPanelStore["rootObjectId"]) {
      commit("setRootObjectId", payload);
    },
    async loadApplicationSystemSchematic(
      { commit },
      request: IGetApplicationSystemSchematicRequest,
    ): Promise<IGetSchematicReply> {
      const reply = await SchematicDal.getApplicationSystemSchematic(request);
      if (!reply.error) {
        await commit("setSchematic", reply.schematic);
        await commit("setLastRequest", request);
      }
      return reply;
    },
    async setApplicationSystemSchematic({ commit }, schematic: Schematic) {
      await commit("setSchematic", _.cloneDeep(schematic));
    },
    async nodeSelect({ commit }, schematicNode: ISchematicNode) {
      await commit("setSelectedNode", schematicNode);
      new SchematicNodeSelectedEvent({
        schematicNode: schematicNode,
      }).publish();
    },
    async nodeSelectionClear({ commit }) {
      commit("setSelectedNode", null);
    },
    async setNodePosition({ commit }, payload: SetNodePositionPayload) {
      commit("setNodePosition", payload);
    },
    async nodeSetPosition(
      { state, rootState },
      payload: NodeUpdatePositionePayload,
    ): Promise<INodeUpdatePositionReply> {
      let reply: INodeUpdatePositionReply;
      if (payload.applicationId) {
        reply = await SchematicDal.nodeUpdatePosition({
          nodeId: payload.nodeId,
          contextId: payload.contextId,
          x: String(payload.position.x),
          y: String(payload.position.y),
          workspaceId: payload.workspaceId,
        });
      } else {
        throw new Error("cannot set node position without: applicationId");
      }
      if (!reply.error) {
        // @ts-ignore
        new NodeUpdatedEvent(reply).publish();
      }
      return reply;
    },
    async connectionCreate(
      { state, rootState },
      payload: ConnectionCreatePayload,
    ): Promise<ConnectionCreateReply> {
      let reply: ConnectionCreateReply;
      if (payload.applicationId) {
        reply = await SchematicDal.connectionCreate({
          connection: payload.connection,
          workspaceId: payload.workspaceId,
          changeSetId: payload.changeSetId,
          editSessionId: payload.editSessionId,
          applicationId: payload.applicationId,
          returnSchematic: true,
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
  },
};
