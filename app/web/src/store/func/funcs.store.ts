import _ from "lodash";
import { defineStore } from "pinia";
import { watch } from "vue";
import { addStoreHooks } from "@/store/lib/pinia_hooks_plugin";

import { Visibility } from "@/api/sdf/dal/visibility";
import { FuncArgument, FuncVariant } from "@/api/sdf/dal/func";

import { nilId } from "@/utils/nilId";
import { useChangeSetsStore } from "../change_sets.store";
import { useRealtimeStore } from "../realtime/realtime.store";
import { useComponentsStore } from "../components.store";

import {
  FuncAssociations,
  InputSourceSocket,
  InputSourceProp,
  CreateFuncAttributeOptions,
} from "./types";
import { ApiRequest } from "../lib/pinia_api_tools";
import { useRouterStore } from "../router.store";

export type FuncId = string;

export type FuncSummary = {
  id: string;
  handler: string;
  variant: FuncVariant;
  name: string;
  description?: string;
  isBuiltin: boolean;
};
export type FuncWithDetails = FuncSummary & {
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
      inputSourceSockets: [] as InputSourceSocket[],
      inputSourceProps: [] as InputSourceProp[],
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

      funcById: (state) => (funcId: string) => state.funcDetailsById[funcId],

      funcList: (state) => _.values(state.funcsById),

      propForId: (state) => (propId: string) =>
        state.inputSourceProps.find((prop) => prop.propId === propId),

      // Filter props by schema variant
      propsAsOptionsForSchemaVariant: (state) => (schemaVariantId: string) =>
        state.inputSourceProps
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
        for (const socket of this.inputSourceSockets ?? []) {
          idMap[socket.internalProviderId] = `Socket: ${socket.name}`;
        }
        for (const prop of this.inputSourceProps ?? []) {
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
        for (const prop of this.inputSourceProps ?? []) {
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
          url: "func/save_func",
          params: {
            ...func,
            ...visibility,
          },
          keyRequestStatusBy: func.id,
          onSuccess: (response) => {
            this.funcDetailsById[func.id].associations = response.associations;
            this.funcDetailsById[func.id].isRevertible = response.isRevertible;
          },
        });
      },

      async REVERT_FUNC(funcId: FuncId) {
        return new ApiRequest<{ success: true }>({
          method: "post",
          url: "func/revert_func",
          params: { id: funcId, ...visibility },
          onSuccess: () => {
            this.FETCH_FUNC_DETAILS(funcId);
          },
        });
      },
      async EXEC_FUNC(funcId: FuncId) {
        return new ApiRequest<{ success: true }>({
          method: "post",
          url: "func/exec_func",
          params: { id: funcId },
        });
      },

      async CREATE_FUNC(createFuncRequest: {
        variant: FuncVariant;
        options?: CreateFuncAttributeOptions;
      }) {
        return new ApiRequest<FuncSummary>({
          method: "post",
          url: "func/create_func",
          params: { ...createFuncRequest, ...visibility },
        });
      },

      async FETCH_INPUT_SOURCE_LIST() {
        return new ApiRequest<{
          sockets: InputSourceSocket[];
          props: InputSourceProp[];
        }>({
          url: "func/list_input_sources",
          params: { ...visibility },
          onSuccess: (response) => {
            this.inputSourceSockets = response.sockets;
            this.inputSourceProps = response.props;
          },
        });
      },

      async updateFuncAttrArgs(funcId: FuncId, args: FuncArgument[]) {
        const func = this.funcDetailsById[funcId];
        if (func?.associations?.type !== "attribute") {
          return;
        }
        func.associations.arguments = args;
        await this.saveUpdatedFunc(funcId);
      },

      updateFuncCode(funcId: FuncId, code: string) {
        this.funcDetailsById[funcId].code = code;
        this.enqueueFuncSave(funcId);
      },

      enqueueFuncSave(funcId: FuncId) {
        // Lots of ways to handle this... we may want to handle this debouncing in the component itself
        // so the component has its own "draft" state that it passes back to the store when it's ready to save
        // however this should work for now, and lets the store handle this logic
        if (!this.saveQueue[funcId]) {
          this.saveQueue[funcId] = _.debounce(() => {
            this.saveUpdatedFunc(funcId);
          }, 2000);
        }
        // call debounced function which will trigger sending the save to the backend
        this.saveQueue[funcId]();
      },
      async saveUpdatedFunc(funcId: FuncId) {
        // saves the latest func state from the store, rather than passing in the new state
        return this.UPDATE_FUNC(this.funcDetailsById[funcId]);
      },
    },
    onActivated() {
      this.FETCH_FUNC_LIST();
      this.FETCH_INPUT_SOURCE_LIST();

      // could do this from components, but may as well do here...
      const stopWatchSelectedFunc = watch([() => this.selectedFuncId], () => {
        if (this.selectedFuncId) {
          // only fetch if we don't have this one already in our state,
          // otherwise we can overwrite functions with their previous value
          // before the save queue is drained.
          if (
            typeof this.funcDetailsById[this.selectedFuncId] === "undefined"
          ) {
            this.FETCH_FUNC_DETAILS(this.selectedFuncId);
          }
        }
      });

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
