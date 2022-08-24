import { ReplaySubject } from "rxjs";
import { SchematicSchemaVariants } from "@/api/sdf/dal/schematic";

export const schematicSchemaVariants$ =
  new ReplaySubject<SchematicSchemaVariants | null>(1);
schematicSchemaVariants$.next(null);
