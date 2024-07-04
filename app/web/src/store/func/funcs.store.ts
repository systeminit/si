import * as _ from "lodash-es";
import { defineStore } from "pinia";
import { addStoreHooks, ApiRequest } from "@si/vue-lib/pinia";
import { Visibility } from "@/api/sdf/dal/visibility";
import {
  FuncArgument,
  FuncId,
  FuncArgumentId,
  FuncSummary,
  FuncCode,
  FuncBinding,
  FuncBindingKind,
  FuncKind,
  Action,
  Attribute,
  CodeGeneration,
  Authentication,
  Qualification,
} from "@/api/sdf/dal/func";

import { nilId } from "@/utils/nilId";
import { trackEvent } from "@/utils/tracking";
import keyedDebouncer from "@/utils/keyedDebouncer";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useAssetStore } from "@/store/asset.store";
import { useChangeSetsStore } from "../change_sets.store";
import { useRealtimeStore } from "../realtime/realtime.store";
import { useComponentsStore } from "../components.store";

import { FuncRunId } from "../func_runs.store";

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

export interface DeleteFuncResponse {
  success: boolean;
  name: string;
}

export const useFuncStore = () => {
  const componentsStore = useComponentsStore();
  const changeSetsStore = useChangeSetsStore();
  const selectedChangeSetId = changeSetsStore.selectedChangeSet?.id;

  // TODO(nick): we need to allow for empty visibility here. Temporarily send down "nil" to mean that we want the
  // query to find the default change set.
  const visibility: Visibility = {
    visibility_change_set_pk:
      selectedChangeSetId ?? changeSetsStore.headChangeSetId ?? nilId(),
  };

  const workspacesStore = useWorkspacesStore();
  const workspaceId = workspacesStore.selectedWorkspacePk;

  let funcSaveDebouncer: ReturnType<typeof keyedDebouncer> | undefined;

  const processBindings = (func: FuncSummary) => {
    const actionBindings = [] as Action[];
    const attributeBindings = [] as Attribute[];
    const authenticationBindings = [] as Authentication[];
    const codegenBindings = [] as CodeGeneration[];
    const qualificationBindings = [] as Qualification[];

    func.bindings.forEach((binding) => {
      switch (binding.bindingKind) {
        case FuncBindingKind.Action:
          actionBindings.push(binding as Action);
          break;
        case FuncBindingKind.Attribute:
          attributeBindings.push(binding as Attribute);
          break;
        case FuncBindingKind.Authentication:
          authenticationBindings.push(binding as Authentication);
          break;
        case FuncBindingKind.CodeGeneration:
          codegenBindings.push(binding as CodeGeneration);
          break;
        case FuncBindingKind.Qualification:
          qualificationBindings.push(binding as Qualification);
          break;
        default:
          throw new Error(`Unexpected FuncBinding ${JSON.stringify(binding)}`);
      }
    });

    return {
      actionBindings,
      attributeBindings,
      authenticationBindings,
      codegenBindings,
      qualificationBindings,
    };
  };

  const API_PREFIX = `v2/workspaces/${workspaceId}/change-sets/${selectedChangeSetId}/funcs`;

  return addStoreHooks(
    defineStore(`ws${workspaceId || "NONE"}/cs${selectedChangeSetId}/funcs`, {
      state: () => ({
        // this powers the list
        funcsById: {} as Record<FuncId, FuncSummary>,
        // this is the code
        funcCodeById: {} as Record<FuncId, FuncCode>,
        // bindings
        actionBindings: {} as Record<FuncId, Action[]>,
        attributeBindings: {} as Record<FuncId, Attribute[]>,
        authenticationBindings: {} as Record<FuncId, Authentication[]>,
        codegenBindings: {} as Record<FuncId, CodeGeneration[]>,
        qualificationBindings: {} as Record<FuncId, Qualification[]>,
        // represents the last, or "focused" func clicked on/open by the editor
        selectedFuncId: undefined as FuncId | undefined,
        editingFuncLatestCode: {} as Record<FuncId, string>,
      }),
      getters: {
        selectedFuncSummary(state): FuncSummary | undefined {
          return state.funcsById[this.selectedFuncId || ""];
        },
        selectedFuncCode(state): FuncCode | undefined {
          return state.funcCodeById[this.selectedFuncId || ""];
        },

        nameForSchemaVariantId: (_state) => (schemaVariantId: string) =>
          componentsStore.schemaVariantsById[schemaVariantId]?.schemaName,

        funcList: (state) => _.values(state.funcsById),
      },

      actions: {
        async FETCH_FUNC_LIST() {
          return new ApiRequest<FuncSummary[], Visibility>({
            url: `${API_PREFIX}`,
            onSuccess: (response) => {
              response.forEach((func) => {
                const bindings = processBindings(func);
                this.actionBindings[func.funcId] = bindings.actionBindings;
                this.attributeBindings[func.funcId] =
                  bindings.attributeBindings;
                this.authenticationBindings[func.funcId] =
                  bindings.authenticationBindings;
                this.qualificationBindings[func.funcId] =
                  bindings.qualificationBindings;
                this.codegenBindings[func.funcId] = bindings.codegenBindings;
              });

              this.funcsById = _.keyBy(response, (f) => f.funcId);
            },
          });
        },
        async FETCH_CODE(funcId: FuncId) {
          return new ApiRequest<FuncCode[]>({
            url: `${API_PREFIX}/code`,
            params: {
              id: funcId,
            },
            keyRequestStatusBy: funcId,
            onSuccess: (response) => {
              response.forEach((func: FuncCode) => {
                this.funcCodeById[func.funcId] = func;
              });
            },
          });
        },
        async CREATE_FUNC(createFuncRequest: {
          name: string;
          displayName: string;
          description: string;
          kind: FuncKind;
          binding: FuncBinding;
        }) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<{ summary: FuncSummary; code: FuncCode }>({
            method: "post",
            url: `${API_PREFIX}/create`,
            params: { ...createFuncRequest },
            onSuccess: (response) => {
              // summary coming through the WsEvent
              this.funcCodeById[response.code.funcId] = response.code;
              // select the fn to load it in the editor done in the component
            },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
          });
        },
        async CREATE_UNLOCKED_COPY(funcId: FuncId) {
          return new ApiRequest<{ summary: FuncSummary; code: FuncCode }>({
            method: "post",
            url: `${API_PREFIX}/${funcId}/create_unlocked_copy`,
            onSuccess: (response) => {
              this.funcsById[response.summary.funcId] = response.summary;
              this.funcCodeById[response.code.funcId] = response.code;
            },
          });
        },
        async DELETE_UNLOCKED_FUNC(funcId: FuncId) {
          return new ApiRequest<DeleteFuncResponse>({
            method: "post",
            url: `${API_PREFIX}/${funcId}/delete`,
          });
        },
        async UPDATE_FUNC(func: FuncSummary) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;
          const isHead = changeSetsStore.headSelected;

          return new ApiRequest({
            method: "post",
            url: `${API_PREFIX}/${func.funcId}/update`,
            params: {
              displayName: func.displayName,
              description: func.description,
            },
            optimistic: () => {
              if (isHead) return () => {};

              const current = this.funcsById[func.funcId];
              return () => {
                if (current) {
                  this.funcsById[func.funcId] = current;
                } else {
                  delete this.funcCodeById[func.funcId];
                  delete this.funcsById[func.funcId];
                }
              };
            },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
            keyRequestStatusBy: func.funcId,
          });
        },
        async CREATE_BINDING(funcId: FuncId, bindings: FuncBinding[]) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          // TODO: jobelenus, handle this WsEvent (no funcId on it?!)
          return new ApiRequest<null>({
            method: "post",
            url: `${API_PREFIX}/${funcId}/bindings/create`,
            params: {
              bindings,
            },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
          });
        },
        async UPDATE_BINDING(funcId: FuncId, bindings: FuncBinding[]) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<null>({
            method: "post",
            url: `${API_PREFIX}/${funcId}/bindings/update`,
            params: {
              funcId,
              bindings,
            },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
          });
        },
        // How you "DETACH" an attribute function
        async RESET_ATTRIBUTE_BINDING(funcId: FuncId, bindings: FuncBinding[]) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<null>({
            method: "post",
            url: `${API_PREFIX}/${funcId}/reset_attribute_binding`,
            params: {
              bindings,
            },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
          });
        },
        // How you "DETACH" all other function bindings
        async DELETE_BINDING(funcId: FuncId, bindings: FuncBinding[]) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<null>({
            method: "post",
            url: `${API_PREFIX}/${funcId}/bindings/delete`,
            params: {
              bindings,
            },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
          });
        },
        async CREATE_FUNC_ARGUMENT(funcId: FuncId, funcArg: FuncArgument) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<null>({
            method: "post",
            url: `${API_PREFIX}/${funcId}/create_argument`,
            params: {
              ...funcArg,
            },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
          });
        },
        async UPDATE_FUNC_ARGUMENT(funcId: FuncId, funcArg: FuncArgument) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<null>({
            method: "post",
            url: `${API_PREFIX}/${funcId}/${funcArg.id}/update`,
            params: {
              ...funcArg,
            },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
          });
        },
        async DELETE_FUNC_ARGUMENT(
          funcId: FuncId,
          funcArgumentId: FuncArgumentId,
        ) {
          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          return new ApiRequest<null>({
            method: "post",
            url: `${API_PREFIX}/${funcId}/${funcArgumentId}/delete`,
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
          });
        },
        // TODO: jobelenus, is this still "save"? or *just* execute?
        async SAVE_AND_EXEC_FUNC(funcId: FuncId) {
          const func = this.funcsById[funcId];
          if (func) {
            trackEvent("func_save_and_exec", {
              id: func.funcId,
              name: func.name,
            });
          }

          if (changeSetsStore.creatingChangeSet)
            throw new Error("race, wait until the change set is created");
          if (changeSetsStore.headSelected)
            changeSetsStore.creatingChangeSet = true;

          return new ApiRequest({
            method: "post",
            url: `${API_PREFIX}/${funcId}/execute`,
            keyRequestStatusBy: funcId,
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
          });
        },
        async FETCH_PROTOTYPE_ARGUMENTS(
          propId?: string,
          outputSocketId?: string,
        ) {
          return new ApiRequest<{
            preparedArguments: Record<string, unknown>;
          }>({
            url: "attribute/get_prototype_arguments",
            params: { propId, outputSocketId, ...visibility },
          });
        },
        async TEST_EXECUTE(executeRequest: {
          funcId: FuncId;
          args: unknown;
          code: string;
          componentId: string;
        }) {
          const func = this.funcsById[executeRequest.funcId];
          if (func) {
            trackEvent("function_test_execute", {
              id: func.funcId,
              name: func.name,
            });
          }

          // why aren't we doing anything with the result of this?!
          return new ApiRequest<{
            funcRunId: FuncRunId;
          }>({
            method: "post",
            url: `${API_PREFIX}/${executeRequest.funcId}/test_execute`,
            params: { ...executeRequest },
          });
        },

        updateFuncCode(funcId: FuncId, code: string) {
          const func = _.cloneDeep(this.funcCodeById[funcId]);
          if (!func || func.code === code) return;
          func.code = code;

          this.enqueueFuncSave(func);
        },

        enqueueFuncSave(func: FuncCode) {
          this.editingFuncLatestCode[func.funcId] = func.code;

          // Lots of ways to handle this... we may want to handle this debouncing in the component itself
          // so the component has its own "draft" state that it passes back to the store when it's ready to save
          // however this should work for now, and lets the store handle this logic
          if (!funcSaveDebouncer) {
            funcSaveDebouncer = keyedDebouncer((id: FuncId) => {
              const f = this.funcCodeById[id];
              const code = this.editingFuncLatestCode[id];
              if (!f || !code) return;
              f.code = code;
              this.SAVE_FUNC(f);
            }, 500);
          }
          // call debounced function which will trigger sending the save to the backend
          const saveFunc = funcSaveDebouncer(func.funcId);
          if (saveFunc) {
            saveFunc(func.funcId);
          }
        },

        async SAVE_FUNC(func: FuncCode) {
          return new ApiRequest<FuncCode>({
            method: "post",
            url: `${API_PREFIX}/${func.funcId}/save_code`,
            params: { code: func.code },
            onFail: () => {
              changeSetsStore.creatingChangeSet = false;
            },
          });
        },
      },
      onActivated() {
        this.FETCH_FUNC_LIST();

        const assetStore = useAssetStore();
        const realtimeStore = useRealtimeStore();

        realtimeStore.subscribe(this.$id, `changeset/${selectedChangeSetId}`, [
          {
            eventType: "ChangeSetWritten",
            callback: () => {
              this.FETCH_FUNC_LIST();
            },
          },
          {
            eventType: "FuncBindingsUpdated",
            callback: (data) => {
              if (data.changeSetId !== selectedChangeSetId) return;
              // all the bindings for one given func
              const funcId = data.bindings[0]?.funcId;
              if (funcId) {
                const func = this.funcsById[funcId];
                if (func) func.bindings = data.bindings;
              }
            },
          },
          {
            eventType: "FuncCreated",
            callback: (data) => {
              if (data.changeSetId !== selectedChangeSetId) return;
              this.funcsById[data.funcSummary.funcId] = data.funcSummary;
              const bindings = processBindings(data.funcSummary);
              this.actionBindings[data.funcSummary.funcId] =
                bindings.actionBindings;
              this.attributeBindings[data.funcSummary.funcId] =
                bindings.attributeBindings;
              this.authenticationBindings[data.funcSummary.funcId] =
                bindings.authenticationBindings;
              this.qualificationBindings[data.funcSummary.funcId] =
                bindings.qualificationBindings;
              this.codegenBindings[data.funcSummary.funcId] =
                bindings.codegenBindings;
            },
          },
          {
            eventType: "FuncUpdated",
            callback: (data) => {
              if (data.changeSetId !== selectedChangeSetId) return;
              this.funcsById[data.funcSummary.funcId] = data.funcSummary;
              const bindings = processBindings(data.funcSummary);
              this.actionBindings[data.funcSummary.funcId] =
                bindings.actionBindings;
              this.attributeBindings[data.funcSummary.funcId] =
                bindings.attributeBindings;
              this.authenticationBindings[data.funcSummary.funcId] =
                bindings.authenticationBindings;
              this.qualificationBindings[data.funcSummary.funcId] =
                bindings.qualificationBindings;
              this.codegenBindings[data.funcSummary.funcId] =
                bindings.codegenBindings;
            },
          },
          {
            eventType: "FuncDeleted",
            callback: (data) => {
              if (data.changeSetId !== selectedChangeSetId) return;
              this.FETCH_FUNC_LIST();
            },
          },
          {
            eventType: "FuncSaved",
            callback: (data) => {
              if (data.changeSetId !== selectedChangeSetId) return;
              // TODO: jobelenus, send data over the wire so i dont need this call
              this.FETCH_FUNC_LIST();

              // Reload the last selected asset to ensure that its func list is up to date.
              // TODO: jobelenus, move this to the asset store!
              const assetId = assetStore.selectedVariantId;
              if (assetId) {
                assetStore.LOAD_SCHEMA_VARIANT(assetId);
              }

              // TODO, i dont know how this would ever fire to someone sitting on head?
              // wouldn't we listen for an event "changeset applied"???
              if (this.selectedFuncId) {
                // Only fetch if we don't have the selected func in our state or if we are on HEAD.
                // If we are on HEAD, the func is immutable, so we are safe to fetch. However, if
                // we are not on HEAD, then the func is mutable. Therefore, we can only fetch
                // relevant metadata in order to avoid overwriting functions with their previous
                // value before the save queue is drained.
                if (data.funcId === this.selectedFuncId) {
                  if (
                    typeof this.funcCodeById[this.selectedFuncId] ===
                      "undefined" ||
                    changeSetsStore.headSelected
                  ) {
                    this.FETCH_CODE(this.selectedFuncId);
                  }
                }
              }
            },
          },
        ]);
        return () => {
          realtimeStore.unsubscribe(this.$id);
        };
      },
    }),
  )();
};
