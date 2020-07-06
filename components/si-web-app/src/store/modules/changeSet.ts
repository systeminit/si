import { Module } from "vuex";
import _ from "lodash";

import {
  ChangeSet,
  ChangeSetCreateRequest,
  ChangeSetCreateReply,
  ChangeSetGetReply,
} from "@/graphql-types";
import {
  graphqlQuery,
  graphqlQueryListAll,
  graphqlMutation,
} from "@/api/apollo";
import { RootStore } from "@/store";
import { generateName } from "@/api/names";

export interface ChangeSetStore {
  changeSets: ChangeSet[];
  current: null | ChangeSet;
}

interface AddMutationPayload {
  changeSets: ChangeSet[];
}

export const changeSet: Module<ChangeSetStore, RootStore> = {
  namespaced: true,
  state: {
    changeSets: [],
    current: null,
  },
  mutations: {
    add(state, payload: AddMutationPayload) {
      state.changeSets = _.unionBy(payload.changeSets, state.changeSets, "id");
    },
    setCurrent(state, payload: ChangeSet) {
      state.current = payload;
    },
    setCurrentById(state, payload: string) {
      const newCurrent = _.find(state.changeSets, ["id", payload]);
      if (newCurrent) {
        state.current = newCurrent;
      }
    },
  },
  getters: {
    currentId(state): string {
      if (!state.current?.id) {
        throw new Error(
          "cannot get current change set ID, as there isn't one!",
        );
      }
      return state.current.id;
    },
    byId: (state: ChangeSetStore) => (
      changeSetId: string,
    ): ChangeSet | undefined => {
      let changeSet = _.find(state.changeSets, ["id", changeSetId]);
      if (changeSet) {
        return changeSet;
      } else {
        return undefined;
      }
    },
  },
  actions: {
    async load({ commit }): Promise<void> {
      const changeSets: ChangeSet[] = await graphqlQueryListAll({
        typeName: "changeSet",
      });
      if (changeSets.length > 0) {
        commit("add", { changeSets });
        //for (let changeSet of changeSets) {
        //  if (changeSet.status == "OPEN") {
        //    commit("setCurrent", changeSet);
        //    break;
        //  }
        //}
      }
    },
    async createDefault({ dispatch, rootGetters }) {
      let name = generateName();
      let createdByUserId: string = rootGetters["user/userId"];
      let workspaceId: string = rootGetters["user/currentWorkspaceId"];
      let request: ChangeSetCreateRequest = {
        name,
        displayName: name,
        createdByUserId,
        workspaceId,
      };
      await dispatch("create", request);
    },
    async create({ commit, state }, payload: ChangeSetCreateRequest) {
      let changeSet: ChangeSetCreateReply = await graphqlMutation({
        typeName: "changeSet",
        methodName: "create",
        variables: payload,
      });
      if (changeSet.item) {
        commit("add", { changeSets: [changeSet.item] });
        commit("setCurrent", changeSet.item);
      }
    },
    async execute({ commit, getters, dispatch }) {
      let changeSetId = getters.currentId;
      let changeSetExecuteResult = await graphqlMutation({
        typeName: "changeSet",
        methodName: "execute",
        variables: {
          id: changeSetId,
        },
        associations: {
          changeSet: ["changeSetEntries"],
        },
      });
      commit("add", { changeSets: [changeSetExecuteResult.item] });
      commit("setCurrent", changeSetExecuteResult.item);
      let pollerCount = 0;
      let poller = setInterval(() => {
        console.log(`POLLING ${pollerCount}`);
        pollerCount++;
        if (pollerCount >= 30) {
          clearInterval(poller);
          return;
        }
        graphqlQuery({
          typeName: "changeSet",
          methodName: "get",
          variables: {
            id: changeSetId,
          },
        })
          .then((res: ChangeSetGetReply) => {
            if (res.item?.status == "CLOSED" || res.item?.status == "FAILED") {
              clearInterval(poller);
              commit("add", { changeSets: [res.item] });
              commit("setCurrent", res.item);
              dispatch("entity/load", {}, { root: true });
            }
          })
          .catch(err => {
            console.log("Polling changeset execute error", err);
          });
      }, 1000);
    },
  },
};
