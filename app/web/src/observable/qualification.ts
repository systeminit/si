import { Subject } from "rxjs";
import { WsCheckedQualifications, WsEvent } from "@/api/sdf/dal/ws_event";

export interface CheckedQualificationId {
  prototypeId: number;
  componentId: number;
  systemId: number;
}

/**
 * Fired with the ids of the component and the system
 */
export const eventCheckedQualifications$ =
  new Subject<WsEvent<WsCheckedQualifications> | null>();
eventCheckedQualifications$.next(null);
