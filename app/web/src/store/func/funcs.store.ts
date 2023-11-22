import * as _ from "lodash-es";
import { defineStore } from "pinia";
import { watch } from "vue";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";

import storage from "local-storage-fallback"; // drop-in storage polyfill which falls back to cookies/memory
import { Visibility } from "@/api/sdf/dal/visibility";
import { FuncVariant } from "@/api/sdf/dal/func";

import { nilId } from "@/utils/nilId";
import { trackEvent } from "@/utils/tracking";
import keyedDebouncer from "@/utils/keyedDebouncer";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useAssetStore } from "@/store/asset.store";
import { useChangeSetsStore } from "../change_sets.store";
import { useRealtimeStore } from "../realtime/realtime.store";
import { useComponentsStore } from "../components.store";

import {
  AttributePrototypeView,
  CreateFuncOptions,
  FuncAssociations,
  InputSourceProp,
  InputSourceSocket,
  OutputLocation,
  OutputSocket,
} from "./types";
import { useRouterStore } from "../router.store";

export type FuncId = string;

export type FuncSummary = {
  id: string;
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

type FuncExecutionState =
  | "Create"
  | "Dispatch"
  | "Failure"
  | "Run"
  | "Start"
  | "Success";

// TODO: remove when fn log stuff gets figured out a bit deeper
/* eslint-disable @typescript-eslint/no-explicit-any */
export type FuncExecutionLog = {
  id: FuncId;
  state: FuncExecutionState;
  value?: any;
  outputStream?: any[];
  functionFailure?: any; // FunctionResultFailure
};

export interface SaveFuncResponse {
  isRevertible: boolean;
  types: string;
  associations?: FuncAssociations;
  success: boolean;
}

export interface DeleteFuncResponse {
  success: boolean;
}

export interface OutputLocationOption {
  label: string;
  value: OutputLocation;
}

const LOCAL_STORAGE_FUNC_IDS_KEY = "si-open-func-ids";

export type InputSourceProps = { [key: string]: InputSourceProp[] };
export type InputSourceSockets = { [key: string]: InputSourceSocket[] };
export type OutputSockets = { [key: string]: OutputSocket[] };

export const useFuncStore = () => {
  const componentsStore = useComponentsStore();
  const changeSetStore = useChangeSetsStore();
  const selectedChangeSetId = changeSetStore.selectedChangeSetId;
  const visibility: Visibility = {
    visibility_change_set_pk: selectedChangeSetId ?? nilId(),
  };

  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  let funcSaveDebouncer: ReturnType<typeof keyedDebouncer> | undefined;

  return addStoreHooks(
    defineStore(`ws${workspaceId || "NONE"}/cs${selectedChangeSetId}/funcs`, {
      state: () => ({
        funcsById: {} as Record<FuncId, FuncSummary>,
        funcDetailsById: {} as Record<FuncId, FuncWithDetails>,
        // map from schema variant ids to the input sources
        inputSourceSockets: {} as InputSourceSockets,
        inputSourceProps: {} as InputSourceProps,
        outputSockets: {} as OutputSockets,
        openFuncIds: [] as FuncId[],
        lastFuncExecutionLogByFuncId: {} as Record<FuncId, FuncExecutionLog>,
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

        propForId:
          (state) =>
          (propId: string): InputSourceProp | undefined => {
            for (const props of Object.values(state.inputSourceProps)) {
              const inputSourceProp = props.find(
                (prop) => prop.propId === propId,
              );
              if (inputSourceProp) {
                return inputSourceProp;
              }
            }
            return undefined;
          },

        propForInternalProviderId:
          (state) =>
          (internalProviderId: string): InputSourceProp | undefined => {
            for (const props of Object.values(state.inputSourceProps)) {
              const inputSourceProp = props.find(
                (prop) => prop.internalProviderId === internalProviderId,
              );
              if (inputSourceProp) {
                return inputSourceProp;
              }
            }
            return undefined;
          },

        inputSocketForInternalProviderId:
          (state) =>
          (internalProviderId: string): InputSourceSocket | undefined => {
            for (const sockets of Object.values(state.inputSourceSockets)) {
              const inputSourceSocket = sockets.find(
                (socket) => socket.internalProviderId === internalProviderId,
              );
              if (inputSourceSocket) {
                return inputSourceSocket;
              }
            }
            return undefined;
          },

        outputSocketForId:
          (state) =>
          (externalProviderId: string): OutputSocket | undefined => {
            for (const sockets of Object.values(state.outputSockets)) {
              const outputSocket = sockets.find(
                (socket) => socket.externalProviderId === externalProviderId,
              );
              if (outputSocket) {
                return outputSocket;
              }
            }
            return undefined;
          },

        // Filter props by schema variant
        propsAsOptionsForSchemaVariant: (state) => (schemaVariantId: string) =>
          (schemaVariantId === nilId()
            ? _.flatten(Object.values(state.inputSourceProps))
            : state.inputSourceProps[schemaVariantId]
          )?.map((prop) => ({
            label: `${prop.path}${prop.name}`,
            value: prop.propId,
          })) ?? [],

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
      },

      actions: {
        internalProviderIdToSourceName(internalProviderId: string) {
          const socket =
            this.inputSocketForInternalProviderId(internalProviderId);
          if (socket) {
            return `Input Socket: ${socket.name}`;
          }

          const prop = this.propForInternalProviderId(internalProviderId);
          if (prop) {
            return `Attribute: ${prop.path}${prop.name}`;
          }

          return undefined;
        },

        propIdToSourceName(propId: string) {
          const prop = this.propForId(propId);
          if (prop) {
            return `Attribute: ${prop.path}${prop.name}`;
          }
        },

        externalProviderIdToSourceName(externalProviderId: string) {
          const outputSocket = this.outputSocketForId(externalProviderId);
          if (outputSocket) {
            return `Output Socket: ${outputSocket.name}`;
          }
          return undefined;
        },

        schemaVariantIdForAttributePrototype(
          prototype: AttributePrototypeView,
        ) {
          if (prototype.propId) {
            return this.propForId(prototype.propId)?.schemaVariantId;
          }

          if (prototype.externalProviderId) {
            return this.outputSocketForId(prototype.externalProviderId)
              ?.schemaVariantId;
          }
        },

        outputLocationForAttributePrototype(
          prototype: AttributePrototypeView,
        ): OutputLocation | undefined {
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
        },

        outputLocationOptionsForSchemaVariant(
          schemaVariantId: string,
        ): OutputLocationOption[] {
          const propOptions =
            (schemaVariantId === nilId()
              ? _.flatten(Object.values(this.inputSourceProps))
              : this.inputSourceProps[schemaVariantId]
            )?.map((prop) => {
              const label = this.propIdToSourceName(prop.propId) ?? "none";
              return {
                label,
                value: {
                  label,
                  propId: prop.propId,
                },
              };
            }) ?? [];

          const socketOptions =
            (schemaVariantId === nilId()
              ? _.flatten(Object.values(this.outputSockets))
              : this.outputSockets[schemaVariantId]
            )?.map((socket) => {
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
            }) ?? [];
          return [...propOptions, ...socketOptions];
        },

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
        async DELETE_FUNC(funcId: FuncId) {
          return new ApiRequest<DeleteFuncResponse>({
            method: "post",
            url: "func/delete_func",
            params: {
              id: funcId,
              ...visibility,
            },
          });
        },
        async UPDATE_FUNC(func: FuncWithDetails) {
          if (changeSetStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetStore.headSelected)
            changeSetStore.creatingChangeSet = true;
          const isHead = changeSetStore.headSelected;

          const oldAssociations = this.funcDetailsById[func.id]?.associations;

          return new ApiRequest<SaveFuncResponse>({
            method: "post",
            url: "func/save_func",
            params: {
              ...func,
              ...visibility,
            },
            optimistic: () => {
              if (isHead) return () => {};

              const current = this.funcById(func.id);
              this.funcDetailsById[func.id] = {
                ...func,
                code: current?.code ?? func.code,
              };
              return () => {
                if (current) {
                  this.funcDetailsById[func.id] = current;
                } else {
                  delete this.funcDetailsById[func.id];
                }
              };
            },
            keyRequestStatusBy: func.id,
            onSuccess: (response) => {
              if (isHead) return;

              if (
                response.associations &&
                "schemaVariantIds" in response.associations &&
                oldAssociations &&
                "schemaVariantIds" in oldAssociations
              ) {
                const addedVariantIds = _.difference(
                  response.associations.schemaVariantIds,
                  oldAssociations.schemaVariantIds,
                );

                const removedVariantIds = _.difference(
                  oldAssociations.schemaVariantIds,
                  response.associations.schemaVariantIds,
                );

                for (const schemaVariantId of _.concat(
                  addedVariantIds,
                  removedVariantIds,
                )) {
                  const assetStore = useAssetStore();
                  const asset =
                    assetStore.assetBySchemaVariantId[schemaVariantId];

                  if (asset) {
                    useAssetStore().LOAD_ASSET(asset.id);
                  }
                }
              }

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
          if (changeSetStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetStore.headSelected)
            changeSetStore.creatingChangeSet = true;

          return new ApiRequest<{ success: true }>({
            method: "post",
            url: "func/revert_func",
            params: { id: funcId, ...visibility },
          });
        },

        async SAVE_AND_EXEC_FUNC(funcId: FuncId) {
          const func = this.funcById(funcId);
          if (func) {
            trackEvent("func_save_and_exec", { id: func.id, name: func.name });
          }

          if (changeSetStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetStore.headSelected)
            changeSetStore.creatingChangeSet = true;
          const isHead = changeSetStore.headSelected;

          return new ApiRequest<SaveFuncResponse>({
            method: "post",
            url: "func/save_and_exec",
            keyRequestStatusBy: funcId,
            params: { ...func, ...visibility },
            onSuccess: (response) => {
              if (isHead) return;
              const func = this.funcDetailsById[funcId];
              if (func) {
                func.associations = response.associations;
                func.isRevertible = response.isRevertible;
                this.funcDetailsById[funcId] = func;
              }
            },
          });
        },
        async GET_FUNC_LAST_EXECUTION(funcId: FuncId) {
          return new ApiRequest({
            method: "get",
            url: "func/get_func_last_execution",
            params: { id: funcId, ...visibility },
            onSuccess: (response) => {
              this.lastFuncExecutionLogByFuncId[funcId] = response;
            },
          });
        },

        async EXECUTE(executeRequest: {
          id: FuncId;
          args: unknown;
          executionKey: string;
          componentId: string;
        }) {
          const func = this.funcById(executeRequest.id);
          if (func) {
            trackEvent("function_test_execute", {
              id: func.id,
              name: func.name,
            });
          }

          return new ApiRequest<{
            id: FuncId;
            args: unknown;
            output: unknown;
            executionKey: string;
            logs: {
              stream: string;
              level: string;
              message: string;
              timestamp: string;
            }[];
          }>({
            method: "post",
            url: "func/execute",
            params: { ...executeRequest, ...visibility },
          });
        },

        async CREATE_FUNC(createFuncRequest: {
          variant: FuncVariant;
          name?: string;
          options?: CreateFuncOptions;
        }) {
          if (changeSetStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetStore.headSelected)
            changeSetStore.creatingChangeSet = true;

          return new ApiRequest<FuncSummary>({
            method: "post",
            url: "func/create_func",
            params: { ...createFuncRequest, ...visibility },
            onSuccess: (response) => {
              this.funcsById[response.id] = response;
            },
          });
        },

        async FETCH_INPUT_SOURCE_LIST(schemaVariantId?: string) {
          return new ApiRequest<{
            inputSockets: InputSourceSocket[];
            outputSockets: OutputSocket[];
            props: InputSourceProp[];
          }>({
            url: "func/list_input_sources",
            params: { schemaVariantId, ...visibility },
            onSuccess: (response) => {
              const inputSourceSockets = this.inputSourceSockets;
              const inputSourceSocketsFromResponse = _.groupBy(
                response.inputSockets,
                "schemaVariantId",
              );
              for (const schemaVariantId in inputSourceSocketsFromResponse) {
                inputSourceSockets[schemaVariantId] =
                  inputSourceSocketsFromResponse[schemaVariantId] ?? [];
              }
              this.inputSourceSockets = inputSourceSockets;

              const inputSourceProps = this.inputSourceProps;
              const inputSourcePropsFromResponse = _.groupBy(
                response.props,
                "schemaVariantId",
              );
              for (const schemaVariantId in inputSourcePropsFromResponse) {
                inputSourceProps[schemaVariantId] =
                  inputSourcePropsFromResponse[schemaVariantId] ?? [];
              }
              this.inputSourceProps = inputSourceProps;

              const outputSockets = this.outputSockets;
              const outputSocketsFromResponse = _.groupBy(
                response.outputSockets,
                "schemaVariantId",
              );
              for (const schemaVariantId in outputSocketsFromResponse) {
                outputSockets[schemaVariantId] =
                  outputSocketsFromResponse[schemaVariantId] ?? [];
              }
              this.outputSockets = outputSockets;
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

        updateFuncCode(funcId: FuncId, code: string) {
          const func = _.cloneDeep(this.funcDetailsById[funcId]);
          if (!func) return;
          func.code = code;

          this.enqueueFuncSave(func);
        },

        enqueueFuncSave(func: FuncWithDetails) {
          if (changeSetStore.headSelected) return this.UPDATE_FUNC(func);

          this.funcDetailsById[func.id] = func;

          // Lots of ways to handle this... we may want to handle this debouncing in the component itself
          // so the component has its own "draft" state that it passes back to the store when it's ready to save
          // however this should work for now, and lets the store handle this logic
          if (!funcSaveDebouncer) {
            funcSaveDebouncer = keyedDebouncer((id: FuncId) => {
              const f = this.funcDetailsById[id];
              if (!f) return;
              this.UPDATE_FUNC(f);
            }, 2000);
          }
          // call debounced function which will trigger sending the save to the backend
          const saveFunc = funcSaveDebouncer(func.id);
          if (saveFunc) {
            saveFunc(func.id);
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
