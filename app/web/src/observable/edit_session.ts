import { ReplaySubject } from "rxjs";
import { EditSession } from "@/api/sdf/dal/edit_session";
import { persistToSession } from "@/observable/session_state";

export const editSession$ = new ReplaySubject<EditSession | null>(1);
editSession$.next(null);
persistToSession("editSession", editSession$);
