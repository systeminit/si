<template>
  <div
    ref="viewport"
    class="w-full h-full overflow-hidden"
    @mouseenter="mouseEnter()"
    @mouseleave="mouseLeave()"
    @mousedown="mouseDown($event)"
    @mousemove="mouseMove($event)"
    @mouseup="mouseUp($event)"
    @keydown.backspace.stop.prevent
    @keyup.backspace.stop.prevent
    @wheel="mouseWheel($event)"
  >
    <div :id="canvas.id" ref="canvas" class="absolute ">
      <svg
        :id="connection.transientConnection.elementId"
        height="100%"
        width="100%"
        class="absolute"
      />

      <SiGraphEdge
        v-if="connection.transientConnection.edge"
        :positionCtx="positionCtx"
        :key="connection.transientConnection.edge.id"
        :edge="connection.transientConnection.edge"
        :graphViewerId="id"
        :schematicPanelStoreCtx="schematicPanelStoreCtx"
      />

      <div v-if="schematic">
        <SiGraphEdge
          v-for="(edge, index) in schematic.edges"
          :positionCtx="positionCtx"
          :key="index"
          :edge="edge"
          :graphViewerId="id"
          :schematicPanelStoreCtx="schematicPanelStoreCtx"
        />
        <SiGraphNode
          v-for="(node, index) in schematic.nodes"
          ref="siGraphNode"
          :positionCtx="positionCtx"
          :key="index"
          :node="node"
          :graphViewerId="id"
          :storesCtx="storesCtx"
          @selectNode="selectNode"
        />
      </div>

      <SiBackground
        ref="siBackground"
        :resolution="canvas.resolution"
        style="default"
      />
    </div>
  </div>
</template>

<script lang="ts">
/**
 * Graph Viewer
 *
 * -> Renders graph (nodes and edges)
 * -> Renders background (grid)
 *
 * -> Handles graph manipulations (nodes and edges: transforms and connections)
 * -> Handles viewport manipulations (pan and zoom)
 *
 **/

import Vue, { PropType, Component } from "vue";
import { mapState } from "vuex";

import { InstanceStoreContext } from "@/store";
import { PanelEventBus } from "@/atoms/PanelEventBus";
import { SiCg } from "@/api/sicg";
import { Cg2dCoordinate, CgResolution } from "@/api/sicg";

import {
  SchematicPanelStore,
  NodeUpdatePositionePayload,
} from "@/store/modules/schematicPanel";
import { SessionStore } from "@/store/modules/session";
import {
  ConnectionCreatePayload,
  TransientEdgeRemovalEvent,
} from "@/store/modules/schematicPanel";

import {
  ISchematic,
  Schematic,
  ISchematicNode,
} from "@/api/sdf/model/schematic";
import { Edge, IEdge, IVertex, EdgeKind } from "@/api/sdf/model/edge";
import { INode } from "@/api/sdf/model/node";

import { EditorStore } from "@/store/modules/editor";
import {
  SetNodePositionPayload,
  NodeDeletePayload,
} from "@/store/modules/schematicPanel";

import {
  Connection,
  ConnectionNodeReference,
  ConnectionCreateReply,
  INodeUpdatePositionReply,
} from "@/api/sdf/dal/schematicDal";

import {
  ShortcutRegistrationEvent,
  ShortcutContext,
  MouseRegistrationEvent,
  MouseContext,
  ShortcutActions,
  ShortcutUpdateEvent,
} from "@/organisims/ShortcutsEventBroker.vue";

import { NodePositionUpdateEvent } from "@/organisims/SchematicViewer/Node.vue";
import {
  EdgePostionUpdateEvent,
  EdgeTemporary,
} from "@/organisims/SchematicViewer/Edge.vue";
import {
  SpaceBarEvents,
  BackspaceEvents,
} from "@/organisims/ShortcutsEventBroker.vue";

import SiBackground from "@/atoms/SiBackground.vue";
import SiGraphNode from "./SchematicViewer/Node.vue";
import SiGraphEdge from "./SchematicViewer/Edge.vue";

import _ from "lodash";

export type StoreCtx = InstanceStoreContext<SchematicPanelStore>;

export interface StoresCtx {
  [storeId: string]: StoreCtx;
}

const COMPONENT_NAMESPACE = "graphViewer";
const TRANSIENT_CONNECTION_ID = "transientConnection";
const TRANSIENT_CONNECTION_SVG_ID = "transientConnectionSvg";

type IPanelLayoutUpdated = boolean;

interface IData {
  id: string;
  canvas: {
    id: string;
    element: HTMLElement | null;
    resolution: CgResolution;
    position: {
      x: number;
      y: number;
    };
    boundingBox: {
      width: number;
      height: number;
      position: {
        x: number;
        y: number;
      };
    };
  };
  viewer: {
    mouseIsDown: boolean;
    spacebarIsDown: boolean;
    draggingMode: boolean;
    panningMode: boolean;
    shortcutsEnabled: boolean;
    isActive: boolean;
    isDragging: boolean;
    isPanning: boolean;
    isNodeConnectionMode: boolean;
    isNodeCreate: boolean;
    isNodeCreateInit: boolean;
    nodeDeselection: boolean;
  };
  viewport: {
    element: HTMLElement | null;
    pan: {
      translation: {
        x: number;
        y: number;
      };
      mouse: {
        x: number;
        y: number;
      };
      originalPosition: {
        x: number | null;
        y: number | null;
      };
      position: {
        x: number;
        y: number;
      };
    };
    zoom: {
      sensitivity: number;
      factor: number;
      min: number;
      max: number;
      translation: {
        x: number;
        y: number;
      };
      canvas: {
        position: {
          x: number;
          y: number;
        };
      };
    };
    size: {
      width: number;
      height: number;
    };
    mouse: {
      position: {
        x: number;
        y: number;
      };
      event: MouseEvent | null;
    };
  };
  selection: {
    element: HTMLElement | null;
    offset: {
      x: number;
      y: number;
    };
    position: Cg2dCoordinate;
    id: string | null;
  };
  connection: {
    transientConnection: {
      id: string;
      sourceSocketId: string;
      destinationSocketId: string;
      elementId: string;
      edge: EdgeTemporary | null;
    };
  };
}

export default Vue.extend({
  name: "SchematicViewer",
  components: {
    SiGraphEdge,
    SiGraphNode,
    SiBackground,
  },
  props: {
    schematic: {
      type: Object as PropType<SchematicPanelStore["schematic"]>,
      required: false,
    },
    schematicPanelStoreCtx: {
      type: Object as PropType<InstanceStoreContext<SchematicPanelStore>>,
      required: false,
    },
    storesCtx: {
      type: Object as PropType<StoresCtx>,
      required: false,
    },
  },
  data(): IData {
    const id = _.uniqueId(COMPONENT_NAMESPACE + ":");
    return {
      id: id,
      canvas: {
        element: null,
        resolution: {
          x: 8000,
          y: 8000,
        },
        position: {
          x: 0,
          y: 0,
        },
        id: id + "." + "canvas",
        boundingBox: {
          width: 0,
          height: 0,
          position: {
            x: 0,
            y: 0,
          },
        },
      },
      viewer: {
        shortcutsEnabled: false,
        panningMode: false,
        isPanning: false,
        draggingMode: false,
        isDragging: false,
        isActive: false,
        isNodeConnectionMode: false,
        isNodeCreate: false,
        mouseIsDown: false,
        spacebarIsDown: false,
        isNodeCreateInit: false,
        nodeDeselection: false,
      },
      viewport: {
        element: null,
        pan: {
          translation: {
            x: 0,
            y: 0,
          },
          mouse: {
            x: 0,
            y: 0,
          },
          originalPosition: {
            x: null,
            y: null,
          },
          position: {
            x: 0,
            y: 0,
          },
        },
        zoom: {
          sensitivity: 0.001,
          factor: 1,
          min: 0.2,
          max: 1,
          translation: {
            x: 0,
            y: 0,
          },
          canvas: {
            position: {
              x: 0,
              y: 0,
            },
          },
        },
        mouse: {
          position: {
            x: 0,
            y: 0,
          },
          event: null,
        },
        size: {
          width: 0,
          height: 0,
        },
      },
      selection: {
        element: null,
        offset: {
          x: 0,
          y: 0,
        },
        position: {
          x: 0,
          y: 0,
        },
        id: null,
      },
      connection: {
        transientConnection: {
          id: TRANSIENT_CONNECTION_ID,
          sourceSocketId: "",
          destinationSocketId: "",
          elementId: id + "." + TRANSIENT_CONNECTION_SVG_ID,
          edge: null,
        },
      },
    };
  },
  mounted(): void {
    this.canvas.element = this.$refs.canvas as HTMLElement;
    this.viewport.element = this.$refs.viewport as HTMLElement;
    this.setCanvasSize();
    this.setCanvasPosition();
    this.registerEvents();
  },
  beforeDestroy() {
    this.deRegisterEvents();
  },
  beforeUpdate: function() {},
  computed: {
    selectedNode(): ISchematicNode | null {
      return this.storesCtx.schematicPanelStoreCtx.state.selectedNode;
    },
    // For nodes position
    positionCtx(): string | null {
      if (this.currentSystem) {
        return this.currentSystem.id;
      } else {
        return null;
      }
    },
    ...mapState({
      currentWorkspace: (state: any): SessionStore["currentWorkspace"] =>
        state.session.currentWorkspace,
      currentSystem: (state: any): SessionStore["currentSystem"] =>
        state.session.currentSystem,
      currentApplicationContext: (state: any): EditorStore["context"] =>
        state.editor.context,
      currentChangeSet: (state: any): EditorStore["currentChangeSet"] =>
        state.editor.currentChangeSet,
      currentEditSession: (state: any): EditorStore["currentEditSession"] =>
        state.editor.currentEditSession,
      editMode(): boolean {
        return this.$store.getters["editor/inEditable"];
      },
    }),
    currentApplicationId(): string | undefined {
      return this.currentApplicationContext?.applicationId;
    },
  },
  methods: {
    registerEvents(): void {
      PanelEventBus.$on("panel-viewport-update", this.redraw);
      PanelEventBus.$on("panel-viewport-edge-remove", this.removeTemporaryEdge);

      SpaceBarEvents.subscribe(event =>
        this.spacebarEvent(event as ShortcutUpdateEvent),
      );

      BackspaceEvents.subscribe(event =>
        this.backspaceEvent(event as ShortcutUpdateEvent),
      );
    },
    deRegisterEvents(): void {
      PanelEventBus.$off("panel-viewport-update", this.redraw);
      PanelEventBus.$off(
        "panel-viewport-edge-remove",
        this.removeTransientEdge,
      );
    },
    activateShortcuts(): void {
      this.viewer.shortcutsEnabled = true;

      const ctx: ShortcutContext = {
        id: this.id,
        isActive: true,
      };
      const event: ShortcutRegistrationEvent = {
        context: ctx,
      };

      PanelEventBus.$emit("shortcuts-registration-update", event);
    },
    deactivateShortcuts(): void {
      this.viewer.shortcutsEnabled = false;

      const ctx: ShortcutContext = {
        id: this.id,
        isActive: false,
      };
      const event: ShortcutRegistrationEvent = {
        context: ctx,
      };

      PanelEventBus.$emit("shortcuts-registration-update", event);
    },
    activateViewer(): void {
      this.viewer.isActive = true;
    },
    deactivateViewer(): void {
      this.viewer.isActive = false;
      if ((this.viewer.panningMode = true)) {
        this.deactivatePanningMode();
      }
      this.viewer.draggingMode = false;
    },
    activatePanningMode(): void {
      this.viewer.panningMode = true;
      this.viewer.isPanning = true;

      if (this.viewport.element) {
        this.viewport.element.style.cursor = "grab";
      }
    },
    deactivatePanningMode(): void {
      this.viewer.panningMode = false;
      this.viewer.isPanning = false;

      if (this.viewport.element) {
        this.viewport.element.style.cursor = "default";
      }
    },
    async selectNode(node: INode) {
      await this.schematicPanelStoreCtx.dispatch("nodeSelect", node);
    },
    async clearNodeSelection() {
      await this.schematicPanelStoreCtx.dispatch("nodeSelectionClear");
    },
    onNodeCreate(nodeId: string, event: MouseEvent): void {
      this.viewer.isNodeCreate = true;
      const id = this.id + "." + nodeId;
      this.selection.element = document.getElementById(id);

      if (this.canvas.element) {
        const mouseLocalSpace = SiCg.cgGetMousePositionInElementSpace(
          event as MouseEvent,
          this.canvas.element,
        );
        if (this.selection.element) {
          const canvasBoundingRect = this.canvas.element.getBoundingClientRect();

          const node_offset = {
            x: 70,
            y: 8,
          };
          this.selection.offset.x = node_offset.x;
          this.selection.offset.y = node_offset.y;
        }
      }
    },
    setCanvasPosition() {
      if (this.viewport.element && this.canvas.element) {
        const viewportBoundingRect = this.viewport.element.getBoundingClientRect();
        const offset = {
          x: Math.abs(
            this.canvas.resolution.x * 0.5 - viewportBoundingRect.width * 0.5,
          ) as number,
          y: Math.abs(
            this.canvas.resolution.y * 0.5 -
              (viewportBoundingRect.height * 0, 5),
          ) as number,
        };

        this.viewport.size.width = viewportBoundingRect.width;
        this.viewport.size.height = viewportBoundingRect.height;

        this.canvas.position.x = -Math.abs(offset.x);
        this.canvas.position.y = -Math.abs(offset.y);

        this.canvas.element.style.left = this.canvas.position.x + "px";
        this.canvas.element.style.top = this.canvas.position.y + "px";

        this.viewport.zoom.canvas.position = {
          x: this.canvas.position.x,
          y: this.canvas.position.y,
        };
      }
    },
    updateCanvasPosition() {
      // called when maximizing/minimizing a panel

      if (this.viewport.element && this.canvas.element) {
        const viewportBoundingRect = this.viewport.element.getBoundingClientRect();

        // calculate the canvas offsets so that it is centered on the viewport.
        const offsetToCenter = {
          x: Math.abs(
            this.canvas.resolution.x * 0.5 - viewportBoundingRect.width * 0.5,
          ) as number,
          y: Math.abs(
            this.canvas.resolution.y * 0.5 -
              (viewportBoundingRect.height * 0, 5),
          ) as number,
        };

        // Compensate for the change in viewport size.
        const viewerOffset = {
          x: viewportBoundingRect.width - this.viewport.size.width,
          y: viewportBoundingRect.height - this.viewport.size.height,
        };

        const position = {
          x: offsetToCenter.x + this.canvas.position.x,
          y: offsetToCenter.y + this.canvas.position.y,
        };

        this.viewport.size.width = viewportBoundingRect.width;
        this.viewport.size.height = viewportBoundingRect.height;

        this.canvas.position.x =
          -Math.abs(offsetToCenter.x) + position.x + viewerOffset.x * 0.5;
        this.canvas.position.y =
          -Math.abs(offsetToCenter.y) + position.y + viewerOffset.y * 0.5;

        this.canvas.element.style.left = this.canvas.position.x + "px";
        this.canvas.element.style.top = this.canvas.position.y + "px";
      }
    },
    setCanvasSize() {
      const canvas = this.$refs.canvas as HTMLElement;
      SiCg.cgSetElementSize(
        canvas,
        this.canvas.resolution.x,
        this.canvas.resolution.y,
      );
    },
    newTemporaryEdge(
      edgeId: string,
      sourceNodeId: string,
      destinationNodeId: string,
    ): EdgeTemporary {
      return {
        id: edgeId,
        headVertex: {
          nodeId: destinationNodeId,
        },
        tailVertex: {
          nodeId: sourceNodeId,
        },
      };
    },
    removeTemporaryEdge() {
      if (this.connection.transientConnection.edge) {
        this.connection.transientConnection.edge = null;
      }
    },
    removeTransientEdge() {
      if (this.connection.transientConnection.id) {
        const transientConnection = document.getElementById(
          this.connection.transientConnection.id,
        );
        if (transientConnection != null) {
          transientConnection.remove();
        }
      }
    },
    redraw(event: IPanelLayoutUpdated | UIEvent) {
      this.$forceUpdate();
    },

    backspaceEvent(event: ShortcutUpdateEvent): void {
      if (event.panelId === this.id) {
        if (event.action === ShortcutActions.DeleteNode) {
          this.deleteActiveNode();
        }
      }
    },
    async deleteActiveNode() {
      if (
        this.editMode &&
        this.selectedNode &&
        this.currentApplicationId &&
        this.currentWorkspace &&
        this.currentChangeSet &&
        this.currentEditSession &&
        this.currentSystem
      ) {
        const nodeDeletePayload: NodeDeletePayload = {
          nodeId: this.selectedNode.node.id,
          applicationId: this.currentApplicationId,
          workspaceId: this.currentWorkspace.id,
          changeSetId: this.currentChangeSet.id,
          editSessionId: this.currentEditSession.id,
          systemId: this.currentSystem.id,
        };
        await this.schematicPanelStoreCtx.dispatch(
          "nodeDelete",
          nodeDeletePayload,
        );
      }
    },
    spacebarEvent(event: ShortcutUpdateEvent) {
      if (event.panelId === this.id) {
        if (event.action === ShortcutActions.StartPan) {
          this.activatePanningMode();
        }
        if (event.action === ShortcutActions.EndPan) {
          this.deactivatePanningMode();
        }
      }
    },
    spacebarDown(e: Event): void {
      this.viewer.spacebarIsDown = true;
    },
    spacebarUp(e: Event): void {
      this.viewer.spacebarIsDown = false;
    },
    mouseEnter() {
      this.activateShortcuts();
      this.activateViewer();
    },
    mouseLeave() {
      this.deactivateShortcuts();
      this.deactivateViewer();
    },
    mouseDown(e: Event): void {
      if (this.viewer.isActive) {
        this.viewer.mouseIsDown = true;

        // Initialize mouse position
        const mouseLocalSpace = SiCg.cgGetMousePositionInElementSpace(
          e as MouseEvent,
          this.canvas.element as HTMLElement,
        );

        const ctx: MouseContext = {
          id: this.id,
          isActive: true,
        };
        const event: MouseRegistrationEvent = {
          context: ctx,
        };

        // ----------------------------------------------------------------
        // Initialize canvas position
        // ----------------------------------------------------------------
        if (this.canvas.element) {
          const canvasRect = this.canvas.element.getBoundingClientRect();
          this.viewport.pan.originalPosition = {
            x: canvasRect.left,
            y: canvasRect.top,
          };
        }

        // ----------------------------------------------------------------
        // Initialize mouse position
        // ----------------------------------------------------------------
        const mouseEvent = e as MouseEvent;
        if (e) {
          this.viewport.mouse.position = {
            x: mouseEvent.clientX,
            y: mouseEvent.clientY,
          };
        }

        // ----------------------------------------------------------------
        // Initialize canvas panning
        // ----------------------------------------------------------------
        // if (this.viewer.panningMode) {
        //   this.viewer.isPanning = true;
        //   console.log("this.viewer.isPanning");
        // }

        // ----------------------------------------------------------------
        // Deselect active node
        // ----------------------------------------------------------------
        if (e.target && e.target instanceof SVGElement && e.target.id) {
          if (
            e.target.id.includes(".transientConnectionSvg") ||
            e.target.id.includes(".edge:")
          ) {
            this.clearNodeSelection();
            this.viewer.nodeDeselection = true;
          }
        }

        // ----------------------------------------------------------------
        // Initialize node dragging
        // ----------------------------------------------------------------
        if (!this.viewer.panningMode && !this.viewer.nodeDeselection) {
          this.viewer.draggingMode = true;

          // Set selected element (node)
          if (e.target && e.target instanceof HTMLDivElement) {
            if (e.target.classList.contains("node")) {
              if (this.selectedNode && this.selectedNode.node.id) {
                this.selection.id = this.selectedNode.node.id;
                const id = this.id + "." + this.selectedNode.node.id;
                this.selection.element = document.getElementById(id);
              } else {
                this.selection.id = null;
                this.selection.element = null;
              }
            } else {
              this.selection.id = null;
              this.selection.element = null;
            }
          }

          // ----------------------------------------------------------------
          // Initialize edge creation
          // ----------------------------------------------------------------
          if (
            e.target &&
            e.target instanceof HTMLElement &&
            e.target.classList.contains("socket")
          ) {
            this.connection.transientConnection.sourceSocketId = e.target.id;

            // Ccreate an edge
            const transientConnection = document.createElementNS(
              "http://www.w3.org/2000/svg",
              "line",
            );

            this.connection.transientConnection.id =
              this.connection.transientConnection.sourceSocketId +
              "." +
              "transient";
            transientConnection.id = this.connection.transientConnection.id;

            transientConnection.setAttribute("x1", String(mouseLocalSpace.x));
            transientConnection.setAttribute("y1", String(mouseLocalSpace.y));
            transientConnection.setAttribute("x2", String(mouseLocalSpace.x));
            transientConnection.setAttribute("y2", String(mouseLocalSpace.y));
            transientConnection.setAttribute("stroke", "rgb(71,99,113)");
            transientConnection.setAttribute("stroke-width", "2");

            // Add the edge to the svg
            const transientConnectionSvg = document.getElementById(
              this.connection.transientConnection.elementId,
            );

            if (transientConnectionSvg) {
              transientConnectionSvg.appendChild(transientConnection);
            }

            // Set state to update the edge when the mouse moves
            this.viewer.isNodeConnectionMode = true;
            this.selection.element = document.getElementById(
              this.connection.transientConnection.id,
            );
          }

          // ----------------------------------------------------------------
          // Set offsets
          // ----------------------------------------------------------------

          // Offsets in canvas space
          if (
            e instanceof MouseEvent &&
            this.selection.element &&
            this.canvas.element
          ) {
            this.selection.offset.x =
              mouseLocalSpace.x - this.selection.element.offsetLeft;
            this.selection.offset.y =
              mouseLocalSpace.y - this.selection.element.offsetTop;
          }
        }
      }
    },
    mouseMove(e: MouseEvent): void {
      if (this.viewer.isActive) {
        const targetElement = e.target as HTMLElement;

        // ----------------------------------------------------------------
        // Mouse is panning
        // ----------------------------------------------------------------
        if (this.viewer.panningMode && this.viewer.isPanning) {
          this.panViewport(e);
        } else if (
          this.viewer.isNodeConnectionMode &&
          this.selection.element &&
          this.editMode
        ) {
          // ----------------------------------------------------------------
          // Mouse is connecting nodes
          // ----------------------------------------------------------------
          const newPosition = SiCg.cgGetMousePositionInElementSpace(
            e as MouseEvent,
            this.canvas.element as HTMLElement,
          );
          this.selection.element.setAttribute("x2", String(newPosition.x));
          this.selection.element.setAttribute("y2", String(newPosition.y));

          if (targetElement && targetElement.classList.contains("socket")) {
            this.connection.transientConnection.destinationSocketId =
              // @ts-ignore
              targetElement.id;
          }
        } else {
          // ----------------------------------------------------------------
          // Mouse is dragging a new node
          // ----------------------------------------------------------------
          if (
            this.viewer.isNodeCreate &&
            this.selectedNode &&
            !this.viewer.isNodeCreateInit &&
            this.selectedNode.node
          ) {
            this.selection.id = this.selectedNode.node.id;
            const id = this.id + "." + this.selection.id;
            this.selection.element = document.getElementById(id);
            this.viewer.draggingMode = true;
          }

          // ----------------------------------------------------------------
          // Mouse is dragging a node
          // ----------------------------------------------------------------
          if (this.selection.element !== null) {
            if (this.viewer.mouseIsDown || this.viewer.isNodeCreate) {
              if (
                this.selection.element.classList.contains("node") &&
                this.editMode
              ) {
                this.viewer.isDragging = true;
                this.viewport.mouse.event = e as MouseEvent;

                window.requestAnimationFrame(this.dragNode);
              }
            }
          }
        }
      }
    },
    mouseUp(e: MouseEvent): void {
      if (this.viewer.isActive) {
        // ----------------------------------------------------------------
        // Mouse dragged a node
        // ----------------------------------------------------------------
        if (this.viewer.isDragging == true) {
          this.viewer.isDragging = false;
          this.viewer.draggingMode = false;
          this.viewer.isNodeCreate = false;
          this.viewer.isNodeCreateInit = false;

          this.setNodePosition(this.selection.position);

          if (this.selection.id) {
            const nodePositionUpdate: NodePositionUpdateEvent = {
              position: this.selection.position,
              nodeId: this.selection.id,
            };
            PanelEventBus.$emit(
              "panel-viewport-node-update",
              nodePositionUpdate,
            );

            const edgePositionUpdate: EdgePostionUpdateEvent = {
              nodeId: this.selection.id as string,
              nodePosition: this.selection.position,
            };

            PanelEventBus.$emit(
              "panel-viewport-edge-update",
              edgePositionUpdate,
            );
          }
          this.selection.element = null;
          this.selection.id = null;
        }

        // ----------------------------------------------------------------
        // Mouse panned the canvas
        // ----------------------------------------------------------------
        if (this.viewer.isPanning == true) {
          this.viewer.isPanning = false;

          const ctx: MouseContext = {
            id: this.id,
            isActive: false,
          };
          const event: MouseRegistrationEvent = {
            context: ctx,
          };
        }

        // ----------------------------------------------------------------
        // Mouse connected nodes
        // ----------------------------------------------------------------
        if (this.viewer.isNodeConnectionMode) {
          this.viewer.isNodeConnectionMode = false;
          this.selection.element = null;
          this.selection.id = null;
          this.connectNodes();
        }
      }

      // ----------------------------------------------------------------
      // Cleanup
      // ----------------------------------------------------------------
      this.viewer.mouseIsDown = false;
      this.viewer.isPanning = false;
      this.viewer.nodeDeselection = false;
      this.removeTransientEdge();
    },
    mouseWheel(e: MouseWheelEvent) {
      this.zoom(e);
    },
    zoom(e: MouseWheelEvent) {
      // console.log(e);

      /**
       * - Zoom on cursor implementation -
       *
       * Uses css transform matrix to scales and translate the <div> element
       * in order to simulate zooming in at a specific location.
       *
       * 1: calculate zoom scale
       * 2: calculate zoom translation
       * 3: perform transforms
       */
      e.preventDefault();

      /**
       * mouseScrollAmount
       * 1: get vertical scroll amount from the mouse event
       * 2: apply zoom sensitivity multiplier (scroll intensity)
       *
       * e.deltaY:: mouse vertical scroll
       * this.viewport.zoom.sensitivity :: our sensitivity setting
       */
      const mouseScrollAmount = e.deltaY * this.viewport.zoom.sensitivity;

      /**
       * zoomFactor :: new zoom factor
       * 1: add new mouse scroll amount to previous mouse scroll factor
       * 2: restrict mouse factor to our min and max range
       *
       * this.viewport.zoom.factor represents last zoom factor
       * this.viewport.zoom.min represents our minimum zoom setting
       * this.viewport.zoom.max represents our maximum zoom setting
       */
      let zoomFactor = this.viewport.zoom.factor + mouseScrollAmount;
      zoomFactor = Math.min(
        this.viewport.zoom.max,
        Math.max(this.viewport.zoom.min, zoomFactor),
      );

      /**
       * zoomDeltaPercentage :: difference (in percentage) between
       *  previous and new zoom factor
       *
       * this.viewport.zoom.factor :: previous zoom factor
       * zoomFactor :: new zoom factor
       */
      const zoomDeltaPercentage = 1 - zoomFactor / this.viewport.zoom.factor;
      const magnitude = zoomDeltaPercentage;

      if (this.canvas.element && this.viewport.element) {
        const mousePosition = {
          x: e.clientX - this.viewport.zoom.canvas.position.x,
          y: e.clientY - this.viewport.zoom.canvas.position.y,
        };

        const translation = {
          x: this.viewport.zoom.translation.x + mousePosition.x * magnitude,
          y: this.viewport.zoom.translation.y + mousePosition.y * magnitude,
        };
        const viewportRect = this.viewport.element.getBoundingClientRect();

        if (this.canvas.element) {
          this.canvas.element.style.transformOrigin = "0 0";
          this.canvas.element.style.transform = `matrix(${zoomFactor}, 0, 0, ${zoomFactor}, ${translation.x}, ${translation.y})`;
        }

        this.viewport.zoom.factor = zoomFactor;

        this.viewport.zoom.translation.x = translation.x;
        this.viewport.zoom.translation.y = translation.y;

        const canvasRect = this.canvas.element.getBoundingClientRect();
        this.viewport.zoom.canvas.position.x = canvasRect.x;
        this.viewport.zoom.canvas.position.y = canvasRect.y;
      }
    },
    panViewport(e: MouseEvent) {
      const mousePosition = {
        x: e.clientX,
        y: e.clientY,
      };

      if (
        this.viewport.pan.position.x != null &&
        this.viewport.pan.position.y != null &&
        this.viewport.pan.originalPosition.x != null &&
        this.viewport.pan.originalPosition.y != null &&
        this.canvas.element &&
        this.viewport.element
      ) {
        // calculate mouse movement
        const mouseMovement = {
          x: mousePosition.x - this.viewport.mouse.position.x,
          y: mousePosition.y - this.viewport.mouse.position.y,
        };

        // update viewport mouse position
        this.viewport.mouse.position = {
          x: mousePosition.x,
          y: mousePosition.y,
        };

        this.viewport.pan.position = {
          x: this.canvas.position.x + mouseMovement.x,
          y: this.canvas.position.y + mouseMovement.y,
        };

        // set canvas position
        const position: Cg2dCoordinate = {
          x: this.viewport.pan.position.x,
          y: this.viewport.pan.position.y,
        };

        const canvasRect = this.canvas.element.getBoundingClientRect();
        const viewportRect = this.viewport.element.getBoundingClientRect();

        // const panLimit = {
        //   left: 0,
        //   top: 0,
        //   right: -Math.abs(canvasRect.width - viewportRect.width),
        //   bottom: -Math.abs(canvasRect.height - viewportRect.height),
        // };

        // const panPostion = {
        //   x: Math.min(
        //     panLimit.left,
        //     Math.max(Math.min(panLimit.left, position.x), panLimit.right),
        //   ),
        //   y: Math.min(
        //     panLimit.top,
        //     Math.max(Math.min(panLimit.top, position.y), panLimit.bottom),
        //   ),
        // };

        this.canvas.element.style.left = position.x + "px";
        this.canvas.element.style.top = position.y + "px";

        // this.viewport.pan.translation = {
        //   x: this.viewport.pan.translation.x + (panPostion.x - this.canvas.position.x),
        //   y: this.viewport.pan.translation.y + (panPostion.y - this.canvas.position.y)
        // }

        this.canvas.position = {
          x: position.x,
          y: position.y,
        };

        this.viewport.zoom.canvas.position = {
          x: canvasRect.x,
          y: canvasRect.y,
        };
      }
    },
    async dragNode() {
      /*
        Notes
         - requires mouse position in canvas space
         - calculates new position for the node that is dragged
         - temporarily set the node position while the mouse is moving
         - update edges related to the noded that is dragged
         - store the new position in the Vue component
       */
      if (this.viewport.mouse.event) {
        // Get mouse position
        const mouseLocalSpace = SiCg.cgGetMousePositionInElementSpace(
          this.viewport.mouse.event,
          this.canvas.element as HTMLElement,
        );

        // define node updated position
        const position: Cg2dCoordinate = {
          x: mouseLocalSpace.x - this.selection.offset.x,
          y: mouseLocalSpace.y - this.selection.offset.y,
        };

        // Update node position
        const setNodePositionPayload: SetNodePositionPayload = {
          nodeId: this.selection.id as string,
          context: this.positionCtx as string,
          position: position,
        };
        await this.storesCtx["schematicPanelStoreCtx"].dispatch(
          "setNodePosition",
          setNodePositionPayload,
        );

        // Update node edges position
        const edgePositionUpdate: EdgePostionUpdateEvent = {
          nodeId: this.selection.id as string,
          nodePosition: position,
        };
        const eventId =
          "panel-viewport-edge-update" +
          "." +
          this.id +
          "." +
          this.selection.id;
        PanelEventBus.$emit(eventId, edgePositionUpdate);

        // store new node position
        this.selection.position = position;
      }
    },
    async setNodePosition(position: Cg2dCoordinate) {
      if (
        this.selectedNode &&
        this.selectedNode.node &&
        this.currentApplicationId &&
        this.currentWorkspace &&
        this.currentSystem
      ) {
        if (this.selectedNode.node.id) {
          const payload: NodeUpdatePositionePayload = {
            nodeId: this.selectedNode?.node.id,
            contextId: this.currentSystem.id,
            position: position,
            applicationId: this.currentApplicationId,
            workspaceId: this.currentWorkspace.id,
          };

          const reply: INodeUpdatePositionReply = await this.schematicPanelStoreCtx.dispatch(
            "nodeSetPosition",
            payload,
          );
          if (reply.error) {
            PanelEventBus.$emit("editor-error-message", reply.error.message);
          }
        }
      }
    },
    async connectNodes() {
      /*
        Notes
         - requires a source and destination sockets
         - requires a source and destination nodes
         - creates the connection between the two nodes
       */
      if (
        this.connection.transientConnection.destinationSocketId !=
        this.connection.transientConnection.sourceSocketId
      ) {
        // define source and destination nodes
        let sourceNode: string[] = [""];
        let destinationNode: string[] = [""];

        const sourceSocketType = this.connection.transientConnection.sourceSocketId
          .split(".")[2]
          .split(":")[1];
        if (sourceSocketType === "output") {
          sourceNode = this.connection.transientConnection.sourceSocketId.split(
            ".",
          );
          destinationNode = this.connection.transientConnection.destinationSocketId.split(
            ".",
          );
        } else {
          destinationNode = this.connection.transientConnection.sourceSocketId.split(
            ".",
          );
          sourceNode = this.connection.transientConnection.destinationSocketId.split(
            ".",
          );
        }
        if (
          this.storesCtx &&
          this.storesCtx.schematicPanelStoreCtx.state.schematic &&
          this.storesCtx.schematicPanelStoreCtx.state.schematic.nodes[
            sourceNode[1]
          ] &&
          this.storesCtx.schematicPanelStoreCtx.state.schematic.nodes[
            destinationNode[1]
          ]
        ) {
          // connect nodes
          const source: ConnectionNodeReference = {
            nodeId: sourceNode[1],
            socketId: sourceNode[2],
            nodeKind: this.storesCtx.schematicPanelStoreCtx.state.schematic
              .nodes[sourceNode[1]].node.objectType as string,
          };

          const destination: ConnectionNodeReference = {
            nodeId: destinationNode[1],
            socketId: destinationNode[2],
            nodeKind: this.storesCtx.schematicPanelStoreCtx.state.schematic
              .nodes[destinationNode[1]].node.objectType as string,
          };

          if (!this.edgeExistsOnGraph(source, destination, "configures")) {
            this.connection.transientConnection.edge = this.newTemporaryEdge(
              "temporaryEdge",
              source.nodeId,
              destination.nodeId,
            );
            this.removeTransientEdge();

            await this.createConnection(
              source,
              destination,
              EdgeKind.Configures,
            );
          }
          this.removeTransientEdge();

          // update node edges position
          const edgePositionUpdate: EdgePostionUpdateEvent = {
            sourceNodeId: sourceNode[1],
            destinationNodeId: destinationNode[1],
          };
          PanelEventBus.$emit("panel-viewport-edge-update", edgePositionUpdate);
        }
      }
    },
    async createConnection(
      source: ConnectionNodeReference,
      destination: ConnectionNodeReference,
      kind: EdgeKind,
    ) {
      if (
        this.currentSystem &&
        this.currentApplicationId &&
        this.currentWorkspace &&
        this.currentChangeSet &&
        this.currentEditSession
      ) {
        const connection: Connection = {
          kind: kind,
          source: source,
          destination: destination,
          systemId: this.currentSystem.id,
        };

        const payload: ConnectionCreatePayload = {
          connection: connection,
          workspaceId: this.currentWorkspace.id,
          changeSetId: this.currentChangeSet.id,
          editSessionId: this.currentEditSession.id,
          applicationId: this.currentApplicationId,
        };

        const reply: ConnectionCreateReply = await this.schematicPanelStoreCtx.dispatch(
          "connectionCreate",
          payload,
        );

        if (reply.error) {
          PanelEventBus.$emit("editor-error-message", reply.error.message);
        }
      }
    },
    edgeExistsOnGraph(
      source: ConnectionNodeReference,
      destination: ConnectionNodeReference,
      kind: string,
    ): boolean {
      if (!this.schematic) {
        throw new Error("graph must be set for edgeExists check, bug!");
      }

      return Object.values(this.schematic.edges).some(function(edge) {
        return (
          edge.headVertex.nodeId == destination.nodeId &&
          edge.tailVertex.nodeId == source.nodeId &&
          edge.kind == kind
        );
      });
    },
  },
});
</script>
