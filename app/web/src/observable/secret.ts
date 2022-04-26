import { ReplaySubject } from "rxjs";
import { WsSecretCreated, WsEvent } from "@/api/sdf/dal/ws_event";

/**
 * Fired with the pk of the new change set when one is created.
 */
export const eventSecretCreated$ =
  new ReplaySubject<WsEvent<WsSecretCreated> | null>(1);
eventSecretCreated$.next(null);
