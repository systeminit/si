import { IRect, Vector2d } from "konva/lib/types";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  DiagramEdgeData,
  DiagramElementUniqueKey,
  DiagramSocketData,
  SocketLocationInfo,
} from "@/components/ModelingDiagram/diagram_types";

export type ViewId = string;

export type Components = Record<ComponentId, IRect>;
export type Groups = Record<ComponentId, IRect>;
export type Edges = Record<DiagramElementUniqueKey, DiagramEdgeData>;
export type Sockets = Record<
  DiagramElementUniqueKey,
  SocketLocationInfo & DiagramSocketData
>;

export interface View {
  id: ViewId;
  name: string;
  components: Components;
  groups: Groups;
  edges: Edges;
  sockets: Sockets;
}
