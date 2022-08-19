import { useObservable } from "@vueuse/rxjs";
import { ReplaySubject } from "rxjs";

export const SelectionService = {
  setSelectedComponentId,
  useSelectedComponentId,
};

const selectedComponentId$ = new ReplaySubject<number | null>();
selectedComponentId$.next(null);

function setSelectedComponentId(componentId: number | null) {
  selectedComponentId$.next(componentId);
}

function useSelectedComponentId() {
  return useObservable(selectedComponentId$);
}
