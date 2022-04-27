import { ReplaySubject } from "rxjs";
import { WsCheckedQualifications, WsEvent } from "@/api/sdf/dal/ws_event";

export interface CheckedQualificationId {
  componentId: number;
  systemId: number;
}

/**
 * Fired with the ids of the component and the system
 */
export const eventCheckedQualifications$ =
  new ReplaySubject<WsEvent<WsCheckedQualifications> | null>(1);
eventCheckedQualifications$.next(null);
