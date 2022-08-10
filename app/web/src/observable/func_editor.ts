import { of, ReplaySubject } from "rxjs";
import { shareReplay, map, mergeMap } from "rxjs/operators";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";

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

export interface FuncEdit {
  type: "change" | "insert" | "remove";
  func: Func;
}

export const funcEdit$ = new ReplaySubject<FuncEdit>(1);

export const funcStream$ = of([] as EditingFunc[]).pipe(
  mergeMap((editingFuncs) =>
    funcEdit$.pipe(
      map(({ type, func }) => {
        switch (type) {
          case "insert":
            if (!editingFuncs.find((f) => f.id === func.id)) {
              editingFuncs.push({
                origFunc: func,
                modifiedFunc: func,
                id: func.id,
              });
            }
            break;
          case "remove":
            editingFuncs = editingFuncs.filter((f) => f.id !== func.id);
            break;
          case "change": {
            const currentFuncIdx = editingFuncs.findIndex(
              (f) => f.id === func.id,
            );
            if (currentFuncIdx == -1) {
              return editingFuncs;
            }
            editingFuncs[currentFuncIdx].modifiedFunc = { ...func };
            break;
          }
        }

        return editingFuncs;
      }),
    ),
  ),
  shareReplay(1),
);
