import { Subject } from "rxjs";
import { WsCommandOutput, WsEvent } from "@/api/sdf/dal/ws_event";

export const eventCommandOutput$ =
  new Subject<WsEvent<WsCommandOutput> | null>();
eventCommandOutput$.next(null);
