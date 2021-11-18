import { changeSet$ } from "@/observable/change_set";

/**
 * An observable stream of the currently selected changeset, if
 * there is any.
 */
export function currentChangeSet(): typeof changeSet$ {
  return changeSet$;
}
