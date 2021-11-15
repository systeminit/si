import {
  BehaviorSubject,
  combineLatest,
  Observable,
  share,
  Subscription,
  tap,
} from "rxjs";

export const sessionSaveEnabled$ = new BehaviorSubject<boolean>(false);
export const sessionSubscriptions: {
  [key: string]: {
    subscription: Subscription;
    observable: Observable<unknown>;
  };
} = {};

export function persistToSession(key: string, obs: Observable<unknown>): void {
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

function saveObservable(key: string, value: any): void {
  sessionStorage.setItem(key, JSON.stringify(value));
}

export function restoreFromSession(): void {
  sessionSaveEnabled$.next(false);
  for (const key in sessionSubscriptions) {
    restoreObservable(key, sessionSubscriptions[key].observable);
  }
  sessionSaveEnabled$.next(true);
}

function restoreObservable(key: string, observable: any): void {
  const json = sessionStorage.getItem(key);
  if (json) {
    const data = JSON.parse(json);
    observable.next(data);
  }
}
