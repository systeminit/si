import { Module } from "vuex";
import {
  IGetObjectListRequest,
  IGetObjectListReply,
  AttributeDal,
  IGetEntityRequest,
  IGetEntityReply,
  IGetEntityReplySuccess,
} from "@/api/sdf/dal/attributeDal";
import { Entity } from "@/api/sdf/model/entity";
import { SchematicNodeSelectedEvent } from "@/api/partyBus/SchematicNodeSelectedEvent";
import { EntitySetNameEvent } from "@/api/partyBus/EntitySetNameEvent";
import { EntityPropertySetEvent } from "@/api/partyBus/EntityPropertySetEvent";
import { diffEntity, DiffResult } from "@/api/diff";
import _ from "lodash";
import Vue from "vue";
import { PanelEventBus, emitPropChangeEvent } from "@/atoms/PanelEventBus";

export const attributeStoreSubscribeEvents = [
  SchematicNodeSelectedEvent,
  EntitySetNameEvent,
  EntityPropertySetEvent,
];

export interface AttributeStore {
  lastObjectListRequest: IGetObjectListRequest | null;
  lastLoadEntityRequest: IGetEntityRequest | null;
  objectList: { value: string; label: string }[] | null;
  currentObject: Entity | null;
  baseObject: Entity | null;
  selectionLocked: boolean;
  needsRefresh: boolean;
}

export const attributeStore: Module<AttributeStore, any> = {
  namespaced: true,
  state(): AttributeStore {
    return {
      objectList: null,
      currentObject: null,
      baseObject: null,
      selectionLocked: true,
      lastObjectListRequest: null,
      lastLoadEntityRequest: null,
      needsRefresh: false,
    };
  },
  mutations: {
    setObjectList(state, payload: AttributeStore["objectList"]) {
      state.objectList = payload;
    },
    setCurrentObject(state, payload: AttributeStore["currentObject"]) {
      state.currentObject = payload;
    },
    setSelectionLocked(state, payload: AttributeStore["selectionLocked"]) {
      state.selectionLocked = payload;
    },
    setLastObjectListRequest(
      state,
      payload: AttributeStore["lastObjectListRequest"],
    ) {
      state.lastObjectListRequest = payload;
    },
    setLastLoadEntityRequest(
      state,
      payload: AttributeStore["lastLoadEntityRequest"],
    ) {
      state.lastLoadEntityRequest = payload;
    },
    setNeedsRefresh(state, payload: AttributeStore["needsRefresh"]) {
      state.needsRefresh = payload;
    },
    updateFromDiff(state, payload: DiffResult) {
      if (state.currentObject) {
        for (let diff of payload.entries) {
          _.setWith(
            state.currentObject,
            diff.path,
            diff.after,
            (nsValue, _key, _nsObject) => {
              if (nsValue?.__ob__) {
                return nsValue;
              } else {
                return Vue.observable(nsValue);
              }
            },
          );
        }
      }
    },
    setFromLoadEntity(
      state,
      payload: { reply: IGetEntityReplySuccess; request: IGetEntityRequest },
    ) {
      state.lastLoadEntityRequest = payload.request;
      state.currentObject = payload.reply.entity;
      state.baseObject = payload.reply.baseEntity;
    },
  },
  actions: {
    toggleSelectionLocked({ commit, state }): boolean {
      if (state.selectionLocked) {
        commit("setSelectionLocked", false);
        return false;
      } else {
        commit("setSelectionLocked", true);
        return true;
      }
    },
    async loadObjectList(
      { commit },
      request: IGetObjectListRequest,
    ): Promise<IGetObjectListReply> {
      let reply = await AttributeDal.getObjectList(request);
      if (!reply.error) {
        reply.objectList.unshift({ label: "", value: "" });
        commit("setObjectList", reply.objectList);
        commit("setLastObjectListRequest", request);
      }
      return reply;
    },
    async loadEntity(
      { commit },
      request: IGetEntityRequest,
    ): Promise<IGetEntityReply> {
      let reply = await AttributeDal.getEntity(request);
      if (!reply.error) {
        commit("setFromLoadEntity", { reply, request });
      }
      return reply;
    },
    async refreshEntity({ dispatch, state }): Promise<IGetEntityReply> {
      return await dispatch("loadEntity", state.lastLoadEntityRequest);
    },
    clearObject({ commit }) {
      commit("setCurrentObject", null);
    },
    async onSchematicNodeSelected(
      { dispatch, state, rootState },
      event: SchematicNodeSelectedEvent,
    ) {
      if (state.selectionLocked) {
        if (rootState.session.currentWorkspace) {
          let request: IGetEntityRequest = {
            workspaceId: rootState.session.currentWorkspace.id,
            entityId: event.schematicNode.object.id,
          };

          if (rootState.editor.currentChangeSet) {
            request.changeSetId = rootState.editor.currentChangeSet.id;
          }
          await dispatch("loadEntity", request);
        }
      }
    },
    async onEntitySetName(
      { state, dispatch, commit },
      event: EntitySetNameEvent,
    ) {
      if (state.lastObjectListRequest) {
        await dispatch("loadObjectList", state.lastObjectListRequest);
      }
      if (state.lastLoadEntityRequest?.entityId == event.entity.id) {
        if (state.currentObject) {
          let diffResults = diffEntity(state.currentObject, event.entity);
          commit("updateFromDiff", diffResults);
          let diffResult = _.find(diffResults.entries, ["path", ["name"]]);
          let kind = "edit";
          if (diffResult) {
            kind = diffResult.kind;
          }
          emitPropChangeEvent(
            { path: ["name"] },
            event.entitySetNameRequest.entityId,
            // @ts-ignore
            kind,
            event.entitySetNameRequest.name,
          );
          for (diffResult of diffResults.entries) {
            if (!_.isEqual(diffResult.path, ["name"])) {
              emitPropChangeEvent(
                diffResult,
                event.entitySetNameRequest.entityId,
                diffResult.kind,
                diffResult.after,
              );
            }
          }
        }

        commit("setNeedsRefresh", true);
      }
    },
    async onEntityPropertySet(
      { state, commit },
      event: EntityPropertySetEvent,
    ) {
      if (state.lastLoadEntityRequest?.entityId == event.entity.id) {
        if (state.currentObject) {
          // All this shit should be coming from the backend, but I don't really want
          // to rewrite the diff algorithim in rust today. :)
          let diffResults = diffEntity(state.currentObject, event.entity);
          commit("updateFromDiff", diffResults);
          let diffResult = _.find(diffResults.entries, [
            "path",
            event.entitySetPropertyRequest.path,
          ]);
          let kind = "edit";
          if (diffResult) {
            kind = diffResult.kind;
          }
          emitPropChangeEvent(
            event.entitySetPropertyRequest,
            event.entitySetPropertyRequest.entityId,
            // @ts-ignore
            kind,
            event.entitySetPropertyRequest.value,
          );
          for (diffResult of diffResults.entries) {
            if (
              !_.isEqual(diffResult.path, event.entitySetPropertyRequest.path)
            ) {
              emitPropChangeEvent(
                diffResult,
                event.entitySetPropertyRequest.entityId,
                diffResult.kind,
                diffResult.after,
              );
            }
          }
        }
        commit("setNeedsRefresh", true);
      }
    },
  },
};
