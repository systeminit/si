import * as Rx from "rxjs";
import { Schematic } from "../../model";

export const schematicData$ = new Rx.ReplaySubject<Schematic | null>(1);
schematicData$.next(null);
