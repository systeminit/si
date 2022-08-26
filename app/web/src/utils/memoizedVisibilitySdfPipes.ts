import Bottle from "bottlejs";
import {
  combineLatest,
  Observable,
  ObservableInput,
  shareReplay,
  ShareReplayConfig,
} from "rxjs";
import { switchMap } from "rxjs/operators";
import { tag } from "rxjs-spy/operators/tag";
import { SDF } from "@/api/sdf";
import { Visibility } from "@/api/sdf/dal/visibility";
import { standardVisibilityTriggers$ } from "@/observable/visibility";

export function memoizedVisibilitySdfPipe<A, R, O extends ObservableInput<R>>(
  project: (visibility: Visibility, sdf: SDF, args?: A) => O,
  memo: { [key: string]: Observable<R> },
  noReplay = false,
  shareReplayConfig: ShareReplayConfig = { bufferSize: 1, refCount: false },
): (args?: A) => Observable<R> {
  return (args?: A): Observable<R> => {
    const memoKey = args ? JSON.stringify(args) : "no-args";
    if (memoKey in memo) {
      return memo[memoKey];
    }

    memo[memoKey] = noReplay
      ? combineLatest([standardVisibilityTriggers$]).pipe(
          switchMap(([[visibility]]) => {
            const bottle = Bottle.pop("default");
            const sdf: SDF = bottle.container.SDF;
            return project(visibility, sdf, args);
          }),
          tag(`memoizedVisibilitySdfPipe:${memoKey}`),
        )
      : combineLatest([standardVisibilityTriggers$]).pipe(
          switchMap(([[visibility]]) => {
            const bottle = Bottle.pop("default");
            const sdf: SDF = bottle.container.SDF;
            return project(visibility, sdf, args);
          }),
          tag(`memoizedVisibilitySdfPipe:${memoKey}`),
          shareReplay(shareReplayConfig),
        );

    return memo[memoKey];
  };
}
