import { ReplaySubject } from "rxjs";
import { persistToSession } from "@/observable/session_state";
import { PanelMaximized } from "@/organisims/PanelTree/panel_types";

export type PanelContainerSize = Record<
  string,
  { width: number; height: number; hidden: boolean }
>;

const panelContainerSize: {
  [panelContainerRef: string]: ReplaySubject<PanelContainerSize | null>;
} = {};

export function createPanelContainerSizeObservable(
  panelContainerRef: string,
): ReplaySubject<PanelContainerSize | null> {
  const obs$ = new ReplaySubject<PanelContainerSize | null>(1);
  panelContainerSize[panelContainerRef] = obs$;
  persistToSession(`panelContainerSize-${panelContainerRef}`, obs$);
  return obs$;
}

export function restorePanelContainerSizeObservable(panelContainerRef: string) {
  const key = `panelContainerSize-${panelContainerRef}`;
  const item = sessionStorage.getItem(key);
  if (item) {
    return JSON.parse(item);
  } else {
    return null;
  }
}

const panelContainerMaximized: {
  [panelContainerRef: string]: ReplaySubject<PanelMaximized | null>;
} = {};

export function createPanelContainerMaximizedObservable(
  panelContainerRef: string,
): ReplaySubject<PanelMaximized | null> {
  const obs$ = new ReplaySubject<PanelMaximized | null>(1);
  panelContainerMaximized[panelContainerRef] = obs$;
  persistToSession(`panelContainerMaximized-${panelContainerRef}`, obs$);
  return obs$;
}

export function restorePanelContainerMaximizedObservable(
  panelContainerRef: string,
) {
  const key = `panelContainerMaximized-${panelContainerRef}`;
  const item = sessionStorage.getItem(key);
  if (item) {
    return JSON.parse(item);
  } else {
    return null;
  }
}

const panelMaximizedFull: {
  [panelRef: string]: ReplaySubject<boolean | null>;
} = {};

export function createPanelMaximizedFullObservable(
  panelRef: string,
): ReplaySubject<boolean | null> {
  const obs$ = new ReplaySubject<boolean | null>(1);
  panelMaximizedFull[panelRef] = obs$;
  persistToSession(`panelMaximizedFull-${panelRef}`, obs$);
  return obs$;
}

export function restorePanelMaximizedFullObservable(panelRef: string) {
  const key = `panelMaximizedFull-${panelRef}`;
  const item = sessionStorage.getItem(key);
  if (item) {
    return JSON.parse(item);
  } else {
    return null;
  }
}

const panelMaximizedContainer: {
  [panelRef: string]: ReplaySubject<boolean | null>;
} = {};

export function createPanelMaximizedContainerObservable(
  panelRef: string,
): ReplaySubject<boolean | null> {
  const obs$ = new ReplaySubject<boolean | null>(1);
  panelMaximizedContainer[panelRef] = obs$;
  persistToSession(`panelMaximizedContainer-${panelRef}`, obs$);
  return obs$;
}

export function restorePanelMaximizedContainerObservable(panelRef: string) {
  const key = `panelMaximizedContainer-${panelRef}`;
  const item = sessionStorage.getItem(key);
  if (item) {
    return JSON.parse(item);
  } else {
    return null;
  }
}
