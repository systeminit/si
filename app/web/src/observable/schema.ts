import { ReplaySubject } from "rxjs";

/**
 * Fired with the id of the new change set when one is canceled.
 */
export const schemaCreated$ = new ReplaySubject<number | null>(1);
schemaCreated$.next(null);
