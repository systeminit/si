import { Module } from "vuex";
import { Entity } from "@/api/sdf/model/entity";
import { System } from "@/api/sdf/model/system";
import { Resource } from "@/api/sdf/model/resource";
import {
  ApplicationDal,
  IApplicationCreateRequest,
  IApplicationCreateReply,
  IApplicationCreateReplySuccess,
  IApplicationListRequest,
  IApplicationListReply,
} from "@/api/sdf/dal/applicationDal";
import _ from "lodash";
import Vue from "vue";
import Bottle from "bottlejs";
import { UpdateTracker } from "@/api/updateTracker";

export { IApplicationCreateReply, IApplicationCreateRequest };

export type ISetListApplicationsRequest = IApplicationListRequest;
export type ISetListApplicationsReply = IApplicationListReply;

export type IApplicationListEntry = Omit<
  IApplicationCreateReplySuccess,
  "error"
>;

export interface ApplicationStore {
  activatedBy: string[];
  applicationList: IApplicationListEntry[];
}

export const application: Module<ApplicationStore, any> = {
  namespaced: true,
  state: {
    activatedBy: [],
    applicationList: [],
  },
  mutations: {
    addToActivatedBy(state, payload: string) {
      state.activatedBy = _.unionBy(state.activatedBy, [payload]);
    },
    removeFromActivatedBy(state, payload: string) {
      state.activatedBy = _.without(state.activatedBy, payload);
    },
    clear(state) {
      state.applicationList = [];
    },
    setApplicationList(state, payload: IApplicationListEntry[]) {
      state.applicationList = payload;
    },
    updateApplicationList(state, payload: IApplicationListEntry) {
      state.applicationList = _.unionBy(
        [payload],
        state.applicationList,
        "application.id",
      );
    },
    updateApplicationInList(state, payload: Entity) {
      let appIndex = _.findIndex(state.applicationList, [
        "application.id",
        payload.id,
      ]);
      let entry = state.applicationList[appIndex];
      entry.application = payload;
      Vue.set(state, appIndex, entry);
    },
    updateSystemInApplicationList(state, payload: System) {
      let entriesWithSystem = _.filter(state.applicationList, entry => {
        return _.some(entry.systems, ["id", payload.id]);
      });
      if (entriesWithSystem.length) {
        for (let entry of entriesWithSystem) {
          entry.systems = _.unionBy([payload], entry.systems, "id");
        }
        state.applicationList = _.unionBy(
          entriesWithSystem,
          state.applicationList,
          "application.id",
        );
      }
    },
    updateResourceInApplicationList(state, payload: Resource) {
      let entryWithResource = _.find(state.applicationList, entry => {
        return _.some(entry.servicesWithResources, swr => {
          return _.some(swr.resources, ["id", payload.id]);
        });
      });
      if (entryWithResource) {
        for (let swr of entryWithResource.servicesWithResources) {
          const index = _.findIndex(swr.resources, ["id", payload.id]);
          if (!_.isNull(index)) {
            swr.resources[index] = payload;
          }
        }
        state.applicationList = _.unionBy(
          [entryWithResource],
          state.applicationList,
          "application.id",
        );
      }
    },
  },
  actions: {
    activate({ commit }, payload: string) {
      const bottle = Bottle.pop("default");
      const updateTracker = bottle.container.UpdateTracker;
      updateTracker.register("Entity", "application");

      commit("addToActivatedBy", payload);
    },
    deactivate({ commit, state }, payload: string) {
      commit("removeFromActivatedBy", payload);
      if (state.activatedBy.length == 0) {
        const bottle = Bottle.pop("default");
        const updateTracker: UpdateTracker = bottle.container.UpdateTracker;
        if (updateTracker) {
          updateTracker.unregister("Entity", "application");
        }
        commit("clear");
      }
    },
    async setListApplications(
      { commit },
      request: ISetListApplicationsRequest,
    ): Promise<ISetListApplicationsReply> {
      let reply = await ApplicationDal.listApplications(request);
      if (!reply.error) {
        commit("setApplicationList", reply.list);
      }
      return reply;
    },
    async createApplication(
      { commit },
      request: IApplicationCreateRequest,
    ): Promise<IApplicationCreateReply> {
      let reply = await ApplicationDal.createApplication(request);
      if (!reply.error) {
        commit("updateApplicationList", reply);
      }
      return reply;
    },
    async fromEntity({ dispatch, state, rootState }, payload: Entity) {
      if (state.activatedBy.length != 0) {
        if (
          payload.entityType == "application" ||
          payload.entityType == "service"
        ) {
          if (rootState.session.currentWorkspace.id) {
            let updateReq: ISetListApplicationsRequest = {
              workspaceId: rootState.session.currentWorkspace.id,
            };
            await dispatch("setListApplications", updateReq);
          }
        }
      }
    },
    async fromSystem({ state, dispatch, rootState }, _payload: System) {
      if (state.activatedBy.length != 0) {
        let updateReq: ISetListApplicationsRequest = {
          workspaceId: rootState.session.currentWorkspace.id,
        };
        await dispatch("setListApplications", updateReq);
      }
    },
    async fromResource({ state, dispatch, rootState }, _payload: Resource) {
      if (state.activatedBy.length != 0) {
        let updateReq: ISetListApplicationsRequest = {
          workspaceId: rootState.session.currentWorkspace.id,
        };
        await dispatch("setListApplications", updateReq);
      }
    },
  },
};
