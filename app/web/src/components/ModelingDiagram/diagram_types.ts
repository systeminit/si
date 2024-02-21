import { IconNames, Tones } from "@si/vue-lib/design-system";
import { ConnectionAnnotation } from "@si/ts-lib";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import { ChangeStatus } from "@/api/sdf/dal/change_set";

export type GridPoint = { x: number; y: number };
export type Size2D = { width: number; height: number };
export type Direction = "up" | "down" | "left" | "right";
export type SideAndCornerIdentifiers =
  | "top"
  | "bottom"
  | "right"
  | "left"
  | "top-right"
  | "top-left"
  | "bottom-right"
  | "bottom-left";
export type EdgeDisplayMode = "EDGES_OVER" | "EDGES_UNDER";

export type DiagramElementUniqueKey = string;

export abstract class DiagramElementData {
  abstract get def(): DiagramNodeDef | DiagramSocketDef | DiagramEdgeDef;

  abstract get uniqueKey(): DiagramElementUniqueKey;
}

export class DiagramNodeData extends DiagramElementData {
  public sockets: DiagramSocketData[];

  constructor(readonly def: DiagramNodeDef) {
    super();
    this.sockets =
      def.sockets?.map((s) => new DiagramSocketData(this, s)) || [];
  }

  get uniqueKey() {
    return DiagramNodeData.generateUniqueKey(this.def.id);
  }

  static generateUniqueKey(id: string | number) {
    return `n-${id}`;
  }
}

export class DiagramGroupData extends DiagramElementData {
  public sockets?: DiagramSocketData[];

  constructor(readonly def: DiagramNodeDef) {
    super();
    this.sockets =
      def.sockets?.map((s) => new DiagramSocketData(this, s)) || [];
  }

  get uniqueKey() {
    return DiagramGroupData.generateUniqueKey(this.def.id);
  }

  static generateUniqueKey(id: string | number) {
    return `g-${id}`;
  }
}

export class DiagramSocketData extends DiagramElementData {
  constructor(
    readonly parent: DiagramNodeData | DiagramGroupData,
    readonly def: DiagramSocketDef,
  ) {
    super();
  }

  get uniqueKey() {
    return DiagramSocketData.generateUniqueKey(
      this.parent.uniqueKey,
      this.def.id,
    );
  }

  // socket ids are only assumed to be unique within their parent
  static generateUniqueKey(parentKey: string, id: string | number) {
    return `${parentKey}--s-${id}`;
  }
}

export class DiagramEdgeData extends DiagramElementData {
  constructor(readonly def: DiagramEdgeDef) {
    super();
  }

  get uniqueKey() {
    return DiagramEdgeData.generateUniqueKey(this.def.id);
  }

  // helpers to get the unique key of the node and sockets this edge is connected to
  get fromNodeKey() {
    const comp = useComponentsStore().componentsById[this.def.fromComponentId];
    if (comp?.isGroup) {
      return DiagramGroupData.generateUniqueKey(this.def.fromComponentId);
    }
    return DiagramNodeData.generateUniqueKey(this.def.fromComponentId);
  }

  get toNodeKey() {
    const comp = useComponentsStore().componentsById[this.def.toComponentId];
    if (comp?.isGroup) {
      return DiagramGroupData.generateUniqueKey(this.def.toComponentId);
    }
    return DiagramNodeData.generateUniqueKey(this.def.toComponentId);
  }

  get fromSocketKey() {
    const comp = useComponentsStore().componentsById[this.def.fromComponentId];
    if (comp?.isGroup) {
      return DiagramSocketData.generateUniqueKey(
        DiagramGroupData.generateUniqueKey(this.def.fromComponentId),
        this.def.fromSocketId,
      );
    }
    return DiagramSocketData.generateUniqueKey(
      DiagramNodeData.generateUniqueKey(this.def.fromComponentId),
      this.def.fromSocketId,
    );
  }

  get toSocketKey() {
    const comp = useComponentsStore().componentsById[this.def.toComponentId];
    if (comp?.isGroup) {
      return DiagramSocketData.generateUniqueKey(
        DiagramGroupData.generateUniqueKey(this.def.toComponentId),
        this.def.toSocketId,
      );
    }
    return DiagramSocketData.generateUniqueKey(
      DiagramNodeData.generateUniqueKey(this.def.toComponentId),
      this.def.toSocketId,
    );
  }

  static generateUniqueKey(id: string | number) {
    return `e-${id}`;
  }
}

type DiagramElementId = string;

export type DiagramStatusIcon = {
  /* name/id of icon (registered in diagram config) */
  icon: IconNames;
  /* tone of icon - gets mapped to some preset colors */
  tone?: Tones;
  /* set to override specific hex color */
  color?: string;
  /* which details tab to link to */
  tabSlug?: "qualifications" | "resource";
};

export enum SchemaKind {
  Concept = "concept",
  Implementation = "implementation",
  Concrete = "concrete",
}

export enum ComponentType {
  Component = "component",
  ConfigurationFrameDown = "configurationFrameDown",
  ConfigurationFrameUp = "configurationFrameUp",
  AggregationFrame = "aggregationFrame",
}

export type DiagramNodeDef = {
  /** unique id of the node */
  id: DiagramElementId;
  /** same as id */
  componentId: ComponentId;
  /** parent frame (or whatever) id */
  parentId?: DiagramElementId;
  /** list of ancestor component ids */
  ancestorIds?: ComponentId[];
  /** node type within the context of the diagram */
  type?: string | null;
  /** category of diagram node */
  category?: string | null;
  /** title of diagram node */
  title: string;
  /** subtitle of diagram node */
  subtitle?: string | null;
  /** more text content displayed within the node */
  content?: string | null;
  /** sockets on the node */
  sockets?: DiagramSocketDef[];
  /** x,y placement of the node on the diagram */
  position: GridPoint;
  /** Size of element on diagram (for manually  resizable components) */
  size?: Size2D;
  /** single hex color to use for node theme */
  color?: string | null;
  /** icon (name/slug) used to help convey node type */
  typeIcon?: string | null;
  /** type of node - define if this is a simple component or a type of frame */
  componentType: ComponentType;
  /** array of icons (slug and colors) to show statuses */
  statusIcons?: DiagramStatusIcon[];
  /** if true, node shows the `loading` overlay */
  isLoading: boolean;
  /** the list of childIds related to the node */
  childIds?: DiagramElementId[];
  /** change status of component in relation to head */
  changeStatus?: ChangeStatus;
};

export type DiagramSocketDef = {
  /** unique id of the socket - should be unique across all sockets */
  id: DiagramElementId;
  /** label displayed with the socket */
  label: string;
  /** socket can only connect with sockets with compatible annotations */
  connectionAnnotations: ConnectionAnnotation[];
  /** direction of data flow from this socket */
  direction: "input" | "output" | "bidirectional";
  /** arity / max number of connections - null = no limit (most will likely be either 1 or null) */
  maxConnections: number | null;
  /** is a connection required to be valid - will affect display */
  isRequired?: boolean;
  /** which side of the node is the socket displayed on */
  nodeSide: "left" | "right"; // add top/bottom later?

  // color
  // shape
};

export type DiagramEdgeDef = {
  id: DiagramElementId;
  type?: string;
  name?: string;
  fromComponentId: DiagramElementId;
  fromSocketId: DiagramElementId;
  toComponentId: DiagramElementId;
  toSocketId: DiagramElementId;
  isBidirectional?: boolean;
  isInvisible?: boolean;
  /** change status of edge in relation to head */
  changeStatus?: ChangeStatus;
  createdAt?: Date;
  deletedAt?: Date;
};

export type DiagramDrawEdgeState = {
  active: boolean;
  fromSocketKey?: DiagramElementUniqueKey;
  toSocketKey?: DiagramElementUniqueKey;
  possibleTargetSocketKeys: DiagramElementUniqueKey[];
  edgeKeysToDelete: DiagramElementUniqueKey[];
};

export type MoveElementsState = {
  active: boolean;
  intoNewParentKey: DiagramElementUniqueKey | undefined;
};

// Event payloads - emitted by generic diagram //////////////////////////////////
export type ElementHoverMeta =
  | { type: "resize"; direction: SideAndCornerIdentifiers }
  | { type: "socket"; socket: DiagramSocketData }
  | { type: "parent" };

export type RightClickElementEvent = {
  element: DiagramElementData;
  e: MouseEvent;
};
