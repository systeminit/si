import { ReplaySubject } from "rxjs";
import { User } from "@/api/sdf/dal/user";

/**
 * The currently logged in user, or null if there isn't one.
 */
export const user$ = new ReplaySubject<User | null>(1);
user$.next(null);
