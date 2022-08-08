import { changeSet$, revision$ } from "@/observable/change_set";

export function switchToHead(): void {
  changeSet$.next(null);
  revision$.next(null);
}
