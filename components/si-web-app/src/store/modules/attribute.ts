import { Module } from "vuex";
import {
  IGetObjectListRequest,
  IGetObjectListReply,
  AttributeDal,
  IGetEntityRequest,
  IGetEntityReply,
} from "@/api/sdf/dal/attributeDal";
import { Entity } from "@/api/sdf/model/entity";

export interface AttributeStore {
  objectList: { value: string; label: string }[] | null;
  currentObject: Entity | null;
}

export const attributeStore: Module<AttributeStore, any> = {
  namespaced: true,
  state(): AttributeStore {
    return {
      objectList: null,
      currentObject: null,
    };
  },
  mutations: {
    setObjectList(state, payload: AttributeStore["objectList"]) {
      state.objectList = payload;
    },
    setCurrentObject(state, payload: AttributeStore["currentObject"]) {
      state.currentObject = payload;
    },
  },
  actions: {
    async loadObjectList(
      { commit },
      request: IGetObjectListRequest,
    ): Promise<IGetObjectListReply> {
      let reply = await AttributeDal.getObjectList(request);
      if (!reply.error) {
        reply.objectList.unshift({ label: "", value: "" });
        commit("setObjectList", reply.objectList);
      }
      return reply;
    },
    async loadEntity(
      { commit },
      request: IGetEntityRequest,
    ): Promise<IGetEntityReply> {
      let reply = await AttributeDal.getEntity(request);
      if (!reply.error) {
        commit("setCurrentObject", reply.entity);
      }
      return reply;
    },
    clearObject({ commit }) {
      commit("setCurrentObject", null);
    },
  },
};
