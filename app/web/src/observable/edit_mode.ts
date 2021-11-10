import { BehaviorSubject } from "rxjs";
import { persistToSession } from "@/observable/session_state";

/**
 * The system is in edit mode if the edit button has been pressed, in any
 * context.
 */
export const editMode$ = new BehaviorSubject<boolean>(false);
persistToSession("editMode", editMode$);
