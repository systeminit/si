import { editSession$ } from "@/observable/edit_session";

/**
 * An observable stream of the currently selected editsession, if
 * there is any.
 */
export function currentEditSession(): typeof editSession$ {
  return editSession$;
}
