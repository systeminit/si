import { ReplaySubject } from "rxjs";
import { WsDependentValuesUpdated, WsEvent } from "@/api/sdf/dal/ws_event";

export interface DependentValuesUpdated {
  componentId: number;
  systemId: number;
}

/**
 * Fired with the ids of the component and the system
 */
export const eventDependentValuesUpdated$ =
  new ReplaySubject<WsEvent<WsDependentValuesUpdated> | null>(1);
eventDependentValuesUpdated$.next(null);
