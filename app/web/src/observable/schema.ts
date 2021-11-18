import { ReplaySubject } from "rxjs";

/**
 * Fired with the id of the new change set when one is canceled.
 */
export const eventSchemaCreated$ = new ReplaySubject<number | null>(1);
eventSchemaCreated$.next(null);
