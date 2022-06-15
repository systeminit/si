import { ReplaySubject } from "rxjs";
import { WsDependentValuesUpdated, WsEvent } from "@/api/sdf/dal/ws_event";
import { editSessionWritten$ } from "@/observable/edit_session";

export interface DependentValuesUpdated {
  componentId: number;
  systemId: number;
}

/**
 * Fired with the ids of the component and the system
 */
export const eventDependentValuesUpdated$ =
  new ReplaySubject<WsEvent<WsDependentValuesUpdated> | null>(1);
eventDependentValuesUpdated$.next(null);

// TODO: don't do this here, check the payload before retriggering the specific action
eventDependentValuesUpdated$.subscribe(() => {
  editSessionWritten$.next(true);
});
