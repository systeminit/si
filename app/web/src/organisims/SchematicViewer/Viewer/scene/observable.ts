import * as Rx from "rxjs";
import { SceneGraphData } from "./sceneManager";

export const sceneGraphData$ = new Rx.ReplaySubject<SceneGraphData | null>(1);
sceneGraphData$.next(null);

export const nodeUpdate$ = new Rx.ReplaySubject<string | null>(1);
nodeUpdate$.next(null);

export const createConnection$ = new Rx.ReplaySubject<string | null>(1);
createConnection$.next(null);

export const deleteConnection$ = new Rx.ReplaySubject<string | null>(1);
deleteConnection$.next(null);

export function schematicData(): Rx.ReplaySubject<SceneGraphData | null> {
  console.log("creating a new schematicData observable");
  const o = new Rx.ReplaySubject<SceneGraphData | null>(1);
  nodeUpdate$.next(null);
  return o;
}
