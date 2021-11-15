import { ReplaySubject } from "rxjs";
import { Workspace } from "@/api/sdf/dal/workspace";
import { persistToSession } from "@/observable/session_state";

/**
 * The currently logged in workspace, or null if there isn't one.
 */
export const workspace$ = new ReplaySubject<Workspace | null>(1);
workspace$.next(null);
persistToSession("workspace", workspace$);
