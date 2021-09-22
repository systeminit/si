import { OpSource } from "./ops";

export interface Tombstone {
  path: string[];
  source: OpSource;
  system: "baseline" | string;
}
