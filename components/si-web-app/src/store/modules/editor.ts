import { Module } from "vuex";
import { EntityObject } from "si-registry/lib/systemComponent";
import { INodeCreateReply } from "@/api/sdf/dal/editorDal";
import { SessionStore } from "@/store/modules/session";
import { ChangeSet } from "@/api/sdf/model/changeSet";
import { PartyBus } from "@/api/partyBus";
import { EditSession } from "@/api/sdf/model/editSession";
import Bottle from "bottlejs";
import { CurrentChangeSetEvent } from "@/api/partyBus/currentChangeSetEvent";
import { EditSessionCurrentSetEvent } from "@/api/partyBus/editSessionCurrentSetEvent";
import { EditorDal } from "@/api/sdf/dal/editorDal";
import { NodeKind } from "@/api/sdf/model/node";
import { NodeCreatedEvent } from "@/api/partyBus/NodeCreatedEvent";

export type IEditorContext = IEditorContextApplication;

export interface IEditorContextApplication {
  applicationId: string;
}

export interface EditorStore {
  version: number;
  context: IEditorContext | null;
  currentChangeSet: ChangeSet | null;
  currentEditSession: EditSession | null;
}

export function setupEditor() {
  const bottle = Bottle.pop("default");
  const partyBus: PartyBus = bottle.container.PartyBus;
  partyBus.subscribeToEvents("editor", undefined, [
    CurrentChangeSetEvent,
    EditSessionCurrentSetEvent,
  ]);
}

export const editor: Module<EditorStore, any> = {
  namespaced: true,
  state: {
    version: 1,
    currentChangeSet: null,
    currentEditSession: null,
    context: null,
  },
  getters: {
    inEditable(state): boolean {
      if (state.currentChangeSet && state.currentEditSession && state.context) {
        return true;
      } else {
        return false;
      }
    },
  },
  mutations: {
    setCurrentChangeSet(state, payload: EditorStore["currentChangeSet"]) {
      state.currentChangeSet = payload;
    },
    setCurrentEditSession(state, payload: EditorStore["currentEditSession"]) {
      state.currentEditSession = payload;
    },
    setContext(state, payload: EditorStore["context"]) {
      state.context = payload;
    },
  },
  actions: {
    async onCurrentChangeSet({ commit }, event: CurrentChangeSetEvent) {
      commit("setCurrentChangeSet", event.changeSet);
    },
    async onEditSessionCurrentSet(
      { commit },
      event: EditSessionCurrentSetEvent,
    ) {
      commit("setCurrentEditSession", event.editSession);
    },
    async setContext({ commit }, context: EditorStore["context"]) {
      commit("setContext", context);
    },
    async nodeCreate(
      { state, rootState },
      entityObject: EntityObject,
    ): Promise<INodeCreateReply> {
      let currentWorkspace: SessionStore["currentWorkspace"] =
        rootState.session.currentWorkspace;
      let currentSystem: SessionStore["currentSystem"] =
        rootState.session.currentSystem;
      if (
        !currentWorkspace ||
        !currentSystem ||
        !state.currentEditSession ||
        !state.currentChangeSet ||
        !state.context
      ) {
        throw new Error(
          "Cannot call nodeCreate without a workspace, system, changeSet and editSession or EditContext! bug!",
        );
      }
      let reply: INodeCreateReply;
      if (state.context.applicationId) {
        reply = await EditorDal.nodeCreateForApplication({
          kind: NodeKind.Entity,
          objectType: entityObject.typeName,
          workspaceId: currentWorkspace.id,
          changeSetId: state.currentChangeSet.id,
          editSessionId: state.currentEditSession.id,
          systemId: currentSystem.id,
          applicationId: state.context.applicationId,
        });
      } else {
        throw new Error("cannot create without an editor context");
      }
      if (!reply.error) {
        new NodeCreatedEvent(reply).publish();
      }

      return reply;
    },
  },
};
