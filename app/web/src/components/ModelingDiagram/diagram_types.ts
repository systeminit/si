import { IconNames, Tones } from "@si/vue-lib/design-system";
import { ConnectionAnnotation } from "@si/ts-lib";
import { Vector2d } from "konva/lib/types";
import { useComponentsStore } from "@/store/components.store";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { ActorAndTimestamp, ComponentId } from "@/api/sdf/dal/component";
import { ComponentType } from "@/api/sdf/dal/schema";

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

  static componentIdFromUniqueKey(uniqueKey: string): ComponentId {
    return uniqueKey.replace("n-", "");
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

  static componentIdFromUniqueKey(uniqueKey: string): string {
    return uniqueKey.replace("g-", "");
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

  static generateUniqueKey(parentKey: string, id: string | number) {
    return `${parentKey}--s-${id}`;
  }
}

export type SocketLocationInfo = { center: Vector2d };

export class DiagramEdgeData extends DiagramElementData {
  // populated from the diagram
  fromPoint: SocketLocationInfo | undefined;
  toPoint: SocketLocationInfo | undefined;

  constructor(readonly def: DiagramEdgeDef) {
    super();
  }

  get uniqueKey() {
    return DiagramEdgeData.generateUniqueKey(this.def.id);
  }

  // helpers to get the unique key of the node and sockets this edge is connected to
  get fromNodeKey() {
    const comp =
      useComponentsStore().allComponentsById[this.def.fromComponentId];
    if (comp?.def.isGroup) {
      return DiagramGroupData.generateUniqueKey(this.def.fromComponentId);
    }
    return DiagramNodeData.generateUniqueKey(this.def.fromComponentId);
  }

  get toNodeKey() {
    const comp = useComponentsStore().allComponentsById[this.def.toComponentId];
    if (comp?.def.isGroup) {
      return DiagramGroupData.generateUniqueKey(this.def.toComponentId);
    }
    return DiagramNodeData.generateUniqueKey(this.def.toComponentId);
  }

  get fromSocketKey() {
    const comp =
      useComponentsStore().allComponentsById[this.def.fromComponentId];
    if (comp?.def.isGroup) {
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
    const comp = useComponentsStore().allComponentsById[this.def.toComponentId];
    if (comp?.def.isGroup) {
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
  color: string;
  /** icon (name/slug) used to help convey node type */
  typeIcon: string;
  /** type of node - define if this is a simple component or a type of frame */
  componentType: ComponentType;
  /** type of node - define if this is a simple component or a type of frame */
  isGroup: boolean;
  /** the list of childIds related to the node */
  childIds?: DiagramElementId[];
  /** change status of component in relation to head */
  changeStatus: ChangeStatus;
  /** component will be deleted after next action run */
  toDelete: boolean;
  /** component is deleted in this changeset, but exists in base change set */
  fromBaseChangeSet: boolean;
  /** can the component be upgraded */
  canBeUpgraded: boolean;
  /** whether the component has a resource  */
  hasResource: boolean;
  /** stats from recursive children */
  numChildren: number;
  numChildrenResources: number;
  schemaName: string;
  schemaVariantName: string;
  schemaVariantId: string;
  displayName: string;
  createdInfo: ActorAndTimestamp;
  deletedInfo?: ActorAndTimestamp;
  updatedInfo: ActorAndTimestamp;
  icon: IconNames;
  resourceId: string;
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
  isInferred?: boolean;
  /** change status of edge in relation to head */
  changeStatus?: ChangeStatus;
  createdAt?: Date;
  deletedAt?: Date;
  toDelete: boolean;
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
  | { type: "parent" }
  | { type: "rename" };

export type RightClickElementEvent = {
  element: DiagramElementData;
  e: MouseEvent;
};
