import { ReplaySubject, tap } from "rxjs";
import { Component } from "@/api/sdf/dal/component";

/**
 * The currently selected application
 */
export const application$ = new ReplaySubject<Component | null>(1);
application$.next(null);
application$.subscribe((a) => console.log("application$", { a }));
