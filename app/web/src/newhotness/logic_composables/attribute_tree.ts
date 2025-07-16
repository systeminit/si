import {
  AttributeTree,
  AttributeValue,
  Prop,
  Secret,
} from "@/workers/types/entity_kind_types";

export interface AttrTree {
  id: string;
  children: AttrTree[];
  parent?: string;
  prop?: Prop;
  secret?: Secret;
  attributeValue: AttributeValue;
  isBuildable: boolean; // is my parent an array or map?
}

export const makeAvTree = (
  data: AttributeTree,
  avId: string,
  isBuildable: boolean,
  parent?: string,
): AttrTree => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const av = data.attributeValues[avId]!;
  const prop = av.propId ? data.props[av.propId] : undefined;
  const secret = av.secret ?? undefined;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const childrenIds = data.treeInfo[avId]!.children;
  const children = childrenIds.map((id) =>
    makeAvTree(data, id, ["array", "map"].includes(prop?.kind ?? ""), avId),
  );
  const tree: AttrTree = {
    id: avId,
    children,
    parent,
    attributeValue: av,
    prop,
    secret,
    isBuildable,
  };
  return tree;
};
