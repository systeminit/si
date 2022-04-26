import { ReplaySubject } from "rxjs";
import { WsResourceSynced, WsEvent } from "@/api/sdf/dal/ws_event";

export interface ResourceSyncId {
  componentId: number;
  systemId: number;
}

/**
 * Fired with the ids of the component and the system
 */
export const eventResourceSynced$ =
  new ReplaySubject<WsEvent<WsResourceSynced> | null>(1);
eventResourceSynced$.next(null);
