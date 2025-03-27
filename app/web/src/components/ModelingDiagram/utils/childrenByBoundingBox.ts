import { IRect } from "konva/lib/types";
import { useViewsStore } from "@/store/views.store";
import { useComponentsStore } from "@/store/components.store";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramViewData,
} from "../diagram_types";
import { rectContainsAnother } from "./math";

export const FindChildrenByBoundingBox = (
  el: DiagramNodeData | DiagramGroupData,
  allowDeletedChildrenToBeFilteredOut: boolean,
): (DiagramNodeData | DiagramGroupData | DiagramViewData)[] => {
  const componentsStore = useComponentsStore();
  const viewsStore = useViewsStore();

  const cRect = el.def.isGroup
    ? viewsStore.groups[el.def.id]
    : viewsStore.components[el.def.id];
  if (!cRect) return [];

  const rect = { ...cRect };
  rect.x -= rect.width / 2;

  const nodes: (DiagramGroupData | DiagramNodeData | DiagramViewData)[] = [];
  const process = ([id, elRect]: [ComponentId, IRect]) => {
    // i do not fit inside myself
    if (el.def.id === id) return;
    const _r = { ...elRect };
    _r.x -= _r.width / 2;
    if (rectContainsAnother(rect, _r)) {
      const component = componentsStore.allComponentsById[id];
      if (component) {
        if (allowDeletedChildrenToBeFilteredOut) {
          if (
            "changeStatus" in component.def &&
            component.def.changeStatus === "deleted"
          )
            return;
          if (
            "fromBaseChangeSet" in component.def &&
            component.def.fromBaseChangeSet
          )
            return;
        }
        nodes.push(component);
      }
    }
  };

  Object.entries(viewsStore.groups).forEach(process);
  Object.entries(viewsStore.components).forEach(process);
  Object.values(viewsStore.viewNodes).forEach((viewNode) => {
    const _r = {
      x: viewNode.def.x,
      y: viewNode.def.y,
      width: viewNode.def.width,
      height: viewNode.def.height,
    };
    _r.x -= _r.width / 2;
    _r.y -= _r.height / 2;
    if (rectContainsAnother(rect, _r)) {
      nodes.push(viewNode);
    }
  });
  return nodes;
};
