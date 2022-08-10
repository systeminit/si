import { BehaviorSubject } from "rxjs";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";

export interface EditingFunc {
  modifiedFunc: Func;
  origFunc: Func;
  id: number;
}

export const nullEditingFunc: EditingFunc = {
  origFunc: {
    id: 0,
    handler: undefined,
    kind: FuncBackendKind.Unset,
    name: "",
    code: "",
  },
  modifiedFunc: {
    id: 0,
    handler: undefined,
    kind: FuncBackendKind.Unset,
    name: "",
    code: "",
  },
  id: 0,
};

export const editingFuncs$ = new BehaviorSubject<EditingFunc[]>([]);
