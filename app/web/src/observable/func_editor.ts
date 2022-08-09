import { BehaviorSubject } from "rxjs";
import { Func } from "@/api/sdf/dal/func";

export interface EditingFunc {
  modifiedFunc: Func;
  origFunc: Func;
  id: number;
}

export const editingFuncs$ = new BehaviorSubject<EditingFunc[]>([]);
export const selectedTab$ = new BehaviorSubject<number>(0);
