import _ from "lodash";
import { defineStore } from "pinia";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";

import { Visibility } from "@/api/sdf/dal/visibility";
import { FuncArgument, FuncBackendKind } from "@/api/sdf/dal/func";

import { useChangeSetsStore } from "../change_sets.store";
import { useRealtimeStore } from "../realtime/realtime.store";
import { useComponentsStore } from "../components.store";

import { AttributePrototypeView, EditingFunc, FuncAssociations } from "./types";
import { ListedFuncView, listFuncs } from "./requests/list_funcs";
import { getFunc } from "./requests/get_func";
import { revertFunc } from "./requests/revert_func";
import {
  createBuiltinFunc,
  CreateBuiltinFuncRequest,
  createFunc,
  CreateFuncRequest,
} from "./requests/create_func";
import {
  listInputSources,
  ListInputSourcesResponse,
} from "./requests/list_input_sources";
import { execFunc } from "./requests/exec_func";
import { saveFunc } from "./requests/save_func";

export const nullEditingFunc: EditingFunc = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  code: "",
  isBuiltin: false,
  isRevertible: false,
};

type FuncId = number;

export const useFuncStore = () => {
  const componentsStore = useComponentsStore();
  const changeSetStore = useChangeSetsStore();
  const selectedChangeSetId = changeSetStore.selectedChangeSetId;
  const visibility: Visibility = {
    visibility_change_set_pk: selectedChangeSetId ?? -1,
  };

  const store = defineStore(`cs${selectedChangeSetId}/funcs`, {
    state: () => ({
      funcList: [] as ListedFuncView[],
      openFuncsById: {} as Record<FuncId, EditingFunc>,
      openFuncsList: [] as ListedFuncView[],
      selectedFuncId: -1 as FuncId,
      inputSources: { sockets: [], props: [] } as ListInputSourcesResponse,

      saveQueue: {} as Record<FuncId, (...args: unknown[]) => unknown>,
    }),
    getters: {
      getFuncById:
        (state) =>
        (funcId: FuncId): EditingFunc | undefined =>
          state.openFuncsById[funcId],
      selectedFuncIndex: (state) =>
        state.openFuncsList.findIndex((f) => f.id === state.selectedFuncId),
      getFuncByIndex: (state) => (index: number) => state.openFuncsList[index],
      getIndexForFunc: (state) => (funcId: FuncId) =>
        state.openFuncsList.findIndex((f) => f.id === funcId),
      selectedFunc: (state) => state.openFuncsById[state.selectedFuncId],
      schemaVariantOptions() {
        return componentsStore.schemaVariants.map((sv) => ({
          label: sv.schemaName,
          value: sv.id,
        }));
      },
      componentOptions() {
        return componentsStore.allComponents.map(({ displayName, id }) => ({
          label: displayName,
          value: id,
        }));
      },
      providerIdToSourceName() {
        const idMap: { [key: number]: string } = {};
        for (const socket of this.inputSources.sockets ?? []) {
          idMap[socket.internalProviderId] = `Socket: ${socket.name}`;
        }
        for (const prop of this.inputSources.props ?? []) {
          if (prop.internalProviderId) {
            idMap[
              prop.internalProviderId
            ] = `Attribute: ${prop.path}${prop.name}`;
          }
        }

        return idMap;
      },
      propIdToSourceName() {
        const idMap: { [key: number]: string } = {};
        for (const prop of this.inputSources.props ?? []) {
          idMap[prop.propId] = `${prop.path}${prop.name}`;
        }
        return idMap;
      },
    },
    actions: {
      async SELECT_FUNC(funcId: FuncId) {
        if (funcId === -1) {
          return;
        }

        const existing = this.openFuncsById[funcId];
        if (existing) {
          this.selectedFuncId = funcId;
          this.ADD_FUNC_TO_OPEN_LIST(funcId);
          return existing;
        }

        await this.FETCH_FUNC(funcId);
        this.selectedFuncId = funcId;
        this.ADD_FUNC_TO_OPEN_LIST(funcId);
        return this.openFuncsById[funcId];
      },
      ADD_FUNC_TO_OPEN_LIST(funcId: FuncId) {
        const func = this.openFuncsById[funcId];
        if (func && this.getIndexForFunc(funcId) === -1) {
          this.openFuncsList.push(func as ListedFuncView);
        }
      },
      REMOVE_FUNC_FROM_OPEN_LIST(funcId: number) {
        this.openFuncsList = this.openFuncsList.filter((f) => f.id !== funcId);
      },
      CLOSE_FUNC(funcId: number) {
        this.REMOVE_FUNC_FROM_OPEN_LIST(funcId);
      },
      SELECT_FUNC_BY_INDEX(index: number) {
        if (index < 0) {
          index = 0;
        } else if (index > this.openFuncsList.length) {
          index--;
        }

        if (this.openFuncsList.length) {
          this.SELECT_FUNC(this.getFuncByIndex(index).id);
        }
      },
      async FETCH_FUNC_LIST() {
        return listFuncs(visibility, (response) => {
          this.funcList = response.funcs;
        });
      },
      async FETCH_INPUT_SOURCE_LIST() {
        return listInputSources(visibility, (response) => {
          this.inputSources = response;
        });
      },
      async FETCH_FUNC(funcId: FuncId) {
        return getFunc({ ...visibility, id: funcId }, (response) => {
          this.openFuncsById[response.id] = response;
        });
      },

      async REVERT_FUNC(funcId: number) {
        return revertFunc({ ...visibility, id: funcId }, (response) => {
          this.FETCH_FUNC(funcId);
        });
      },
      async EXEC_FUNC(funcId: number) {
        return execFunc({ ...visibility, id: funcId });
      },
      async CREATE_FUNC(createFuncRequest: CreateFuncRequest) {
        return createFunc({
          ...visibility,
          ...createFuncRequest,
        });
      },
      async CREATE_BUIILTIN_FUNC(createFuncRequest: CreateBuiltinFuncRequest) {
        return createBuiltinFunc({
          ...visibility, // seems odd the backend is asking for this?
          ...createFuncRequest,
        });
      },

      removeFuncAttrPrototype(funcId: FuncId, prototypeId: number) {
        const func = this.openFuncsById[funcId];
        if (func?.associations?.type !== "attribute") {
          return;
        }

        func.associations.prototypes = func.associations.prototypes.filter(
          (proto) => proto.id !== prototypeId,
        );
        this.enqueueFuncSave(funcId);
      },
      updateFuncAttrPrototype(
        funcId: FuncId,
        prototype: AttributePrototypeView,
      ) {
        const func = this.openFuncsById[funcId];
        if (func?.associations?.type !== "attribute") {
          return;
        }

        if (prototype.id === -1) {
          func.associations.prototypes.push(prototype);
        } else {
          const currentPrototypeIdx = func.associations.prototypes.findIndex(
            (proto) => proto.id === prototype.id,
          );
          func.associations.prototypes[currentPrototypeIdx] = prototype;
        }
        this.enqueueFuncSave(funcId);
      },
      updateFuncAttrArgs(funcId: FuncId, args: FuncArgument[]) {
        const func = this.openFuncsById[funcId];
        if (func?.associations?.type !== "attribute") {
          return;
        }
        func.associations.arguments = args;
        this.enqueueFuncSave(funcId);
      },
      updateFuncAssociations(
        funcId: FuncId,
        associations: FuncAssociations | undefined,
      ) {
        this.openFuncsById[funcId].associations = associations;
        this.enqueueFuncSave(funcId);
      },
      updateFuncCode(funcId: FuncId, code: string) {
        this.openFuncsById[funcId].code = code;
        this.enqueueFuncSave(funcId);
      },

      enqueueFuncSave(funcId: FuncId) {
        // Lots of ways to handle this... we may want to handle this debouncing in the component itself
        // so the component has it's own "draft" state that it passes back to the store when it's ready to save
        // however this should work for now, and lets the store handle this logic
        if (!this.saveQueue[funcId]) {
          this.saveQueue[funcId] = _.debounce(() => {
            this.SAVE_UPDATED_FUNC(funcId);
          }, 2000);
        }
        // call debounced function which will trigger sending the save to the backend
        this.saveQueue[funcId]();
      },
      async SAVE_UPDATED_FUNC(funcId: FuncId) {
        // saves the latest func state from the store, rather than passing in the new state
        return saveFunc(
          {
            ...visibility,
            ...this.openFuncsById[funcId],
          },
          (response) => {
            const { associations, isRevertible } = response;
            if (associations) {
              this.openFuncsById[funcId].associations = associations;
            }
            this.openFuncsById[funcId].isRevertible = isRevertible;
          },
        );
      },
    },
    onActivated() {
      this.FETCH_FUNC_LIST();
      this.FETCH_INPUT_SOURCE_LIST();

      const realtimeStore = useRealtimeStore();
      realtimeStore.subscribe(this.$id, `changeset/${selectedChangeSetId}`, [
        {
          eventType: "ChangeSetWritten",
          callback: (writtenChangeSetId) => {
            if (writtenChangeSetId !== selectedChangeSetId) return;
            this.FETCH_FUNC_LIST();
          },
        },
      ]);
      return () => {
        realtimeStore.unsubscribe(this.$id);
      };
    },
  });

  return addStoreHooks(store)();
};
