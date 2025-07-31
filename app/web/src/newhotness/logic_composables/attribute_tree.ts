import {
  AttributeTree,
  AttributeValue,
  Prop,
  Secret,
} from "@/workers/types/entity_kind_types";
import { AttributePath, ComponentId } from "@/api/sdf/dal/component";
import { PropKind } from "@/api/sdf/dal/prop";
import { componentTypes } from "../api_composables";

export interface AttrTree {
  id: string;
  children: AttrTree[];
  parent?: string;
  prop?: Prop;
  secret?: Secret;
  attributeValue: AttributeValue;
  isBuildable: boolean; // is my parent an array or map?
  componentId: string;
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
    componentId: data.id,
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

export const arrayAttrTreeIntoTree = (
  matches: AttrTree[],
  map: Record<string, AttrTree>,
  stopAtId?: string,
) => {
  // get new instances of all the objects with empty children arrays
  const parentsWithoutChildren = Object.values(map)
    .map((attr) => {
      return {
        ...attr,
        children: [],
      };
    })
    .reduce((map, attr) => {
      map[attr.id] = attr;
      return map;
    }, {} as Record<string, AttrTree>);

  const matchesAsTree: Record<string, AttrTree> = {};
  // work backwards from the leaf node, filling in their parents children arrays
  // make sure there are no dupes b/c matches will give us dupes
  matches.forEach((attr) => {
    const parents = [attr.parent];
    let prevPid: string | undefined;
    while (parents.length > 0) {
      const pId = parents.shift();
      if (!pId) throw new Error("no pid");
      let p: AttrTree | undefined;
      p = matchesAsTree[pId];
      if (!p) p = parentsWithoutChildren[pId];
      if (p) {
        if (prevPid) {
          const lastParent = matchesAsTree[prevPid];
          if (lastParent && !p.children.some((c) => c.id === lastParent.id))
            p.children.push(lastParent);
        } else if (!p.children.some((c) => c.id === attr.id))
          p.children.push(attr);

        matchesAsTree[p.id] = p;

        if (stopAtId && p.parent && p.id !== stopAtId)
          // dont traverse past domain
          parents.push(p.parent);
      }
      prevPid = pId;
    }
  });

  return matchesAsTree;
};

export const cleanForBulk = (t: AttrTree): AttrTree => {
  // clear out the AVs and externalSources
  // because we don't want them showing up in the form
  const m = {
    ...t,
    attributeValue: {
      ...t.attributeValue,
      value: null,
      externalSources: undefined,
    },
    children: t.children.map((_t) => cleanForBulk(_t)),
  };
  delete m.secret;
  return m;
};

export type MakePayload = (
  path: AttributePath,
  value: string,
  propKind: PropKind,
  connectingComponentId?: ComponentId,
) => componentTypes.UpdateComponentAttributesArgs;

export const makeSavePayload: MakePayload = (
  path: AttributePath,
  value: string,
  propKind: PropKind,
  connectingComponentId?: ComponentId,
): componentTypes.UpdateComponentAttributesArgs => {
  // TODO - Paul there's a better way to handle this for sure!
  let coercedVal: string | boolean | number | null = value;

  // We don't want to coerce a prop path when connecting via a subscription, so skip it (e.g. prop
  // kind is "integer", but the value is the prop path, which is a "string").
  if (!connectingComponentId) {
    if (value === "") {
      // For now, we don't allow a user to enter an empty string becuase passing an empty string is
      // effectively an unset operation.
      coercedVal = null;
    } else if (propKind === PropKind.Boolean) {
      coercedVal = value.toLowerCase() === "true" || value === "1";
    } else if (propKind === PropKind.Integer) {
      coercedVal = Math.trunc(Number(value));
    } else if (propKind === PropKind.Float) {
      coercedVal = Number(value);
    }
  }

  const payload: componentTypes.UpdateComponentAttributesArgs = {};
  payload[path] = coercedVal;
  if (connectingComponentId) {
    payload[path] = {
      $source: { component: connectingComponentId, path: coercedVal },
    };
  }

  return payload;
};
