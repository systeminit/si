import * as _ from "lodash-es";
import { IconNames, Tones } from "@si/vue-lib/design-system";
import { ConnectionAnnotation } from "@si/ts-lib";
import { Vector2d } from "konva/lib/types";
import { useComponentsStore } from "@/store/components.store";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { ActorAndTimestamp, ComponentId } from "@/api/sdf/dal/component";
import { ComponentType } from "@/api/sdf/dal/schema";
import { ViewNode } from "@/api/sdf/dal/views";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  GROUP_BOTTOM_INTERNAL_PADDING,
  NODE_HEADER_HEIGHT,
  NODE_PADDING_BOTTOM,
  NODE_SUBTITLE_TEXT_HEIGHT,
  NODE_WIDTH,
  SOCKET_GAP,
  SOCKET_MARGIN_TOP,
  SOCKET_SIZE,
  SOCKET_TOP_MARGIN,
} from "./diagram_constants";

export type Bounds = {
  left: number;
  top: number;
  bottom: number;
  right: number;
};
export function toRequiredBounds({ left, right, top, bottom }: Partial<Bounds>): Bounds {
  if (left === undefined) throw new Error("no left!");
  if (right === undefined) throw new Error("no right!");
  if (top === undefined) throw new Error("no top!");
  if (bottom === undefined) throw new Error("no bottom!");
  return { left, right, top, bottom };
}
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
  abstract get def(): DiagramNodeDef | DiagramSocketDef | DiagramEdgeDef | DiagramViewDef;

  abstract get uniqueKey(): DiagramElementUniqueKey;
}

export interface DiagramSocketDataWithPosition extends DiagramSocketData {
  position: Vector2d;
}
abstract class DiagramNodeHasSockets extends DiagramElementData {
  public sockets: DiagramSocketData[];
  public diagramSockets: DiagramSocketData[]; // contains both actual sockets and data sockets which show on the diagram in Simple Socket UI
  public readonly socketStartingY: number = NODE_HEADER_HEIGHT + SOCKET_TOP_MARGIN + SOCKET_MARGIN_TOP;

  constructor(readonly def: DiagramNodeDef) {
    super();
    this.sockets = def.sockets?.map((s) => new DiagramSocketData(this, s)) || [];
    const featureFlagsStore = useFeatureFlagsStore();
    if (featureFlagsStore.SIMPLE_SOCKET_UI) {
      this.diagramSockets = [...this.diagramSocketsLeft(), ...this.diagramSocketsRight(), ...this.sockets];
    } else {
      this.diagramSockets = this.sockets;
    }
  }

  get socketEndingY() {
    return (
      SOCKET_TOP_MARGIN +
      SOCKET_MARGIN_TOP +
      SOCKET_GAP * (this.layoutLeftSockets(0).sockets.length + this.layoutRightSockets(0).sockets.length - 1) +
      SOCKET_SIZE / 2 +
      GROUP_BOTTOM_INTERNAL_PADDING
    );
  }

  inputSocketId() {
    return `${this.def.id}-inputsocket`;
  }

  outputSocketId() {
    return `${this.def.id}-outputsocket`;
  }

  diagramSocketsLeft() {
    const featureFlagsStore = useFeatureFlagsStore();

    // Always return the management socket first
    const sockets = this.sockets.filter((s) => s.def.isManagement && s.def.nodeSide === "left");

    if (featureFlagsStore.SIMPLE_SOCKET_UI) {
      sockets.push(
        new DiagramSocketData(this, {
          id: this.inputSocketId(),
          label: "Input",
          connectionAnnotations: [],
          maxConnections: null,
          direction: "input",
          nodeSide: "left",
        }),
      );
    } else {
      // Not single-socket UI, so add the real data sockets
      const dataSockets = _.sortBy(
        this.sockets.filter((s) => s.def.label !== "Frame" && s.def.nodeSide === "left" && !s.def.isManagement),
        (s) => s.def.label,
      );
      sockets.push(...dataSockets);
    }

    return sockets;
  }

  diagramSocketsRight() {
    const featureFlagsStore = useFeatureFlagsStore();

    // Always return the management socket first
    const sockets = this.sockets.filter((s) => s.def.isManagement && s.def.nodeSide === "right");

    if (featureFlagsStore.SIMPLE_SOCKET_UI) {
      sockets.push(
        new DiagramSocketData(this, {
          id: this.outputSocketId(),
          label: "Output",
          connectionAnnotations: [],
          maxConnections: null,
          direction: "output",
          nodeSide: "right",
        }),
      );
    } else {
      // Not single-socket UI, so add the real data sockets
      const dataSockets = _.sortBy(
        this.sockets.filter((s) => s.def.label !== "Frame" && s.def.nodeSide === "right" && !s.def.isManagement),
        (s) => s.def.label,
      );
      sockets.push(...dataSockets);
    }

    return sockets;
  }

  layoutLeftSockets(nodeWidth: number) {
    const sockets = this.diagramSocketsLeft();
    const layout: DiagramSocketDataWithPosition[] = [];

    for (const [i, socket] of sockets.entries()) {
      const y = i * SOCKET_GAP;
      const x = 15;
      socket.position = { x, y };
      layout.push(socket as DiagramSocketDataWithPosition);
    }
    return {
      x: (nodeWidth / 2) * -1,
      y: this.socketStartingY,
      sockets: layout,
    };
  }

  layoutRightSockets(nodeWidth: number) {
    const sockets = this.diagramSocketsRight();
    const layout: DiagramSocketDataWithPosition[] = [];

    const numLeft = this.sockets.filter((s) => s.def.nodeSide === "left").filter((s) => s.def.label !== "Frame").length;
    const leftManagementSocket = this.sockets.find((s) => s.def.isManagement && s.def.nodeSide === "left");
    let numLeftSimple = leftManagementSocket ? 2 : 1;
    if (numLeft < 2) numLeftSimple = numLeft;

    for (const [i, socket] of sockets.entries()) {
      const y = i * SOCKET_GAP;
      const x = -nodeWidth + 15;
      socket.position = { x, y };
      layout.push(socket as DiagramSocketDataWithPosition);
    }
    const featureFlagsStore = useFeatureFlagsStore();
    const socketGapMult = featureFlagsStore.SIMPLE_SOCKET_UI ? numLeftSimple : numLeft;
    return {
      x: nodeWidth / 2,
      y: this.socketStartingY + SOCKET_GAP * socketGapMult,
      sockets: layout,
    };
  }
}

export class DiagramNodeData extends DiagramNodeHasSockets {
  width: number = NODE_WIDTH;

  get uniqueKey() {
    return DiagramNodeData.generateUniqueKey(this.def.id);
  }

  get bodyHeight() {
    // PSA: This is duplicated in lang-js management func layout code. Change in both places!
    return (
      NODE_SUBTITLE_TEXT_HEIGHT +
      SOCKET_MARGIN_TOP +
      SOCKET_GAP *
        (this.layoutLeftSockets(this.width).sockets.length + this.layoutRightSockets(this.width).sockets.length - 1) +
      SOCKET_SIZE / 2 +
      // TODO: this isn't right yet!
      NODE_PADDING_BOTTOM +
      // (statusIcons?.value.length ? 30 : 0)
      30 // keeping this there as a constant for the moment
    );
  }

  get height() {
    return this.bodyHeight + NODE_HEADER_HEIGHT;
  }

  static generateUniqueKey(id: string | number) {
    return `n-${id}`;
  }

  static componentIdFromUniqueKey(uniqueKey: string): ComponentId {
    return uniqueKey.replace("n-", "");
  }
}

export class DiagramGroupData extends DiagramNodeHasSockets {
  public readonly socketStartingY: number = SOCKET_TOP_MARGIN + SOCKET_MARGIN_TOP;

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

export class DiagramViewData extends DiagramElementData {
  constructor(readonly def: DiagramViewDef) {
    super();
  }

  get uniqueKey() {
    return `v-${this.def.id}`;
  }

  static generateUniqueKey(id: string | number) {
    return `v-${id}`;
  }

  static componentIdFromUniqueKey(uniqueKey: string): string {
    return uniqueKey.replace("v-", "");
  }
}

export class DiagramSocketData extends DiagramElementData {
  position?: Vector2d;

  constructor(readonly parent: DiagramNodeData | DiagramGroupData, readonly def: DiagramSocketDef) {
    super();
  }

  get uniqueKey() {
    return DiagramSocketData.generateUniqueKey(this.parent.uniqueKey, this.def.id);
  }

  static generateUniqueKey(parentKey: string, id: string | number) {
    return `${parentKey}--s-${id}`;
  }

  // removing the circular reference in parent so this object can serialize
  toJSON() {
    return {
      def: this.def,
      uniqueKey: this.uniqueKey,
      position: this.position,
      parentId: this.parent.def.id,
      parentKey: this.parent.uniqueKey,
    };
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
    const comp = useComponentsStore().allComponentsById[this.def.fromComponentId];
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

  get simpleDisplayFromSocketKey() {
    const comp = useComponentsStore().allComponentsById[this.def.fromComponentId];
    return DiagramSocketData.generateUniqueKey(
      `${comp?.def.isGroup ? "g" : "n"}-${this.def.fromComponentId}`,
      `${this.def.fromComponentId}-outputsocket`,
    );
  }

  get simpleDisplayToSocketKey() {
    const comp = useComponentsStore().allComponentsById[this.def.toComponentId];
    return DiagramSocketData.generateUniqueKey(
      `${comp?.def.isGroup ? "g" : "n"}-${this.def.toComponentId}`,
      `${this.def.toComponentId}-inputsocket`,
    );
  }

  static generateUniqueKey(id: string | number) {
    return `e-${id}`;
  }
}

export class DiagramSocketEdgeData extends DiagramEdgeData {
  declare def: DiagramSocketEdgeDef;

  get fromSocketKey() {
    if (!("fromSocketId" in this.def)) {
      throw new Error("fromSocketId is required for fromSocketKey");
    }
    const comp = useComponentsStore().allComponentsById[this.def.fromComponentId];
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
}

export class DiagramEdgeDataWithConnectionCount extends DiagramEdgeData {
  connectionCount: number;

  constructor(readonly def: DiagramEdgeDef) {
    super(def);
    this.connectionCount = 1;
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
  schemaDocsLink?: string;
  displayName: string;
  createdInfo: ActorAndTimestamp;
  deletedInfo?: ActorAndTimestamp;
  updatedInfo: ActorAndTimestamp;
  icon: IconNames;
  resourceId: string;
};

export type DiagramSocketDirection = "input" | "output" | "bidirectional";

export type DiagramSocketDef = {
  /** unique id of the socket - should be unique across all sockets */
  id: DiagramElementId;
  /** label displayed with the socket */
  label: string;
  /** socket can only connect with sockets with compatible annotations */
  connectionAnnotations: ConnectionAnnotation[];
  /** direction of data flow from this socket */
  direction: DiagramSocketDirection;
  /** arity / max number of connections - null = no limit (most will likely be either 1 or null) */
  maxConnections: number | null;
  /** is a connection required to be valid - will affect display */
  isRequired?: boolean;
  /** which side of the node is the socket displayed on */
  nodeSide: "left" | "right"; // add top/bottom later?
  /** is the socket a management socket */
  isManagement?: boolean;
  /** schema id, for management socket compatibility */
  schemaId?: string;
  /** if there's a value propagated */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  value?: any;
  // color
  // shape
};

export type DiagramViewDef = ViewNode & {
  icon: IconNames;
  color: string;
  schemaName: string;
  schemaDocsLink?: string;
};

export interface DiagramEdgeDef {
  id: DiagramElementId;
  type?: string;
  name?: string;
  fromComponentId: DiagramElementId;
  toComponentId: DiagramElementId;
  isBidirectional?: boolean;
  isInferred?: boolean;
  isManagement?: boolean;
  /** change status of edge in relation to head */
  changeStatus?: ChangeStatus;
  createdAt?: Date;
  deletedAt?: Date;
  toDelete?: boolean;
}
export interface DiagramSocketEdgeDef extends DiagramEdgeDef {
  fromSocketId: DiagramElementId;
  toSocketId: DiagramElementId;
}
export function isDiagramSocketEdgeDef(edge: DiagramEdgeDef): edge is DiagramSocketEdgeDef {
  return "fromSocketId" in edge;
}

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
