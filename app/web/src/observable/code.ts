import { ReplaySubject } from "rxjs";
import { WsCodeGenerated, WsEvent } from "@/api/sdf/dal/ws_event";

export interface CodeGenerationId {
  componentId: number;
  systemId: number;
}

/**
 * Fired with the ids of the component and the system
 */
export const eventCodeGenerated$ =
  new ReplaySubject<WsEvent<WsCodeGenerated> | null>(1);
eventCodeGenerated$.next(null);
