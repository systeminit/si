import { IRect } from "konva/lib/types";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  DiagramEdgeData,
  DiagramElementUniqueKey,
  DiagramSocketData,
  SocketLocationInfo,
} from "@/components/ModelingDiagram/diagram_types";

export type ViewId = string;

export type Components = Record<DiagramElementUniqueKey, IRect>;
export type Edges = Record<DiagramElementUniqueKey, DiagramEdgeData>;
export type Sockets = Record<
  DiagramElementUniqueKey,
  SocketLocationInfo & DiagramSocketData
>;

export interface View {
  id: ViewId;
  name: string;
  components: Components;
  edges: Edges;
  sockets: Sockets;
}
