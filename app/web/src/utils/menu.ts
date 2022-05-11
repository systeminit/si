export interface SiMenuBase {
  name: string;
}

export interface SiMenuLeaf extends SiMenuBase {
  kind: "leaf";
  value: unknown;
}

export interface SiMenuTree extends SiMenuBase {
  kind: "tree";
  children: Array<SiMenuTree | SiMenuLeaf>;
}
