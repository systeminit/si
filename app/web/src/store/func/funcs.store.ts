import _ from "lodash";
import { defineStore } from "pinia";
import { useRoute } from "vue-router";
import { watch } from "vue";
import { addStoreHooks } from "@/store/lib/pinia_hooks_plugin";

import { Visibility } from "@/api/sdf/dal/visibility";
import { FuncArgument, FuncVariant } from "@/api/sdf/dal/func";

import { nilId } from "@/utils/nilId";
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
import { ApiRequest } from "../lib/pinia_api_tools";
import { useRouterStore } from "../router.store";

export const nullEditingFunc: EditingFunc = {
  id: nilId(),
  handler: "",
  variant: FuncVariant.Attribute,
  name: "",
  code: "",
  isBuiltin: false,
  isRevertible: false,
};

export type FuncId = string;

type FuncSummary = {
  id: string;
  handler: string;
  variant: FuncVariant;
  name: string;
  description?: string;
  isBuiltin: boolean;
};
type FuncWithDetails = FuncSummary & {
  code: string;
  isRevertible: boolean;
  associations?: FuncAssociations;
};

export const useFuncStore = () => {
  const componentsStore = useComponentsStore();
  const changeSetStore = useChangeSetsStore();
  const selectedChangeSetId = changeSetStore.selectedChangeSetId;
  const visibility: Visibility = {
    visibility_change_set_pk: selectedChangeSetId ?? nilId(),
  };

  const store = defineStore(`cs${selectedChangeSetId}/funcs`, {
    state: () => ({
      funcsById: {} as Record<FuncId, FuncSummary>,
      funcDetailsById: {} as Record<FuncId, FuncWithDetails>,
      inputSources: { sockets: [], props: [] } as ListInputSourcesResponse,
      saveQueue: {} as Record<FuncId, (...args: unknown[]) => unknown>,
    }),
    getters: {
      urlSelectedFuncId: () => {
        const route = useRouterStore().currentRoute;
        return route?.params?.funcId as FuncId | undefined;
      },
      selectedFuncId(): FuncId | undefined {
        return this.selectedFuncSummary?.id;
      },
      selectedFuncSummary(): FuncSummary | undefined {
        return this.funcsById[this.urlSelectedFuncId || ""];
      },
      selectedFuncDetails(): FuncWithDetails | undefined {
        return this.funcDetailsById[this.urlSelectedFuncId || ""];
      },

      funcList: (state) => _.values(state.funcsById),

      // Filter props by schema variant
      propsAsOptionsForSchemaVariant: (state) => (schemaVariantId: string) =>
        state.inputSources.props
          .filter(
            (prop) =>
              schemaVariantId === nilId() ||
              schemaVariantId === prop.schemaVariantId,
          )
          .map((prop) => ({
            label: `${prop.path}${prop.name}`,
            value: prop.propId,
          })),
      // selectedFunc: (state) => state.openFuncsById[state.selectedFuncId || ""],
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
        const idMap: { [key: string]: string } = {};
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
        const idMap: { [key: string]: string } = {};
        for (const prop of this.inputSources.props ?? []) {
          idMap[prop.propId] = `${prop.path}${prop.name}`;
        }
        return idMap;
      },
    },
    actions: {
      async FETCH_FUNC_LIST() {
        return new ApiRequest<{ funcs: FuncSummary[] }, Visibility>({
          url: "func/list_funcs",
          params: {
            ...visibility,
          },
          onSuccess: (response) => {
            this.funcsById = _.keyBy(response.funcs, (f) => f.id);
          },
        });
      },
      async FETCH_FUNC_DETAILS(funcId: FuncId) {
        return new ApiRequest<FuncWithDetails>({
          url: "func/get_func",
          params: {
            id: funcId,
            ...visibility,
          },
          keyRequestStatusBy: funcId,
          onSuccess: (response) => {
            this.funcDetailsById[response.id] = response;
          },
        });
      },
      async UPDATE_FUNC(func: FuncWithDetails) {
        return new ApiRequest<FuncWithDetails>({
          method: "post",
          url:
            import.meta.env.DEV && func.isBuiltin
              ? "dev/save_func"
              : "func/save_func",
          params: {
            ...func,
            ...visibility,
          },
          keyRequestStatusBy: func.id,
          onSuccess: (response) => {
            this.funcDetailsById[response.id] = response;
          },
        });
      },

      async FETCH_INPUT_SOURCE_LIST() {
        return listInputSources(visibility, (response) => {
          this.inputSources = response;
        });
      },

      async REVERT_FUNC(funcId: string) {
        return revertFunc({ ...visibility, id: funcId }, (response) => {
          this.FETCH_FUNC_DETAILS(funcId);
        });
      },
      async EXEC_FUNC(funcId: string) {
        return execFunc({ ...visibility, id: funcId });
      },
      async CREATE_FUNC(createFuncRequest: CreateFuncRequest) {
        return createFunc({
          ...visibility,
          ...createFuncRequest,
        });
      },
      async CREATE_BUILTIN_FUNC(createFuncRequest: CreateBuiltinFuncRequest) {
        return createBuiltinFunc({
          ...visibility, // seems odd the backend is asking for this?
          ...createFuncRequest,
        });
      },

      async removeFuncAttrPrototype(funcId: FuncId, prototypeId: string) {
        const func = this.funcDetailsById[funcId];
        if (func?.associations?.type !== "attribute") {
          return;
        }

        func.associations.prototypes = func.associations.prototypes.filter(
          (proto) => proto.id !== prototypeId,
        );
        this.funcDetailsById[funcId] = { ...func };
        await this.SAVE_UPDATED_FUNC(funcId);
      },
      async updateFuncAttrPrototype(
        funcId: FuncId,
        prototype: AttributePrototypeView,
      ) {
        const func = this.funcDetailsById[funcId];
        if (func?.associations?.type !== "attribute") {
          return;
        }

        if (prototype.id === nilId()) {
          func.associations.prototypes.push(prototype);
        } else {
          const currentPrototypeIdx = func.associations.prototypes.findIndex(
            (proto) => proto.id === prototype.id,
          );
          func.associations.prototypes[currentPrototypeIdx] = prototype;
        }
        this.SAVE_UPDATED_FUNC(funcId);
      },
      async updateFuncAttrArgs(funcId: FuncId, args: FuncArgument[]) {
        const func = this.funcDetailsById[funcId];
        if (func?.associations?.type !== "attribute") {
          return;
        }
        func.associations.arguments = args;
        await this.SAVE_UPDATED_FUNC(funcId);
      },
      async updateFuncAssociations(
        funcId: FuncId,
        associations: FuncAssociations | undefined,
      ) {
        this.funcDetailsById[funcId].associations = associations;
        await this.SAVE_UPDATED_FUNC(funcId);
      },
      updateFuncCode(funcId: FuncId, code: string) {
        this.funcDetailsById[funcId].code = code;
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
            ...this.funcDetailsById[funcId],
          },
          (response) => {
            const { associations, isRevertible } = response;
            if (associations) {
              this.funcDetailsById[funcId].associations = associations;
            }
            this.funcDetailsById[funcId].isRevertible = isRevertible;
          },
        );
      },
    },
    onActivated() {
      this.FETCH_FUNC_LIST();
      this.FETCH_INPUT_SOURCE_LIST();

      // could do this from components, but may as well do here...
      const stopWatchSelectedFunc = watch(
        [() => this.selectedFuncSummary],
        () => {
          if (this.selectedFuncSummary) {
            this.FETCH_FUNC_DETAILS(this.selectedFuncSummary?.id);
          }
        },
      );

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
        stopWatchSelectedFunc();
        realtimeStore.unsubscribe(this.$id);
      };
    },
  });

  return addStoreHooks(store)();
};
