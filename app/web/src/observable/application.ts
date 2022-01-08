import { ReplaySubject, tap } from "rxjs";
import { Component } from "@/api/sdf/dal/component";

/**
 * The currently selected application
 */
export const application$ = new ReplaySubject<Component | null>(1);
application$.next(null);

/**
 * The currently selected applications node id
 */
export const application_node_id$ = new ReplaySubject<number | null>(1);
application_node_id$.next(null);
