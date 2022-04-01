import { changeSet$, revision$ } from "@/observable/change_set";
import { editSession$ } from "@/observable/edit_session";
import { editMode$ } from "@/observable/edit_mode";

export function switchToHead(): void {
  changeSet$.next(null);
  editSession$.next(null);
  revision$.next(null);
  editMode$.next(false);
}
