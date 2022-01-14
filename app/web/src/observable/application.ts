import { ReplaySubject } from "rxjs";
import { Component } from "@/api/sdf/dal/component";
import { persistToSession } from "@/observable/session_state";

/**
 * The currently selected application
 */
export const application$ = new ReplaySubject<Component | null>(1);
application$.next(null);
persistToSession("application", application$);

/**
 * The currently selected applications node id
 */
export const applicationNodeId$ = new ReplaySubject<number | null>(1);
applicationNodeId$.next(null);
persistToSession("applicationNodeId", applicationNodeId$);
