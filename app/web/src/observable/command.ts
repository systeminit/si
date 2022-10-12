import { Subject } from "rxjs";
import {
  WsCommandReturn,
  WsCommandOutput,
  WsFixReturn,
  WsEvent,
} from "@/api/sdf/dal/ws_event";

export const eventCommandOutput$ =
  new Subject<WsEvent<WsCommandOutput> | null>();
eventCommandOutput$.next(null);

export const eventCommandReturn$ =
  new Subject<WsEvent<WsCommandReturn> | null>();
eventCommandReturn$.next(null);

export const eventFixReturn$ =
  new Subject<WsEvent<WsFixReturn> | null>();
eventFixReturn$.next(null);
