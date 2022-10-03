import { reactive } from "vue";
import { Func, FuncArgument, FuncBackendKind } from "@/api/sdf/dal/func";
import { EditingFunc, saveFuncToBackend$ } from "@/observable/func";
import { SaveFuncRequest } from "@/service/func/save_func";
import { GetFuncResponse } from "@/service/func/get_func";
import { AttributePrototypeView, FuncAssociations } from "@/service/func";

export const nullEditingFunc: EditingFunc = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  code: "",
  isBuiltin: false,
  isRevertible: false,
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

export const updateFuncFromSave = (
  funcId: number,
  revertible: boolean,
  associations?: FuncAssociations,
) => {
  const currentFuncIdx = getFuncIndex(funcId);
  if (currentFuncIdx === -1) {
    return;
  }

  funcState.funcs[currentFuncIdx].isRevertible = revertible;
  if (associations) {
    funcState.funcs[currentFuncIdx].associations = associations;
  }
};

export const removeAttributePrototype = (
  funcId: number,
  prototypeId: number,
) => {
  const currentFuncIdx = getFuncIndex(funcId);
  if (currentFuncIdx === -1) {
    return;
  }

  // This is code duplicatey but we need to narrow the type here
  const currentFunc = funcState.funcs[currentFuncIdx];
  if (currentFunc.associations?.type !== "attribute") {
    return;
  }

  currentFunc.associations.prototypes =
    currentFunc.associations.prototypes.filter(
      (proto) => proto.id !== prototypeId,
    );

  changeFunc({
    ...currentFunc,
  });
};

// Add or change an attribute prototype
export const saveAttributePrototype = (
  funcId: number,
  prototype: AttributePrototypeView,
) => {
  const currentFuncIdx = getFuncIndex(funcId);
  if (currentFuncIdx === -1) {
    return;
  }
  const currentFunc = funcState.funcs[currentFuncIdx];
  if (currentFunc.associations?.type !== "attribute") {
    return;
  }

  const associations = currentFunc.associations;

  if (prototype.id === -1) {
    associations.prototypes.push(prototype);
  } else {
    const currentPrototypeIdx = associations.prototypes.findIndex(
      (proto) => proto.id === prototype.id,
    );
    associations.prototypes[currentPrototypeIdx] = prototype;
  }

  currentFunc.associations = { ...associations };
  changeFunc(currentFunc);
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

export const updateAttributeFuncArguments = (
  funcId: number,
  args: FuncArgument[],
) => {
  const func = funcById(funcId);
  if (
    !func ||
    func.kind !== FuncBackendKind.JsAttribute ||
    func.associations?.type !== "attribute"
  ) {
    return;
  }

  console.log([...args]);

  changeFunc({
    ...func,
    associations: {
      ...func.associations,
      arguments: [...args],
    },
  });
};

export const clearFuncs = () => {
  funcState.funcs = [];
};
