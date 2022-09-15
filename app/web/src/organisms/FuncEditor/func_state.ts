import { reactive } from "vue";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { EditingFunc, saveFuncToBackend$ } from "@/observable/func";
import { SaveFuncRequest } from "@/service/func/save_func";
import { GetFuncResponse } from "@/service/func/get_func";

export const nullEditingFunc: EditingFunc = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  code: "",
  isBuiltin: false,
  isRevertable: false,
  components: [],
  schemaVariants: [],
};

export const funcState = reactive<{ funcs: EditingFunc[] }>({ funcs: [] });

const getFuncIndex = (funcId: number) =>
  funcState.funcs.findIndex((f) => f.id === funcId);

export const insertFunc = (func: GetFuncResponse) => {
  const idx = getFuncIndex(func.id);
  if (idx === -1) {
    funcState.funcs.push(func);
  } else {
    funcState.funcs[idx] = func;
  }
};

export const funcById = (funcId: number) =>
  funcState.funcs.find((f) => f.id === funcId);

export const funcExists = (funcId: number) => !!funcById(funcId);

export const setFuncRevertable = (funcId: number, revertable: boolean) => {
  const currentFuncIdx = getFuncIndex(funcId);
  if (currentFuncIdx === -1) {
    return;
  }
  funcState.funcs[currentFuncIdx].isRevertable = revertable;
};

export const changeFunc = (func: GetFuncResponse) => {
  const currentFuncIdx = getFuncIndex(func.id);
  if (currentFuncIdx === -1) {
    return;
  }
  funcState.funcs[currentFuncIdx] = { ...func };
  saveFuncToBackend$.next(func as SaveFuncRequest);
};

export const removeFunc = (func: Func) => {
  funcState.funcs = funcState.funcs.filter((f) => f.id !== func.id);
};

export const clearFuncs = () => {
  funcState.funcs = [];
};
