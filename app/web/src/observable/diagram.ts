import { ReplaySubject } from "rxjs";
import { DiagramSchemaVariants } from "@/api/sdf/dal/diagram";

export const diagramSchemaVariants$ =
  new ReplaySubject<DiagramSchemaVariants | null>(1);
diagramSchemaVariants$.next(null);
