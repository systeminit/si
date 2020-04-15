import { Props } from "./attrList";

export class PropRegistry {
  propStore: Record<string, Props>;

  constructor() {
    this.propStore = {};
  }

  add(p: Props): void {
    if (p.lookupTag) {
      this.propStore[p.lookupTag] = p;
    }
  }

  get(k: string): undefined | Props {
    return this.propStore[k];
  }
}

export const propRegistry = new PropRegistry();
