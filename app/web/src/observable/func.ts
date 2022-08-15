import { ReplaySubject } from "rxjs";

import { SaveFuncRequest } from "@/service/func/save_func";

export const saveFuncToBackend$ = new ReplaySubject<SaveFuncRequest>(1);
