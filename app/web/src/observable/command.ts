import { Subject } from "rxjs";
import { WsCommandOutput, WsEvent } from "@/api/sdf/dal/ws_event";

/**
 * Fired with the pk of the new change set when one is created.
 */
export const eventCommandOutput$ =
  new Subject<WsEvent<WsCommandOutput> | null>();
eventCommandOutput$.next(null);
