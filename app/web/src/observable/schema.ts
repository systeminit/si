import { ReplaySubject } from "rxjs";

/**
 * Fired with the id of a schema when it is created
 */
export const schemaCreated$ = new ReplaySubject<number | null>(1);
schemaCreated$.next(null);
