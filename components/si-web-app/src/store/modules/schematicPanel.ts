import { Module } from "vuex";

import {
  Schematic,
  SchematicKind,
  ISchematicNode,
} from "@/api/sdf/model/schematic";
import {
  IGetApplicationSystemSchematicRequest,
  IGetSchematicReply,
  SchematicDal,
} from "@/api/sdf/dal/schematicDal";

import Bottle from "bottlejs";
import { PartyBus } from "@/api/partyBus";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import { ConnectionCreatedEvent } from "@/api/partyBus/ConnectionCreatedEvent";

import { SchematicNodeSelectedEvent } from "@/api/partyBus/SchematicNodeSelectedEvent";
import { NodeCreatedEvent } from "@/api/partyBus/NodeCreatedEvent";
import { EntitySetNameEvent } from "@/api/partyBus/EntitySetNameEvent";
import { NodeUpdatedEvent } from "@/api/partyBus/NodeUpdatedEvent";
import { EditSessionCancelEvent } from "@/api/partyBus/EditSessionCancelEvent";

import { Cg2dCoordinate } from "@/api/sicg";
import { Edge } from "@/api/sdf/model/edge";
import { schematicSelectedEntityId$ } from "@/observables";

export interface SchematicPanelStore {
  kind: SchematicKind | null;
  schematic: Schematic | null;
  rootObjectId: string | null;
  selectedNode: ISchematicNode | null;
  lastRequest: IGetApplicationSystemSchematicRequest | null;
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
    },
    setLastRequest(state, payload: SchematicPanelStore["lastRequest"]) {
      state.lastRequest = payload;
    },
    setNodePosition(state, payload: SetNodePositionPayload) {
      if (state.schematic) {
        let position = {
          x: String(payload.position.x),
          y: String(payload.position.y),
        };
        if (
          state.schematic.nodes[payload.nodeId].node.positions[payload.context]
        ) {
          state.schematic.nodes[payload.nodeId].node.positions[
            payload.context
          ] = position;
        } else {
          state.schematic.nodes[payload.nodeId].node.positions = {
            [payload.context]: position,
          };
        }
      }
    },
    lastRequestRemoveEditSession(state) {
      if (state.lastRequest?.editSessionId) {
        delete state.lastRequest?.editSessionId;
      }
    },
  },
  actions: {
    async onNodeCreated({ state, dispatch }, event: NodeCreatedEvent) {
      if (state.lastRequest) {
        await dispatch("loadApplicationSystemSchematic", state.lastRequest);
        if (event.node != null && event.sourcePanelId != undefined) {
          let node = this.state.schematicPanel[event.sourcePanelId].schematic
            .nodes[event.node.id];
          await dispatch("nodeSelect", node);
        }
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
      _event: ConnectionCreatedEvent,
    ) {
      if (state.lastRequest) {
        await dispatch("loadApplicationSystemSchematic", state.lastRequest);

        PanelEventBus.$emit("panel-viewport-edge-remove");
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
      let reply = await SchematicDal.getApplicationSystemSchematic(request);
      if (!reply.error) {
        commit("setSchematic", reply.schematic);
        commit("setLastRequest", request);
      }
      return reply;
    },
    async nodeSelect({ commit }, schematicNode: ISchematicNode) {
      commit("setSelectedNode", schematicNode);
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
  },
};
