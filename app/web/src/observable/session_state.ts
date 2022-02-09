import {
  BehaviorSubject,
  combineLatest,
  share,
  Subject,
  Subscription,
  tap,
} from "rxjs";
import _ from "lodash";

export const sessionSaveEnabled$ = new BehaviorSubject<boolean>(false);
export const sessionSubscriptions: {
  [key: string]: {
    subscription: Subscription;
    observable: Subject<unknown>;
  };
} = {};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function persistToSession(key: string, obs: Subject<any>): void {
  const observable = combineLatest([obs, sessionSaveEnabled$]).pipe(
    tap(([obs, sessionSaveEnabled]) => {
      if (sessionSaveEnabled) {
        saveObservable(key, obs);
      }
    }),
    share(),
  );
  const subscription = observable.subscribe();
  sessionSubscriptions[key] = { observable: obs, subscription };
}

export function saveObservable(key: string, value: unknown): void {
  if (_.isUndefined(value)) {
    return;
  }
  sessionStorage.setItem(key, JSON.stringify(value));
}

export function restoreFromSession(): void {
  sessionSaveEnabled$.next(false);
  for (const key in sessionSubscriptions) {
    restoreObservable(key, sessionSubscriptions[key].observable);
  }
  sessionSaveEnabled$.next(true);
}

function restoreObservable(key: string, obs: Subject<unknown>): void {
  const json = sessionStorage.getItem(key);
  if (json) {
    const data = JSON.parse(json);
    obs.next(data);
  }
}
