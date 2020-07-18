import { Module } from "vuex";
import _ from "lodash";

import {
  ChangeSet,
  ChangeSetCreateRequest,
  ChangeSetCreateReply,
  ChangeSetGetReply,
  ItemListReply,
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

interface AddMutation {
  changeSets: ChangeSet[];
}

interface CountGetter {
  status: ChangeSet["status"];
  forId?: string;
}

export const changeSet: Module<ChangeSetStore, RootStore> = {
  namespaced: true,
  state: {
    changeSets: [],
    current: null,
  },
  mutations: {
    add(state, payload: AddMutation) {
      state.changeSets = _.unionBy(payload.changeSets, state.changeSets, "id");
    },
    current(state, payload: ChangeSet) {
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
    // prettier-ignore
    count: (state) => (payload: CountGetter): number => {
      let results = _.filter(state.changeSets, (changeSet) => {
        if (changeSet.associations?.changeSetEntries?.items && changeSet.status == payload.status) {
          return _.find(changeSet.associations.changeSetEntries.items, (entry: any) => {
            if (entry.siStorable?.itemId == payload.forId) {
              return true;
            } else {
              return false;
            }
          });
        }
      });
      return results.length;
    },
    current(state): ChangeSet {
      if (state.current) {
        return state.current;
      } else {
        throw new Error("Cannot get current changeSet; it is not set!");
      }
    },
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
        associations: {
          changeSet: ["changeSetEntries"],
        },
      });
      if (changeSets.length > 0) {
        commit("add", { changeSets });
      }
    },
    async get(
      { commit, state },
      { changeSetId }: { changeSetId: string },
    ): Promise<void> {
      const changeSetReply: ChangeSetGetReply = await graphqlQuery({
        typeName: "changeSet",
        methodName: "get",
        variables: {
          id: changeSetId,
        },
        associations: {
          changeSet: ["changeSetEntries"],
        },
      });
      if (changeSetReply.item) {
        commit("add", { changeSets: [changeSetReply.item] });
        if (state.current?.id == changeSetReply.item?.id) {
          commit("current", changeSetReply.item);
        }
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
        associations: {
          changeSet: ["changeSetEntries"],
        },
      });
      if (changeSet.item) {
        commit("add", { changeSets: [changeSet.item] });
        commit("current", changeSet.item);
      }
    },
    async execute({ commit, getters, dispatch }, payload: { wait?: boolean }) {
      const wait = payload.wait ? payload.wait : false;
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
      commit("current", changeSetExecuteResult.item);
      let pollerCount = 0;
      let finished = false;
      let poller = setInterval(() => {
        pollerCount++;
        if (pollerCount >= 300) {
          clearInterval(poller);
          return;
        }
        graphqlQuery({
          typeName: "changeSet",
          methodName: "get",
          variables: {
            id: changeSetId,
          },
          associations: {
            changeSet: ["changeSetEntries"],
          },
        })
          .then(async (res: ChangeSetGetReply) => {
            if (res.item?.status == "CLOSED" || res.item?.status == "FAILED") {
              clearInterval(poller);
              commit("add", { changeSets: [res.item] });
              commit("current", res.item);
              if (res.item?.associations?.changeSetEntries?.items) {
                let remainingItems = true;
                let nextPageToken =
                  res.item.associations.changeSetEntries.nextPageToken;
                let changeSetEntryItems =
                  res.item.associations.changeSetEntries.items;
                while (remainingItems) {
                  for (const changeSetEntry of changeSetEntryItems) {
                    if (
                      !changeSetEntry.siStorable?.typeName?.endsWith(
                        "entity_event",
                      )
                    ) {
                      await dispatch(
                        "entity/get",
                        {
                          id: changeSetEntry.id,
                          typeName: changeSetEntry.siStorable?.typeName,
                        },
                        { root: true },
                      );
                      await dispatch(
                        "entity/get",
                        {
                          id: changeSetEntry.siStorable?.itemId,
                          typeName: changeSetEntry.siStorable?.typeName,
                        },
                        { root: true },
                      );
                    }
                  }
                  if (nextPageToken) {
                    const nextResults: ItemListReply = await graphqlQuery({
                      typeName: "item",
                      methodName: "list",
                      variables: {
                        pageToken: nextPageToken,
                      },
                    });
                    if (nextResults.items) {
                      nextPageToken = nextResults.nextPageToken;
                      changeSetEntryItems = nextResults.items;
                    } else {
                      // But how did we get here?
                      remainingItems = false;
                    }
                  } else {
                    remainingItems = false;
                  }
                }
              }
              finished = true;
            }
          })
          .catch(err => {
            console.log("Polling changeset execute error", err);
            finished = true;
            clearInterval(poller);
          });
      }, 100);
      if (wait) {
        let finishCounter = 0;
        while (!finished) {
          if (finishCounter > 100) {
            return;
          }
          await new Promise(resolve => setTimeout(resolve, 100));
          finishCounter++;
        }
      }
    },
  },
};
