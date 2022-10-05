import { ReplaySubject } from "rxjs";
import { ChangeSet } from "@/api/sdf/dal/change_set";

/**
 * The currently active change set, or null if there isn't one.
 */
export const changeSet$ = new ReplaySubject<ChangeSet | null>(1);
changeSet$.next(null);

function currentChangeSet(): typeof changeSet$ {
  return changeSet$;
}

export const ChangeSetService = {
  currentChangeSet,
};

// TODO: remove this
// used as a signal by a few spots adding functions...
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const eventChangeSetWritten$ = new ReplaySubject<any>(1);
eventChangeSetWritten$.next(null);
