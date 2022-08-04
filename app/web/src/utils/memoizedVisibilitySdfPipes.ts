import Bottle from "bottlejs";
import { SDF } from "@/api/sdf";
import { combineLatest, Observable, ObservableInput, shareReplay } from "rxjs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { standardVisibilityTriggers$ } from "@/observable/visibility";

export function memoizedVisibilitySdfPipe<
  A extends {},
  R,
  O extends ObservableInput<R>,
>(
  project: (visibility: Visibility, sdf: SDF, args?: A) => O,
): (args?: A) => Observable<R> {
  const memo: {
    [key: string]: Observable<R>;
  } = {};

  return (args?: A): Observable<R> => {
    const memoKey = args ? JSON.stringify(args) : "undefined";
    if (memoKey in memo) {
      return memo[memoKey];
    }

    memo[memoKey] = combineLatest([standardVisibilityTriggers$]).pipe(
      switchMap(([[visibility]]) => {
        const bottle = Bottle.pop("default");
        const sdf: SDF = bottle.container.SDF;
        return project(visibility, sdf, args);
      }),
      shareReplay({ bufferSize: 1, refCount: true }),
    );

    return memo[memoKey];
  };
}
