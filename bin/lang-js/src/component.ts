export interface System {
  name: string;
}

export interface Component {
  name: string;
  system?: System;
  properties: Record<string, unknown>;
}
