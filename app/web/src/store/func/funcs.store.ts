import * as _ from "lodash-es";
import { defineStore } from "pinia";
import { watch } from "vue";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";

import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { Visibility } from "@/api/sdf/dal/visibility";
import { FuncVariant } from "@/api/sdf/dal/func";

import { nilId } from "@/utils/nilId";
import { trackEvent } from "@/utils/tracking";
import { useChangeSetsStore } from "../change_sets.store";
import { useRealtimeStore } from "../realtime/realtime.store";
import { useComponentsStore } from "../components.store";

import {
  FuncAssociations,
  InputSourceSocket,
  InputSourceProp,
  OutputSocket,
  CreateFuncAttributeOptions,
  AttributePrototypeView,
  OutputLocation,
} from "./types";
import { useRouterStore } from "../router.store";

export type FuncId = string;

export type FuncSummary = {
  id: string;
  handler: string;
  variant: FuncVariant;
  name: string;
  displayName?: string;
  description?: string;
  isBuiltin: boolean;
};
export type FuncWithDetails = FuncSummary & {
  code: string;
  types: string;
  isRevertible: boolean;
  associations?: FuncAssociations;
};

export interface SaveFuncResponse {
  isRevertible: boolean;
  types: string;
  associations?: FuncAssociations;
  success: boolean;
}

export interface OutputLocationOption {
  label: string;
  value: OutputLocation;
}

const LOCAL_STORAGE_FUNC_IDS_KEY = "si-open-func-ids";

export const useFuncStore = () => {
  const componentsStore = useComponentsStore();
  const changeSetStore = useChangeSetsStore();
  const selectedChangeSetId = changeSetStore.selectedChangeSetId;
  const visibility: Visibility = {
    visibility_change_set_pk: selectedChangeSetId ?? nilId(),
  };

  return addStoreHooks(
    defineStore(`cs${selectedChangeSetId}/funcs`, {
      state: () => ({
        funcsById: {} as Record<FuncId, FuncSummary>,
        funcDetailsById: {} as Record<FuncId, FuncWithDetails>,
        inputSourceSockets: [] as InputSourceSocket[],
        inputSourceProps: [] as InputSourceProp[],
        outputSockets: [] as OutputSocket[],
        saveQueue: {} as Record<FuncId, (...args: unknown[]) => unknown>,
        openFuncIds: [] as FuncId[],
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

        nameForSchemaVariantId: (_state) => (schemaVariantId: string) =>
          componentsStore.schemaVariantsById[schemaVariantId]?.schemaName,

        funcById: (state) => (funcId: string) => state.funcDetailsById[funcId],

        funcList: (state) => _.values(state.funcsById),

        propForId: (state) => (propId: string) =>
          state.inputSourceProps.find((prop) => prop.propId === propId),

        outputSocketForId() {
          return (externalProviderId: string) =>
            this.outputSockets?.find(
              (socket) => socket.externalProviderId === externalProviderId,
            );
        },

        schemaVariantIdForAttributePrototype() {
          return (prototype: AttributePrototypeView) => {
            if (prototype.propId) {
              return this.propForId(prototype.propId)?.schemaVariantId;
            }

            if (prototype.externalProviderId) {
              return this.outputSocketForId(prototype.externalProviderId)
                ?.schemaVariantId;
            }
          };
        },

        outputLocationForAttributePrototype() {
          return (
            prototype: AttributePrototypeView,
          ): OutputLocation | undefined => {
            if (prototype.propId) {
              return {
                label: this.propIdToSourceName(prototype.propId) ?? "none",
                propId: prototype.propId,
              };
            }

            if (prototype.externalProviderId) {
              return {
                label:
                  this.externalProviderIdToSourceName(
                    prototype.externalProviderId,
                  ) ?? "none",
                externalProviderId: prototype.externalProviderId,
              };
            }

            return undefined;
          };
        },

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

        outputLocationOptionsForSchemaVariant(): (
          schemaVariantId: string,
        ) => OutputLocationOption[] {
          return (schemaVariantId: string) => {
            const propOptions = this.inputSourceProps
              .filter(
                (prop) =>
                  schemaVariantId === nilId() ||
                  schemaVariantId === prop.schemaVariantId,
              )
              .map((prop) => {
                const label = this.propIdToSourceName(prop.propId) ?? "none";
                return {
                  label,
                  value: {
                    label,
                    propId: prop.propId,
                  },
                };
              });

            const socketOptions = this.outputSockets
              .filter(
                (socket) =>
                  schemaVariantId === nilId() ||
                  schemaVariantId === socket.schemaVariantId,
              )
              .map((socket) => {
                const label =
                  this.externalProviderIdToSourceName(
                    socket.externalProviderId,
                  ) ?? "none";
                return {
                  label,
                  value: {
                    label,
                    externalProviderId: socket.externalProviderId,
                  },
                };
              });

            return [...propOptions, ...socketOptions];
          };
        },

        schemaVariantOptions() {
          return componentsStore.schemaVariants.map((sv) => ({
            label: sv.schemaName,
            value: sv.id,
          }));
        },
        componentOptions(): { label: string; value: string }[] {
          return componentsStore.allComponents.map(
            ({ displayName, id, schemaVariantId }) => ({
              label: `${displayName} (${
                this.nameForSchemaVariantId(schemaVariantId) ?? "unknown"
              })`,
              value: id,
            }),
          );
        },
        internalProviderIdToSourceName() {
          return (internalProviderId: string) => {
            const socket = this.inputSourceSockets?.find(
              (socket) => socket.internalProviderId === internalProviderId,
            );
            if (socket) {
              return `Input Socket: ${socket.name}`;
            }

            const prop = this.inputSourceProps.find(
              (prop) => prop.internalProviderId === internalProviderId,
            );
            if (prop) {
              return `Attribute: ${prop.path}${prop.name}`;
            }

            return undefined;
          };
        },
        propIdToSourceName() {
          return (propId: string) => {
            const prop = this.inputSourceProps.find(
              (prop) => prop.propId === propId,
            );
            if (prop) {
              return `Attribute: ${prop.path}${prop.name}`;
            }
          };
        },
        externalProviderIdToSourceName() {
          return (externalProviderId: string) => {
            const outputSocket = this.outputSockets?.find(
              (socket) => socket.externalProviderId === externalProviderId,
            );
            if (outputSocket) {
              return `Output Socket: ${outputSocket.name}`;
            }
            return undefined;
          };
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
              this.recoverOpenFuncIds();
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
          return new ApiRequest<SaveFuncResponse>({
            method: "post",
            url: "func/save_func",
            params: {
              ...func,
              ...visibility,
            },
            keyRequestStatusBy: func.id,
            onSuccess: (response) => {
              func.associations = response.associations;
              func.isRevertible = response.isRevertible;
              this.funcDetailsById[func.id] = func;

              // Forces a reload if the types have changed (reloads typescript compiler)
              if (func.types !== response.types) {
                this.FETCH_FUNC_DETAILS(func.id);
              }
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

        async SAVE_AND_EXEC_FUNC(funcId: FuncId) {
          const func = this.funcById(funcId);

          if (func) {
            trackEvent("func_save_and_exec", { id: func.id, name: func.name });
          }

          return new ApiRequest<SaveFuncResponse>({
            method: "post",
            url: "func/save_and_exec",
            keyRequestStatusBy: funcId,
            params: { ...func, ...visibility },
            onSuccess: (response) => {
              const func = this.funcDetailsById[funcId];
              if (func) {
                func.associations = response.associations;
                func.isRevertible = response.isRevertible;
                this.funcDetailsById[funcId] = func;
              }
            },
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
            onSuccess: (response) => {
              this.funcsById[response.id] = response;
            },
          });
        },

        async FETCH_INPUT_SOURCE_LIST() {
          return new ApiRequest<{
            inputSockets: InputSourceSocket[];
            outputSockets: OutputSocket[];
            props: InputSourceProp[];
          }>({
            url: "func/list_input_sources",
            params: { ...visibility },
            onSuccess: (response) => {
              this.inputSourceSockets = response.inputSockets;
              this.inputSourceProps = response.props;
              this.outputSockets = response.outputSockets;
            },
          });
        },

        async recoverOpenFuncIds() {
          // fetch the list of open funcs from localstorage
          const localStorageFuncIds = (
            storage.getItem(LOCAL_STORAGE_FUNC_IDS_KEY) ?? ""
          ).split(",") as FuncId[];
          // Filter out cached ids that don't correspond to funcs anymore
          const newOpenFuncIds = _.intersection(
            localStorageFuncIds,
            _.keys(this.funcsById),
          );
          if (!_.isEqual(newOpenFuncIds, this.openFuncIds)) {
            this.openFuncIds = newOpenFuncIds;
          }

          // if we have a url selected function, make sure it exists in the list of open funcs
          if (this.urlSelectedFuncId) {
            this.setOpenFuncId(this.urlSelectedFuncId, true, true);
          }
        },

        setOpenFuncId(id: FuncId, isOpen: boolean, unshift?: boolean) {
          if (isOpen) {
            if (!this.openFuncIds.includes(id)) {
              this.openFuncIds[unshift ? "unshift" : "push"](id);
            }
          } else {
            const funcIndex = _.indexOf(this.openFuncIds, id);
            if (funcIndex >= 0) this.openFuncIds.splice(funcIndex, 1);
          }

          storage.setItem(
            LOCAL_STORAGE_FUNC_IDS_KEY,
            this.openFuncIds.join(","),
          );
        },

        updateFuncMetadata(func: FuncWithDetails) {
          const currentCode = this.funcById(func.id)?.code ?? "";
          this.funcDetailsById[func.id] = {
            ...func,
            code: currentCode,
          };

          this.UPDATE_FUNC({
            ...func,
            code: currentCode,
          });
        },

        updateFuncCode(funcId: FuncId, code: string) {
          const func = this.funcDetailsById[funcId];
          if (func) {
            func.code = code;
            this.funcDetailsById[funcId] = func;
            this.enqueueFuncSave(funcId);
          }
        },

        enqueueFuncSave(funcId: FuncId) {
          // Lots of ways to handle this... we may want to handle this debouncing in the component itself
          // so the component has its own "draft" state that it passes back to the store when it's ready to save
          // however this should work for now, and lets the store handle this logic
          if (!this.saveQueue[funcId]) {
            this.saveQueue[funcId] = _.debounce(() => {
              const func = this.funcById(funcId);
              if (func) {
                this.UPDATE_FUNC(func);
              }
            }, 2000);
          }
          // call debounced function which will trigger sending the save to the backend
          const saveFunc = this.saveQueue[funcId];
          if (saveFunc) {
            saveFunc();
          }
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

            // add the func to the list of open ones
            this.setOpenFuncId(this.selectedFuncId, true);
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
    }),
  )();
};
