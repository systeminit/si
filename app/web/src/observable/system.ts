import { ReplaySubject } from "rxjs";
import { System } from "@/api/sdf/dal/system";
import { persistToSession } from "@/observable/session_state";

/**
 * The currently logged in associated system, or null if there isn't one.
 */
export const system$ = new ReplaySubject<System | null>(1);
system$.next(null);
persistToSession("system", system$);
