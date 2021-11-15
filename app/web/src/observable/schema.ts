import { BehaviorSubject } from "rxjs";

/**
 * Triggered when a list of schemas needs to be refreshed, either because of
 * local action or remote behavior.
 */
export const schemaListRefresh$ = new BehaviorSubject<boolean>(true);
