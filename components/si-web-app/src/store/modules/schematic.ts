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

export interface SchematicStore {
  kind: SchematicKind | null;
  schematic: Schematic | null;
  rootObjectId: string | null;
  selectedNode: ISchematicNode | null;
}

export const schematicStore: Module<SchematicStore, any> = {
  namespaced: true,
  state(): SchematicStore {
    return {
      kind: null,
      schematic: null,
      rootObjectId: null,
      selectedNode: null,
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
  },
  actions: {
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
      }
      return reply;
    },
    async nodeSelect({ commit }, schematicNode: ISchematicNode) {
      commit("selectedNode", schematicNode);
      new SchematicNodeSelectedEvent({ schematicNode: schematicNode });
    },
  },
};
