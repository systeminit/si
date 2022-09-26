import { PropKind } from "./prop";

export interface PropTreeNode {
  children: PropTreeNode[];
  parentId: number;
  propId: number;
  kind: PropKind;
  schemaVariantId: number;
  internalProviderId?: number;
  jsonPointer: string;
}
