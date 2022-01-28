import { ReplaySubject } from "rxjs";
import { WsResourceSyncedSaved, WsEvent } from "@/api/sdf/dal/ws_event";

export interface ResourceSyncId {
  componentId: number;
  resourceId: number;
}

/**
 * Fired with the ids of the component and the system
 */
export const eventResourceSynced$ = new ReplaySubject<WsEvent<WsResourceSynced> | null>(
  1,
);
eventResourceSynced$.next(null);
