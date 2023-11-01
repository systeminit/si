// The actor entitiy that initiates an activitiy--this could represent be a
// person, service, etc.
export type ActorView =
  | { kind: "system"; label: string; email?: string }
  | { kind: "user"; label: string; id?: string; email?: string };
