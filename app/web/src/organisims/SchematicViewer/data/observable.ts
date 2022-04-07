import * as Rx from "rxjs";

import { ComponentMetadata } from "@/service/component/get_components_metadata";
import { Schematic } from "../model";

export const schematicData$ = new Rx.ReplaySubject<Schematic | null>(1);
schematicData$.next(null);

export const componentsMetadata$ = new Rx.ReplaySubject<
  ComponentMetadata[] | null
>(1);
componentsMetadata$.next(null);
