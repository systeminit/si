import { IRect } from "konva/lib/types";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  DiagramElementUniqueKey,
  SocketLocationInfo,
} from "@/components/ModelingDiagram/diagram_types";

export type ViewId = string;

export type Components = Record<ComponentId, IRect>;
export type Groups = Record<ComponentId, IRect>;
export type Sockets = Record<DiagramElementUniqueKey, SocketLocationInfo>;

export interface View {
  id: ViewId;
  name: string;
  components: Components;
  groups: Groups;
  sockets: Sockets;
}

export interface ViewDescription {
  id: ViewId;
  name: string;
  isDefault: boolean;
}
export interface StringGeometry {
  x: string;
  y: string;
  width: string;
  height: string;
}
