import { ReplaySubject } from "rxjs";
import { Organization } from "@/api/sdf/dal/organization";

/**
 * The currently logged in organization, or null if there isn't one.
 */
export const organization$ = new ReplaySubject<Organization | null>(1);
organization$.next(null);
