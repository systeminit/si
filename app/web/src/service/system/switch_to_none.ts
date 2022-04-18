import { system$ } from "../../observable/system";

export function switchToNone(): void {
  system$.next(null);
}
