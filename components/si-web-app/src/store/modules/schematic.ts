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
import { SchematicNodeSelectedEvent } from "@/api/partyBus/SchematicNodeSelectedEvent";
import { NodeCreatedEvent } from "@/api/partyBus/NodeCreatedEvent";

export interface SchematicStore {
  kind: SchematicKind | null;
  schematic: Schematic | null;
  rootObjectId: string | null;
  selectedNode: ISchematicNode | null;
  lastRequest: IGetApplicationSystemSchematicRequest | null;
}

export const schematicStoreSubscribeEvents = [NodeCreatedEvent];

export const schematicStore: Module<SchematicStore, any> = {
  namespaced: true,
  state(): SchematicStore {
    return {
      kind: null,
      schematic: null,
      rootObjectId: null,
      selectedNode: null,
      lastRequest: null,
    };
  },
  mutations: {
    setRootObjectId(state, payload: SchematicStore["rootObjectId"]) {
      state.rootObjectId = payload;
    },
    setSchematic(state, payload: SchematicStore["schematic"]) {
      state.schematic = payload;
    },
    setSelectedNode(state, payload: SchematicStore["selectedNode"]) {
      state.selectedNode = payload;
    },
    setLastRequest(state, payload: SchematicStore["lastRequest"]) {
      state.lastRequest = payload;
    },
  },
  actions: {
    async onNodeCreated({ state, dispatch }, _event: NodeCreatedEvent) {
      if (state.lastRequest) {
        await dispatch("loadApplicationSystemSchematic", state.lastRequest);
      }
    },
    setRootObjectId({ commit }, payload: SchematicStore["rootObjectId"]) {
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
      commit("selectedNode", schematicNode);
      new SchematicNodeSelectedEvent({ schematicNode: schematicNode });
    },
  },
};
