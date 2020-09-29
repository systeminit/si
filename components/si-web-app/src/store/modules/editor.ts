import { Module } from "vuex";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";
import { System } from "@/api/sdf/model/system";
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/model/changeSet";
import { EditSession } from "@/api/sdf/model/editSession";
import { User } from "@/api/sdf/model/user";
import { Node, NodeKind, NodeObject, Position } from "@/api/sdf/model/node";
import { Edge } from "@/api/sdf/model/edge";
import { RootStore } from "@/store";

export interface ActionSetCurrent {
  id: string;
}

export interface ActionSetSystem {
  id: string;
}

export interface ActionSetChangeSet {
  id: string | undefined;
}

export interface ActionChangeSetCreate {
  name: string;
}

export interface ActionNodeCreate {
  kind: NodeKind;
  objectType: string;
}

export interface ActionSetNodePosition {
  nodeId: string;
  position: Position;
}

export interface IConnectionPosition {
  sourceNodePosition: {
    nodeId: string;
    x: number;
    y: number;
  };
  destinationNodePosition: {
    nodeId: string;
    x: number;
    y: number;
  };
}

export interface EditorStore {
  mode: "view" | "edit";
  context: string;
  mouseTrackSelection: string | undefined;
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
  edges: Edge[];
  node: Node | undefined;
}

export let SET_POSITION_FUNCTIONS: Record<string, any> = {};

export const editor: Module<EditorStore, RootStore> = {
  namespaced: true,
  state: {
    context: "none",
    mode: "view",
    mouseTrackSelection: undefined,
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
    edges: [],
    node: undefined,
  },
  mutations: {
    mouseTrackSelection(state, payload: string | undefined) {
      state.mouseTrackSelection = payload;
    },
    context(state, payload: string) {
      state.context = payload;
    },
    updateObjects(state, payload: NodeObject) {
      state.objects[payload.nodeId] = payload;
    },
    node(state, payload: Node | undefined) {
      state.node = payload;
    },
    updateNodes(state, payload: Node) {
      state.nodes = _.unionBy([payload], state.nodes, "id");
    },
    setNodes(state, payload: Node[]) {
      state.nodes = payload;
    },
    setObjects(state, payload: EditorStore["objects"]) {
      state.objects = payload;
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
    setEdges(state, edges: Edge[]) {
      state.edges = edges;
    },
    changeSetsOpenAdd(state, payload: ChangeSet) {
      state.changeSetsOpen = _.unionBy([payload], state.changeSetsOpen, "id");
    },
    changeSetsOpenRemove(state, payload: ChangeSet) {
      let changeSetsOpen = _.cloneDeep(state.changeSetsOpen);
      _.remove(changeSetsOpen, ["id", payload.id]);
      state.changeSetsOpen = changeSetsOpen;
    },
    changeSet(state, payload: ChangeSet | undefined) {
      state.changeSet = payload;
    },
    editSession(state, payload: EditSession | undefined) {
      state.editSession = payload;
    },
    clear(state) {
      state.context = "none";
      state.mode = "view";
      state.mouseTrackSelection = undefined;
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
      state.edges = [];
      SET_POSITION_FUNCTIONS = {};
    },
  },
  getters: {
    nodeList(state): EditorStore["nodes"] {
      return _.filter(state.nodes, n => {
        if (state.objects[n.id]) {
          return true;
        } else {
          return false;
        }
      });
    },
    positions(state, getters): IConnectionPosition[] {
      const result: IConnectionPosition[] = [];
      if (state.context) {
        for (let edge of state.edges) {
          let sourceNode = _.find(getters["nodeList"], [
            "id",
            edge.tailVertex.nodeId,
          ]);
          let sourceNodePosition: Position | undefined;
          let destinationNodePosition: Position | undefined;
          if (sourceNode) {
            sourceNodePosition = {
              nodeId: sourceNode.id,
              ...Node.upgrade(sourceNode).position(state.context),
            };
          }
          let destNode = _.find(getters["nodeList"], [
            "id",
            edge.headVertex.nodeId,
          ]);
          if (destNode) {
            destinationNodePosition = {
              nodeId: destNode.id,
              ...Node.upgrade(destNode).position(state.context),
            };
          }
          if (sourceNodePosition && destinationNodePosition) {
            result.push({ sourceNodePosition, destinationNodePosition });
          }
        }
      }
      return result;
    },
  },
  actions: {
    async setNodePosition({ state, commit }, payload: ActionSetNodePosition) {
      let node = _.find(state.nodes, ["id", payload.nodeId]);
      if (node) {
        let unode = Node.upgrade(_.cloneDeep(node));
        unode.positions[state.context] = payload.position;
        commit("updateNodes", unode);
      }
      if (SET_POSITION_FUNCTIONS[payload.nodeId]) {
        SET_POSITION_FUNCTIONS[payload.nodeId](payload.position);
      } else {
        SET_POSITION_FUNCTIONS[payload.nodeId] = _.debounce(
          async (position: Position) => {
            let node = await Node.get({ id: payload.nodeId });
            await node.setPosition(state.context, position);
          },
          1000,
        );
        SET_POSITION_FUNCTIONS[payload.nodeId](payload.position);
      }
    },
    async setMouseTrackSelection({ commit }, payload: string | undefined) {
      commit("mouseTrackSelection", payload);
    },
    async context({ commit, state }) {
      let contextState = ["application"];

      if (state.application) {
        contextState.push(state.application.id);
      }
      if (state.system) {
        contextState.push(state.system.id);
      }
      commit("context", contextState.join("."));
    },
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
    async changeSetExecute({ commit, state }) {
      const changeSet = state.changeSet;
      if (changeSet) {
        await ChangeSet.upgrade(changeSet).execute({ hypothetical: false });
        commit("setMode", "view");
      }
    },
    async setChangeSet({ commit, state }, payload: ActionSetChangeSet) {
      if (payload.id) {
        // @ts-ignore
        let changeSet = await ChangeSet.get(payload);
        commit("changeSet", changeSet);
      } else {
        commit("changeSet", undefined);
      }
      let application = state.application;
      console.log("application", { application });
      if (application) {
        let applicationNode = await Node.get({ id: application.nodeId });
        let successors = await applicationNode.successors();
        let objects: Record<string, NodeObject> = {};
        for (let n of successors) {
          try {
            let obj = await n.displayObject(payload.id);
            objects[obj.nodeId] = obj;
          } catch {}
        }
        commit("setObjects", objects);
      }
    },
    async setApplication(
      { commit, state, dispatch },
      payload: ActionSetCurrent,
    ) {
      let application = await Entity.get_head(payload);
      let applicationNode = await Node.get({ id: application.nodeId });
      let successors = await applicationNode.successors();
      let systems = await application.systems();
      commit("application", application);
      commit("setSystems", systems);
      commit("system", systems[0]);
      let objects: Record<string, NodeObject> = {};
      for (let n of successors) {
        try {
          let obj = await n.displayObject(state.changeSet?.id);
          objects[obj.nodeId] = obj;
        } catch {
          console.log("node object not included in this changeset");
        }
      }
      commit("setObjects", objects);
      commit("setNodes", successors);
      let edges = await applicationNode.successorEdges();
      commit("setEdges", edges);
      await dispatch("context");
    },
    async setSystem({ commit, dispatch }, payload: ActionSetSystem) {
      let system = await System.get(payload);
      commit("system", system);
      await dispatch("context");
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
      console.log("started create");
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
      const edge = await node.configuredBy(application.nodeId);
      const object = await node.displayObject(changeSetId);
      commit("updateObjects", object);
      commit("updateNodes", node);
      commit("mouseTrackSelection", node.id);
      console.log("finished create");
    },
    async fromNode({ commit, state }, payload: Node) {
      if (state.application) {
        let appNode = await state.application.node();
        let successors = await appNode.successors();
        if (_.find(successors, ["id", payload.id])) {
          commit("updateNodes", payload);
          if (state.node?.id == payload.id) {
            commit("node", payload);
          }
        }
      }
    },
    async fromEntity({ commit, state }, payload: Entity) {
      if (state.application) {
        let application = state.application;
        let appNode = await Node.get({ id: application.id });
        let successors = await appNode.successors();
        if (_.find(successors, ["tailVertex.objectId", payload.id])) {
          let changeSet = state.changeSet;
          if (changeSet) {
            if (
              payload.siChangeSet.changeSetId == changeSet.id &&
              payload.head == false
            ) {
              commit("updateObjects", payload);
            }
          } else {
            if (payload.head == true) {
              commit("updateObjects", payload);
            }
          }
        }
      }
    },
    async fromEdge({ commit, state }, payload: Edge) {
      let application = state.application;
      let updatedEdges = false;
      if (application) {
        let appNode = await Node.get({ id: application.nodeId });
        let successors = await appNode.successors();
        if (
          _.find(successors, ["id", payload.tailVertex.nodeId]) ||
          appNode.id == payload.tailVertex.nodeId
        ) {
          updatedEdges = true;
          let changeSetId = state.changeSet?.id;
          let node = await Node.get({ id: payload.headVertex.nodeId });
          let entity = await node.displayObject(changeSetId);
          commit("updateNodes", node);
          if (state.node?.id == node.id) {
            commit("node", node);
          }
          commit("updateObjects", entity);
          let nSuccessors = await node.successors();
          for (let ns of nSuccessors) {
            let ne = await ns.displayObject(changeSetId);
            commit("updateNodes", ns);
            if (state.node?.id == ns.id) {
              commit("node", ns);
            }
            commit("updateObjects", ne);
          }
        }
      }
      if (updatedEdges && application) {
        let appNode = await Node.get({ id: application.nodeId });
        let edges = await appNode.successorEdges();
        commit("setEdges", edges);
      }
    },
    fromChangeSet({ commit, dispatch, state }, payload: ChangeSet) {
      if (payload.status == ChangeSetStatus.Open) {
        console.log("updating from change set", { payload });
        commit("changeSetsOpenAdd", payload);
      } else {
        if (state.changeSet?.id == payload.id) {
          console.log("removing from change set", { payload });
          dispatch("setChangeSet", { id: undefined });
        }
        commit("changeSetsOpenRemove", payload);
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
