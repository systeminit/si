import { Subject } from "rxjs";
import { WsCommandReturn, WsCommandOutput, WsEvent } from "@/api/sdf/dal/ws_event";

export const eventCommandOutput$ =
  new Subject<WsEvent<WsCommandOutput> | null>();
eventCommandOutput$.next(null);

export const eventCommandReturn$ =
  new Subject<WsEvent<WsCommandReturn> | null>();
eventCommandReturn$.next(null);
