import { ReplaySubject } from "rxjs";
import { Schematic, SchematicSchemaVariants } from "@/api/sdf/dal/schematic";

export const schematicData$ = new ReplaySubject<Schematic | null>(1);
schematicData$.next(null);

export const schematicSchemaVariants$ =
  new ReplaySubject<SchematicSchemaVariants | null>(1);
schematicSchemaVariants$.next(null);
