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
    <div :id="canvas.id" ref="canvas" class="absolute">
      <svg
        :id="connection.transientConnection.elementId"
        height="100%"
        width="100%"
        class="absolute"
      />

      <SiGraphEdge
        v-if="connection.transientConnection.edge"
        :positionCtx="positionCtx"
        :orientation="schematicOrientation()"
        :key="connection.transientConnection.edge.id"
        :edge="connection.transientConnection.edge"
        :schematic="schematic"
        :graphViewerId="id"
      />

      <div v-if="schematic">
        <div v-for="edge in schematic.edges" :key="edge.id">
          <SiGraphEdge
            :positionCtx="positionCtx"
            :orientation="schematicOrientation()"
            :edge="edge"
            :schematic="schematic"
            :graphViewerId="id"
          />
        </div>

        <div v-for="node in schematic.nodes" :key="node.node.id">
          <SiGraphNode
            :positionCtx="positionCtx"
            :orientation="schematicOrientation()"
            :node="node"
            :schematic="schematic"
            :graphViewerId="id"
            :selectedNode="selectedNode"
            :deploymentSelectedNode="deploymentSelectedNode"
            :schematicKind="schematicKind"
            @selectNode="selectNode"
          />
        </div>
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

import Vue, { PropType } from "vue";
import _ from "lodash";

import { PanelEventBus, emitEditorErrorMessage } from "@/atoms/PanelEventBus";
import { SiCg } from "@/api/sicg";
import { Cg2dCoordinate, CgResolution } from "@/api/sicg";
import {
  SchematicKind,
  ISchematicNode,
  Schematic,
} from "@/api/sdf/model/schematic";
import { EdgeKind } from "@/api/sdf/model/edge";
import {
  Connection,
  ConnectionNodeReference,
  ConnectionCreateReply,
  INodeUpdatePositionReply,
  SchematicDal,
  INodeDeleteRequest,
  INodeDeleteReply,
  INodeUpdatePositionRequest,
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
import SiGraphNode from "@/organisims/SchematicViewer/Node.vue";
import SiGraphEdge from "@/organisims/SchematicViewer/Edge.vue";
import {
  schematicSelectNode$,
  applicationId$,
  editMode$,
  workspace$,
  system$,
  changeSet$,
  editSession$,
  deploymentSchematicSelectNode$,
  schematicUpdated$,
  nodePositionUpdated$,
  edgeCreating$,
  refreshChangesSummary$,
  nodeDeleted$,
} from "@/observables";
import { SiEntity } from "si-entity";
import { Entity } from "@/api/sdf/model/entity";
import { Input } from "si-registry/dist/registryEntry";

export interface SetNodePositionPayload {
  nodeId: string;
  context: string;
  position: Cg2dCoordinate;
}

export enum SchematicOrientation {
  Vertical = "vertical",
  Horizontal = "horizontal",
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
    event: {
      mouse: MouseEvent | null;
    };
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
    element: string | null;
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
      type: Object as PropType<Schematic>,
      required: false,
    },
    schematicKind: {
      type: String as PropType<SchematicKind>,
      required: false,
      default: undefined,
    },
    positionCtx: {
      type: String,
    },
    rootObjectId: {
      type: String,
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
        event: {
          mouse: null,
        },
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
  subscriptions: function (this: any): Record<string, any> {
    return {
      selectedNode: schematicSelectNode$,
      deploymentSelectedNode: deploymentSchematicSelectNode$,
      editMode: editMode$,
      currentWorkspace: workspace$,
      currentSystem: system$,
      currentApplicationId: applicationId$,
      currentChangeSet: changeSet$,
      currentEditSession: editSession$,
    };
  },
  methods: {
    registerEvents(): void {
      // ALEX: DISABLED (please keep arround)
      // PanelEventBus.$on("panel-viewport-update", this.redraw);

      PanelEventBus.$on("panel-viewport-edge-remove", this.removeTemporaryEdge);

      SpaceBarEvents.subscribe((event) =>
        this.spacebarEvent(event as ShortcutUpdateEvent),
      );

      BackspaceEvents.subscribe((event) =>
        this.backspaceEvent(event as ShortcutUpdateEvent),
      );

      document.addEventListener("keydown", this.handleKeyDown);
      document.addEventListener("keyup", this.handleKeyUp);
    },
    deRegisterEvents(): void {
      // ALEX: DISABLED (please keep arround)
      // PanelEventBus.$off("panel-viewport-update", this.redraw);

      PanelEventBus.$off(
        "panel-viewport-edge-remove",
        this.removeTransientEdge,
      );

      document.removeEventListener("keydown", this.handleKeyDown);
      document.removeEventListener("keyup", this.handleKeyUp);
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
    handleKeyDown(e: KeyboardEvent) {
      if (this.viewer.isActive) {
        if (e.key === "Spacebar" || e.key === " ") {
          e.preventDefault();
          this.viewer.spacebarIsDown = true;
          this.viewer.panningMode = true;
        }
      }
    },
    handleKeyUp(e: KeyboardEvent) {
      if (this.viewer.isActive) {
        if (e.key === "Spacebar" || e.key === " ") {
          e.preventDefault();
          this.viewer.spacebarIsDown = false;
          this.viewer.panningMode = false;
        }
      }
    },

    //edgeKind(): EdgeKind | undefined {
    //  switch (this.schematicKind) {
    //    case SchematicKind.Deployment: {
    //      return EdgeKind.Deployment;
    //    }

    //    case SchematicKind.Component: {
    //      return EdgeKind.Implementation;
    //    }

    //    default: {
    //      return undefined;
    //    }
    //  }
    //},
    // This is a hack, and needs to be refactored!
    //nodeType(): string[] {
    //  switch (this.schematicKind) {
    //    case SchematicKind.Deployment: {
    //      return ["service"];
    //    }

    //    case SchematicKind.Component: {
    //      return ["service", "torture", "dockerImage"];
    //    }

    //    default: {
    //      return ["service", "torture", "dockerImage"];
    //    }
    //  }
    //},
    schematicOrientation(): SchematicOrientation | null {
      switch (this.schematicKind) {
        case SchematicKind.Deployment: {
          return SchematicOrientation.Vertical;
        }

        case SchematicKind.Component: {
          return SchematicOrientation.Horizontal;
        }

        default: {
          return null;
        }
      }
    },
    async selectNode(node: ISchematicNode) {
      schematicSelectNode$.next(node);
      if (this.schematicKind == SchematicKind.Deployment) {
        deploymentSchematicSelectNode$.next(node);
      }
    },
    async clearNodeSelection() {
      // @ts-ignore
      if (this.selectedNode) {
        schematicSelectNode$.next(null);
        if (this.schematicKind == SchematicKind.Deployment) {
          deploymentSchematicSelectNode$.next(null);
        }
      }
    },
    onNodeCreate(nodeId: string, _event: MouseEvent): void {
      this.viewer.isNodeCreate = true;
      const id = this.id + "." + nodeId;
      this.selection.element = id; // document.getElementById(id);

      if (this.canvas.element) {
        //const mouseLocalSpace = SiCg.cgGetMousePositionInElementSpace(
        //  event as MouseEvent,
        //  this.canvas.element,
        //);
        // TODO: This guard might be unneeded or a bug!
        if (this.selection.element) {
          //const canvasBoundingRect = this.canvas.element.getBoundingClientRect();

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
      source: ConnectionNodeReference,
      destination: ConnectionNodeReference,
    ): EdgeTemporary {
      return {
        id: edgeId,
        headVertex: {
          nodeId: destination.nodeId,
          socket: destination.socketName,
        },
        tailVertex: {
          nodeId: source.nodeId,
          socket: source.socketName,
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
    redraw(_event: IPanelLayoutUpdated | UIEvent) {
      // ALEX DISABLED
      // this.$forceUpdate();
    },

    backspaceEvent(event: ShortcutUpdateEvent): void {
      if (event.panelId === this.id) {
        if (event.action === ShortcutActions.DeleteNode) {
          this.deleteActiveNode();
        }
      }
    },
    async deleteActiveNode(this: any) {
      if (
        this.editMode &&
        this.selectedNode &&
        this.currentApplicationId &&
        this.currentWorkspace &&
        this.currentChangeSet &&
        this.currentEditSession &&
        this.currentSystem
      ) {
        const nodeId = this.selectedNode.node.id;
        const reply = await SchematicDal.nodeDelete({
          nodeId,
          applicationId: this.currentApplicationId,
          workspaceId: this.currentWorkspace.id,
          changeSetId: this.currentChangeSet.id,
          editSessionId: this.currentEditSession.id,
          systemId: this.currentSystem.id,
        });
        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
          return;
        }
        nodeDeleted$.next({ nodeId });
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
    spacebarDown(_e: Event): void {
      this.viewer.spacebarIsDown = true;
    },
    spacebarUp(_e: Event): void {
      this.viewer.spacebarIsDown = false;
    },
    mouseEnter() {
      this.activateShortcuts();
      this.activateViewer();
    },
    mouseLeave() {
      this.deactivateShortcuts();
      this.deactivateViewer();

      if (this.viewport.event.mouse != null) {
        this.mouseUp(this.viewport.event.mouse);
      }
    },
    mouseDown(this: any, e: Event): void {
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
        const _event: MouseRegistrationEvent = {
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
        if (this.viewer.panningMode) {
          this.viewer.isPanning = true;
        }

        // ----------------------------------------------------------------
        // Deselect active node
        // ----------------------------------------------------------------
        if (
          e.target &&
          e.target instanceof SVGElement &&
          e.target.id &&
          !this.viewer.panningMode
        ) {
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
                this.selection.element = id; //document.getElementById(id);
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
            console.log(e.target);

            let graphViewerId = this.id;
            let entityType = e.target.getAttribute("entityType");
            let entityId = e.target.getAttribute("entityId");
            let schematicKind = e.target.getAttribute("schematicKind");
            if (entityType && schematicKind && entityId) {
              edgeCreating$.next({
                graphViewerId,
                entityType,
                schematicKind,
                entityId,
              });
            }

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
            this.selection.element = this.connection.transientConnection.id;

            // document.getElementById(
            //  this.connection.transientConnection.id,
            //);
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
            let selectionElement = document.getElementById(
              this.selection.element,
            ) as HTMLElement;
            this.selection.offset.x =
              mouseLocalSpace.x - selectionElement.offsetLeft;
            this.selection.offset.y =
              mouseLocalSpace.y - selectionElement.offsetTop;
          }
        }
      }
    },
    mouseMove(this: any, e: MouseEvent): void {
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
          let selectionElement = document.getElementById(
            this.selection.element,
          ) as HTMLElement;

          // ----------------------------------------------------------------
          // Mouse is connecting nodes
          // ----------------------------------------------------------------
          this.viewport.event.mouse = e;
          const newPosition = SiCg.cgGetMousePositionInElementSpace(
            e as MouseEvent,
            this.canvas.element as HTMLElement,
          );
          selectionElement.setAttribute("x2", String(newPosition.x));
          selectionElement.setAttribute("y2", String(newPosition.y));

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
            this.viewport.event.mouse = e;
            this.selection.id = this.selectedNode.node.id;
            const id = this.id + "." + this.selection.id;
            this.selection.element = id; //document.getElementById(id);
            this.viewer.draggingMode = true;
          }

          // ----------------------------------------------------------------
          // Mouse is dragging a node
          // ----------------------------------------------------------------
          if (this.selection.element !== null) {
            if (this.viewer.mouseIsDown || this.viewer.isNodeCreate) {
              this.viewport.event.mouse = e;
              let selectionElement = document.getElementById(
                this.selection.element,
              ) as HTMLElement;

              if (
                selectionElement &&
                selectionElement.classList.contains("node") &&
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
            positionCtx: this.positionCtx,
          };
          PanelEventBus.$emit("panel-viewport-node-update", nodePositionUpdate);

          const edgePositionUpdate: EdgePostionUpdateEvent = {
            nodeId: this.selection.id as string,
            nodePosition: this.selection.position,
            positionCtx: this.positionCtx,
          };

          PanelEventBus.$emit("panel-viewport-edge-update", edgePositionUpdate);
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
        const _event: MouseRegistrationEvent = {
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

      // ----------------------------------------------------------------
      // Cleanup
      // ----------------------------------------------------------------
      this.viewer.mouseIsDown = false;
      this.viewer.isPanning = false;
      this.viewer.nodeDeselection = false;
      this.removeTransientEdge();
      this.removeTemporaryEdge();
      this.viewport.event.mouse = null;
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
        //const viewportRect = this.viewport.element.getBoundingClientRect();

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
        //const viewportRect = this.viewport.element.getBoundingClientRect();

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
        this.setNodeLocalPosition(setNodePositionPayload);

        // Update node edges position
        const edgePositionUpdate: EdgePostionUpdateEvent = {
          nodeId: this.selection.id as string,
          nodePosition: position,
          positionCtx: this.positionCtx,
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
    setNodeLocalPosition(payload: SetNodePositionPayload) {
      if (this.schematic) {
        if (payload.position.x == 0 && payload.position.y == 0) {
          return;
        }
        const position = {
          x: String(payload.position.x),
          y: String(payload.position.y),
        };

        if (
          this.schematic &&
          this.schematic.nodes[payload.nodeId] &&
          this.schematic.nodes[payload.nodeId].node &&
          this.schematic.nodes[payload.nodeId].node.positions[payload.context]
        ) {
          this.schematic.nodes[payload.nodeId].node.positions[
            payload.context
          ] = position;
        } else {
          if (
            this.schematic &&
            this.schematic.nodes[payload.nodeId] &&
            this.schematic.nodes[payload.nodeId].node
          ) {
            this.schematic.nodes[payload.nodeId].node.positions = {
              [payload.context]: position,
            };
          }
        }
      }
    },
    async setNodePosition(this: any, position: Cg2dCoordinate) {
      if (
        this.selectedNode &&
        this.selectedNode.node &&
        this.currentApplicationId &&
        this.currentWorkspace &&
        this.currentSystem
      ) {
        if (this.selectedNode.node.id) {
          if (position.x == 0 && position.y == 0) {
            return;
          }
          const request: INodeUpdatePositionRequest = {
            nodeId: this.selectedNode.node.id,
            contextId: this.positionCtx,
            x: `${position.x}`,
            y: `${position.y}`,
            workspaceId: this.currentWorkspace.id,
          };

          let reply: INodeUpdatePositionReply = await SchematicDal.nodeUpdatePosition(
            request,
          );

          if (reply.error) {
            emitEditorErrorMessage(reply.error.message);
          } else {
            nodePositionUpdated$.next({ positionCtx: this.positionCtx });
          }

          if (this.schematicKind == SchematicKind.Deployment) {
            let componentPositionCtx = `${this.selectedNode.object.id}.component`;
            if (!this.selectedNode.node.positions[componentPositionCtx]) {
              if (this.canvas.element) {
                // const canvasBoundingRect = this.canvas.element.getBoundingClientRect();
                // const x = canvasBoundingRect.x * 0.5;
                // const y = canvasBoundingRect.y * 0.5;
                const request: INodeUpdatePositionRequest = {
                  nodeId: this.selectedNode.node.id,
                  contextId: componentPositionCtx,
                  x: `${position.x}`,
                  y: `${position.y}`,
                  workspaceId: this.currentWorkspace.id,
                };

                let reply: INodeUpdatePositionReply = await SchematicDal.nodeUpdatePosition(
                  request,
                );

                if (reply.error) {
                  emitEditorErrorMessage(reply.error.message);
                } else {
                  nodePositionUpdated$.next({ positionCtx: this.positionCtx });
                }
              }
            }
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
          this.schematic &&
          this.schematic.nodes[sourceNode[1]] &&
          this.schematic.nodes[destinationNode[1]]
        ) {
          const connectionKind = this.connectionKind();
          let sourceEntity = SiEntity.fromJson(
            this.schematic.nodes[sourceNode[1]].object as Entity,
          );
          let destinationEntity = SiEntity.fromJson(
            this.schematic.nodes[destinationNode[1]].object as Entity,
          );
          let destinationSocketName = destinationNode[2].split(":")[1];
          if (connectionKind != undefined) {
            let validInput = this.validInput(
              sourceEntity,
              destinationEntity,
              connectionKind,
            );
            if (validInput) {
              destinationSocketName = validInput.name;
            }
          }

          // connect nodes
          const source: ConnectionNodeReference = {
            nodeId: sourceNode[1],
            socketId: sourceNode[2],
            socketName: sourceNode[2].split(":")[1],
            nodeKind: this.schematic.nodes[sourceNode[1]].node
              .objectType as string,
          };

          const destination: ConnectionNodeReference = {
            nodeId: destinationNode[1],
            socketId: destinationNode[2],
            socketName: destinationSocketName,
            nodeKind: this.schematic.nodes[destinationNode[1]].node
              .objectType as string,
          };

          if (
            connectionKind != undefined &&
            !this.edgeExistsOnGraph(source, destination, connectionKind) &&
            this.validEdge(sourceEntity, destinationEntity, connectionKind)
          ) {
            this.connection.transientConnection.edge = this.newTemporaryEdge(
              "temporaryEdge",
              source,
              destination,
            );
            this.removeTransientEdge();

            await this.createConnection(source, destination);
            this.removeTemporaryEdge();
          }
          this.removeTransientEdge();

          // update node edges position
          const edgePositionUpdate: EdgePostionUpdateEvent = {
            sourceNodeId: sourceNode[1],
            destinationNodeId: destinationNode[1],
            positionCtx: this.positionCtx,
          };
          PanelEventBus.$emit("panel-viewport-edge-update", edgePositionUpdate);
        }
      }
      edgeCreating$.next(null);
    },
    validInput(
      sourceEntity: Entity,
      destinationEntity: Entity,
      connectionKind: string,
    ): Input | undefined {
      let destinationEntitySchema = destinationEntity.schema();
      let sourceEntitySchema = sourceEntity.schema();
      let validInput = _.find(destinationEntitySchema.inputs, (input) => {
        return (
          input.edgeKind == connectionKind &&
          (_.includes(input.types, sourceEntity.entityType) ||
            input.types == "dependencies" ||
            (input.types == "implementations" &&
              _.includes(
                sourceEntitySchema.implements,
                destinationEntity.entityType,
              )))
        );
      });
      return validInput;
    },
    validEdge(
      sourceEntity: Entity,
      destinationEntity: Entity,
      connectionKind: string,
    ): boolean {
      let validInput = this.validInput(
        sourceEntity,
        destinationEntity,
        connectionKind,
      );
      if (validInput) {
        return true;
      } else {
        return false;
      }
    },
    async createConnection(
      this: any,
      source: ConnectionNodeReference,
      destination: ConnectionNodeReference,
    ) {
      if (
        this.currentSystem &&
        this.currentApplicationId &&
        this.currentWorkspace &&
        this.currentChangeSet &&
        this.currentEditSession &&
        this.rootObjectId
      ) {
        const connection: Connection = {
          source: source,
          destination: destination,
        };

        let reply: ConnectionCreateReply = await SchematicDal.connectionCreate({
          connection: connection,
          workspaceId: this.currentWorkspace.id,
          changeSetId: this.currentChangeSet.id,
          editSessionId: this.currentEditSession.id,
          rootObjectId: this.rootObjectId,
          schematicKind: this.schematicKind,
        });

        if (reply.error) {
          emitEditorErrorMessage(reply.error.message);
        } else {
          if (reply.schematic) {
            schematicUpdated$.next({
              schematicKind: this.schematicKind,
              schematic: reply.schematic,
              rootObjectId: this.rootObjectId,
            });
          }
        }
      }
    },
    connectionKind(): EdgeKind | undefined {
      switch (this.schematicKind) {
        case SchematicKind.Deployment: {
          return EdgeKind.Deployment;
        }

        case SchematicKind.Component: {
          return EdgeKind.Configures;
        }

        default: {
          return undefined;
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

      return Object.values(this.schematic.edges).some(function (edge) {
        return (
          edge.headVertex.nodeId == destination.nodeId &&
          edge.headVertex.socket == destination.socketId &&
          edge.tailVertex.nodeId == source.nodeId &&
          edge.tailVertex.socket == source.socketId &&
          edge.kind == kind
        );
      });
    },
  },
});
</script>
