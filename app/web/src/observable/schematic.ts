import * as Rx from "rxjs";
import { Schematic, SchematicSchemaVariants } from "@/api/sdf/dal/schematic";

export const schematicData$ = new Rx.ReplaySubject<Schematic | null>(1);
schematicData$.next(null);

export const schematicSchemaVariants$ =
  new Rx.ReplaySubject<SchematicSchemaVariants | null>(1);
schematicSchemaVariants$.next(null);
