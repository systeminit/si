import { reactive } from "vue";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { saveFuncToBackend$ } from "@/observable/func";
import { SaveFuncRequest } from "@/service/func/save_func";

export interface EditingFunc {
  modifiedFunc: Func;
  origFunc: Func;
  id: number;
}

export const nullEditingFunc: EditingFunc = {
  origFunc: {
    id: 0,
    handler: "",
    kind: FuncBackendKind.Unset,
    name: "",
    code: "",
  },
  modifiedFunc: {
    id: 0,
    handler: "",
    kind: FuncBackendKind.Unset,
    name: "",
    code: "",
  },
  id: 0,
};

export const funcState = reactive<{ funcs: EditingFunc[] }>({ funcs: [] });

export const insertFunc = (func: Func) => {
  if (!funcState.funcs.find((f) => f.id === func.id)) {
    funcState.funcs.push({
      origFunc: func,
      modifiedFunc: func,
      id: func.id,
    });
  }
};

export const funcById = (funcId: number) =>
  funcState.funcs.find((f) => f.id === funcId);

export const funcExists = (funcId: number) => !!funcById(funcId);

export const changeFunc = (func: Func) => {
  const currentFuncIdx = funcState.funcs.findIndex((f) => f.id === func.id);

  if (currentFuncIdx == -1) {
    return;
  }
  funcState.funcs[currentFuncIdx].modifiedFunc = { ...func };
  saveFuncToBackend$.next(func as SaveFuncRequest);
};

export const removeFunc = (func: Func) => {
  funcState.funcs = funcState.funcs.filter((f) => f.id !== func.id);
};

export const clearFuncs = () => {
  funcState.funcs = [];
};
