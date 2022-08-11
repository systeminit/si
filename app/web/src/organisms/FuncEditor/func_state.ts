import { reactive } from "vue";
import { EditingFunc, nullEditingFunc } from "@/observable/func_editor";
import { Func } from "@/api/sdf/dal/func";

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
  console.log("change");
  const currentFuncIdx = funcState.funcs.findIndex((f) => f.id === func.id);

  if (currentFuncIdx == -1) {
    return;
  }
  funcState.funcs[currentFuncIdx].modifiedFunc = { ...func };
};

export const removeFunc = (func: Func) => {
  funcState.funcs = funcState.funcs.filter((f) => f.id !== func.id);
};

export const clearFuncs = () => {
  funcState.funcs = [];
};
