import { ReplaySubject } from "rxjs";
import { WsResourceRefreshed, WsEvent } from "@/api/sdf/dal/ws_event";

export interface ResourceRefreshId {
  componentId: number;
  systemId: number;
}

/**
 * Fired with the ids of the component and the system
 */
export const eventResourceRefreshed$ =
  new ReplaySubject<WsEvent<WsResourceRefreshed> | null>(1);
eventResourceRefreshed$.next(null);
