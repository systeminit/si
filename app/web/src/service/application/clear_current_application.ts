import { application$ } from "@/observable/application";

export function clearCurrentApplication() {
  application$.next(null);
}
