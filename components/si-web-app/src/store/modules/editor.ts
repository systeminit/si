import { Module } from "vuex";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";
import { System } from "@/api/sdf/model/system";
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/model/changeSet";
import { EditSession } from "@/api/sdf/model/editSession";
import { User } from "@/api/sdf/model/user";
import { Node, NodeKind, NodeObject } from "@/api/sdf/model/node";
import { RootStore } from "@/store";

export interface ActionSetCurrent {
  id: string;
}

export interface ActionSetSystem {
  id: string;
}

export interface ActionSetChangeSet {
  id: string;
}

export interface ActionChangeSetCreate {
  name: string;
}

export interface ActionNodeCreate {
  kind: NodeKind;
  objectType: string;
}

export interface EditorStore {
  mode: "view" | "edit";
  isSaving: boolean;
  editSaveError: undefined | Error;
  changeSetsOpen: ChangeSet[];
  changeSet: ChangeSet | undefined;
  editSession: EditSession | undefined;
  application: Entity | undefined;
  system: System | undefined;
  systems: System[];
  nodes: Node[];
  objects: {
    [key: string]: NodeObject;
  };
  node: Node | undefined;
}

export const editor: Module<EditorStore, RootStore> = {
  namespaced: true,
  state: {
    mode: "view",
    isSaving: false,
    editSaveError: undefined,
    changeSetsOpen: [],
    changeSet: undefined,
    editSession: undefined,
    application: undefined,
    system: undefined,
    systems: [],
    nodes: [],
    objects: {},
    node: undefined,
  },
  mutations: {
    updateObjects(state, payload: NodeObject) {
      state.objects[payload.nodeId] = payload;
    },
    node(state, payload: Node | undefined) {
      state.node = payload;
    },
    updateNodes(state, payload: Node) {
      state.nodes = _.unionBy([payload], state.nodes, "id");
    },
    application(state, payload: Entity | undefined) {
      state.application = payload;
    },
    system(state, payload: System | undefined) {
      state.system = payload;
    },
    setSystems(state, payload: System[]) {
      state.systems = payload;
    },
    setIsSaving(state, saving: boolean) {
      state.isSaving = saving;
    },
    setEditSaveError(state, error: Error) {
      state.editSaveError = error;
    },
    setMode(state, mode: EditorStore["mode"]) {
      state.mode = mode;
    },
    changeSetsOpenAdd(state, payload: ChangeSet) {
      state.changeSetsOpen = _.unionBy([payload], state.changeSetsOpen, "id");
    },
    changeSetsOpenRemove(state, payload: ChangeSet) {
      state.changeSetsOpen = _.filter(state.changeSetsOpen, ["id", payload.id]);
    },
    changeSet(state, payload: ChangeSet | undefined) {
      state.changeSet = payload;
    },
    editSession(state, payload: EditSession | undefined) {
      state.editSession = payload;
    },
    clear(state) {
      state.mode = "view";
      state.isSaving = false;
      state.editSaveError = undefined;
      state.changeSetsOpen = [];
      state.changeSet = undefined;
      state.editSession = undefined;
      state.application = undefined;
      state.system = undefined;
      state.systems = [];
      state.nodes = [];
      state.objects = {};
      state.node = undefined;
    },
  },
  actions: {
    modeSwitch({ commit, state }) {
      if (state.mode == "view") {
        commit("setMode", "edit");
      } else {
        commit("setMode", "view");
      }
    },
    node({ commit }, payload: Node | undefined) {
      commit("node", payload);
    },
    async setChangeSet({ commit }, payload: ActionSetChangeSet) {
      if (payload.id) {
        let changeSet = await ChangeSet.get(payload);
        commit("changeSet", changeSet);
      } else {
        commit("changeSet", undefined);
      }
    },
    async setApplication({ commit }, payload: ActionSetCurrent) {
      let application = await Entity.get_head(payload);
      let systems = await application.systems();
      commit("application", application);
      commit("setSystems", systems);
      commit("system", systems[0]);
    },
    async setSystem({ commit }, payload: ActionSetSystem) {
      let system = await System.get(payload);
      commit("system", system);
    },
    async editSessionCreate({ commit, rootGetters, state }) {
      let workspace = rootGetters["workspace/current"];
      let organization = rootGetters["organization/current"];
      let user: User = rootGetters["user/current"];
      let changeSet = state.changeSet;
      if (!changeSet) {
        throw new Error("cannot start an edit session without a change set!");
      }
      let currentDate = new Date();
      let name = `${user.name} ${currentDate.toISOString()}`;
      let editSession = await EditSession.create(changeSet.id, {
        name,
        workspaceId: workspace.id,
        organizationId: organization.id,
      });
      commit("editSession", editSession);
    },
    async changeSetCreate(
      { commit, rootGetters, dispatch },
      payload: ActionChangeSetCreate,
    ) {
      let workspace = rootGetters["workspace/current"];
      let organization = rootGetters["organization/current"];
      let changeSet = await ChangeSet.create({
        name: payload.name,
        workspaceId: workspace.id,
        organizationId: organization.id,
      });
      commit("changeSet", changeSet);
      await dispatch("editSessionCreate");
    },
    async nodeCreate(
      { commit, rootGetters, state },
      payload: ActionNodeCreate,
    ) {
      let workspace = rootGetters["workspace/current"];
      let organization = rootGetters["organization/current"];
      let changeSetId = state.changeSet?.id;
      let editSessionId = state.editSession?.id;
      let system = state.system;
      let application = state.application;
      if (!changeSetId || !editSessionId || !system || !application) {
        throw new Error("invalid editor state; cannot add node");
      }

      const node = await Node.create({
        kind: payload.kind,
        objectType: payload.objectType,
        organizationId: organization.id,
        workspaceId: workspace.id,
        changeSetId,
        editSessionId,
        systemIds: [system.id],
      });
      const edge = await node.configured_by(application.nodeId);
      const object = await node.displayObject(changeSetId);
      // TODO: Getting node create to work! fishing about with the display state now.
      commit("updateNodes", node);
      commit("updateObjects", object);
    },
    fromChangeSet({ commit }, payload: ChangeSet) {
      if (payload.status == ChangeSetStatus.Open) {
        commit("changeSetsOpenAdd", payload);
      } else {
        commit("changeSetOpenRemove", payload);
      }
    },
    fromEditSession({ state, commit }, payload: EditSession) {
      if (state.editSession?.id == payload.id) {
        if (!_.isEqual(state.editSession, payload)) {
          commit("editSession", payload);
        }
      }
    },
    async clear({ commit }) {
      commit("clear");
    },
  },
};
