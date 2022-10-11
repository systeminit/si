import { defineStore } from "pinia";
import { bufferTime } from "rxjs/operators";
import { EditingFunc, saveFuncToBackend$ } from "@/observable/func";
import { ListedFuncView, listFuncs } from "@/service/func/list_funcs";
import { addStoreHooks } from "@/utils/pinia_hooks_plugin";
import { getFunc, GetFuncResponse } from "@/service/func/get_func";
import { revertFunc } from "@/service/func/revert_func";
import {
  createFunc,
  CreateFuncRequest,
  CreateFuncResponse,
} from "@/service/func/create_func";
import {
  listInputSources,
  ListInputSourcesResponse,
} from "@/service/func/list_input_sources";
import { Visibility } from "@/api/sdf/dal/visibility";
import { execFunc } from "@/service/func/exec_func";
import { saveFunc, SaveFuncRequest } from "@/service/func/save_func";
import { SaveFuncResponse, DevService } from "@/service/dev";
import { FuncArgument, FuncBackendKind } from "@/api/sdf/dal/func";
import { AttributePrototypeView } from "@/service/func";
import { useChangeSetsStore } from "./change_sets.store";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useComponentsStore } from "./components.store";

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

export interface FuncStore {
  funcList: ListedFuncView[];
  openFuncs: Record<FuncId, EditingFunc>;
  openFuncsList: ListedFuncView[];
  selectedFuncId: FuncId;
  editingFunc: EditingFunc;
  inputSources: ListInputSourcesResponse;
  isLoadingFuncList: boolean;
  isLoadingFunc: boolean;
}

export const useFuncStore = () => {
  const componentsStore = useComponentsStore();
  const changeSetStore = useChangeSetsStore();
  const selectedChangeSetId = changeSetStore.selectedChangeSetId;
  const visibility: Visibility = {
    visibility_change_set_pk: selectedChangeSetId ?? -1,
  };

  const store = defineStore(`cs${selectedChangeSetId}/funcs`, {
    state: (): FuncStore => ({
      funcList: [],
      openFuncs: {},
      openFuncsList: [],
      selectedFuncId: -1,
      editingFunc: nullEditingFunc,
      inputSources: { sockets: [], props: [] },
      isLoadingFuncList: false,
      isLoadingFunc: false,
    }),
    getters: {
      getFuncById:
        (state) =>
        (funcId: FuncId): EditingFunc | undefined =>
          state.openFuncs[funcId],
      selectedFuncIndex: (state) =>
        state.openFuncsList.findIndex((f) => f.id === state.selectedFuncId),
      getFuncByIndex: (state) => (index: number) => state.openFuncsList[index],
      getIndexForFunc: (state) => (funcId: FuncId) =>
        state.openFuncsList.findIndex((f) => f.id === funcId),
      selectedFunc: (state) => state.openFuncs[state.selectedFuncId],
      schemaVariantOptions() {
        return componentsStore.schemaVariants.map((sv) => ({
          label: sv.schemaName,
          value: sv.id,
        }));
      },
      componentOptions() {
        return componentsStore.schemaVariants.map((sv) => ({
          label: sv.schemaName,
          value: sv.id,
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
        const existing = this.getFuncById(funcId);
        if (existing) {
          this.selectedFuncId = funcId;
          this.ADD_FUNC_TO_OPEN_LIST(funcId);
          return existing;
        }

        await this.FETCH_FUNC(funcId);
        this.selectedFuncId = funcId;
        this.ADD_FUNC_TO_OPEN_LIST(funcId);
        return this.getFuncById(funcId);
      },
      ADD_FUNC_TO_OPEN_LIST(funcId: FuncId) {
        const func = this.getFuncById(funcId);
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
        this.isLoadingFuncList = true;
        return listFuncs(visibility, (response) => {
          this.funcList = response.funcs;
          this.isLoadingFuncList = false;
        });
      },
      async FETCH_INPUT_SOURCE_LIST() {
        return listInputSources(visibility, (response) => {
          this.inputSources = response;
        });
      },
      async FETCH_FUNC(funcId: FuncId) {
        this.isLoadingFunc = true;
        return getFunc({ ...visibility, id: funcId }, (response) => {
          this.openFuncs[response.id] = response;
          this.isLoadingFunc = false;
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
      async CREATE_FUNC(
        createFuncRequest: CreateFuncRequest,
        onSuccess: (response: CreateFuncResponse) => void,
      ) {
        return createFunc(
          {
            ...visibility,
            ...createFuncRequest,
          },
          (response) => {
            onSuccess(response);
          },
        );
      },
      async SAVE_FUNC(
        saveFuncRequest: SaveFuncRequest,
        onSuccess?: (response: SaveFuncResponse) => void,
      ) {
        return saveFunc(
          {
            ...visibility,
            ...saveFuncRequest,
          },
          (response) => {
            const { associations, isRevertible } = response;

            if (associations) {
              this.openFuncs[saveFuncRequest.id].associations = associations;
            }
            this.openFuncs[saveFuncRequest.id].isRevertible = isRevertible;

            if (onSuccess) {
              onSuccess(response);
            }
          },
        );
      },
      UPDATE_FUNC(func: GetFuncResponse) {
        this.openFuncs[func.id] = { ...func };
        saveFuncToBackend$.next(func as SaveFuncRequest);
      },
      REMOVE_ATTR_PROTOTYPE(funcId: FuncId, prototypeId: number) {
        const func = this.getFuncById(funcId);
        if (func?.associations?.type !== "attribute") {
          return;
        }

        func.associations.prototypes = func.associations.prototypes.filter(
          (proto) => proto.id !== prototypeId,
        );
        this.UPDATE_FUNC({ ...func });
      },
      UPDATE_ATTR_PROTOTYPE(funcId: FuncId, prototype: AttributePrototypeView) {
        const func = this.getFuncById(funcId);
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

        this.UPDATE_FUNC({ ...func });
      },
      UPDATE_ATTR_FUNC_ARGS(funcId: FuncId, args: FuncArgument[]) {
        const func = this.getFuncById(funcId);
        if (func?.associations?.type !== "attribute") {
          return;
        }

        this.UPDATE_FUNC({
          ...func,
          associations: {
            ...func.associations,
            arguments: [...args],
          },
        });
      },
    },
    onActivated() {
      this.FETCH_FUNC_LIST();
      this.FETCH_INPUT_SOURCE_LIST();

      const isDevMode = import.meta.env.DEV;

      const saveSub = saveFuncToBackend$
        .pipe(bufferTime(2000))
        .subscribe((saveRequests) =>
          Object.values(
            saveRequests.reduce(
              (acc, saveReq) => ({ ...acc, [saveReq.id]: saveReq }),
              {} as { [key: number]: SaveFuncRequest },
            ),
          ).forEach(async (saveReq) => {
            if (isDevMode && saveReq.isBuiltin) {
              DevService.saveBuiltinFunc(saveReq);
            } else {
              this.SAVE_FUNC(saveReq);
            }
          }),
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
        realtimeStore.unsubscribe(this.$id);
        saveSub.unsubscribe();
      };
    },
  });

  return addStoreHooks(store)();
};

export const saveFuncPromise = (
  func: SaveFuncRequest,
): Promise<SaveFuncResponse> =>
  new Promise((resolve) => {
    const funcStore = useFuncStore();
    funcStore.SAVE_FUNC(func, (response) => resolve(response));
  });

export const createFuncPromise = (
  func: CreateFuncRequest,
): Promise<CreateFuncResponse> =>
  new Promise((resolve) => {
    const funcStore = useFuncStore();
    funcStore.CREATE_FUNC(func, (response) => resolve(response));
  });
