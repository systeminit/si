import * as Rx from "rxjs";

import { ComponentMetadata } from "@/service/component/get_components_metadata";

export const componentsMetadata$ = new Rx.ReplaySubject<
  ComponentMetadata[] | null
>(1);
componentsMetadata$.next(null);
