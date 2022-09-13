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
  components: [],
  schemaVariants: [],
};

export const funcState = reactive<{ funcs: EditingFunc[] }>({ funcs: [] });

export const insertFunc = (func: GetFuncResponse) => {
  if (!funcState.funcs.find((f) => f.id === func.id)) {
    funcState.funcs.push(func);
  }
};

export const funcById = (funcId: number) =>
  funcState.funcs.find((f) => f.id === funcId);

export const funcExists = (funcId: number) => !!funcById(funcId);

export const changeFunc = (func: GetFuncResponse) => {
  const currentFuncIdx = funcState.funcs.findIndex((f) => f.id === func.id);

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
