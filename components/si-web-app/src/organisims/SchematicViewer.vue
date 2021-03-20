<template>
  <div
    ref="viewport"
    class="w-full h-full"
    @mouseenter="mouseEnter()"
    @mouseleave="mouseLeave()"
    @mousedown="mouseDown($event)"
    @mousemove="mouseMove($event)"
    @mouseup="mouseUp($event)"
  >
    <div :id="canvas.id" ref="canvas" class="absolute block">
      <svg
        :id="connection.transientConnection.elementId"
        height="100%"
        width="100%"
        class="absolute"
      />

      <SiGraphEdge
        v-if="connection.transientConnection.edge"
        :key="connection.transientConnection.edge.id"
        :edge="connection.transientConnection.edge"
        :graphViewerId="id"
        :schematicPanelStoreCtx="schematicPanelStoreCtx"
      />

      <div v-if="graph">
        <SiGraphEdge
          v-for="(edge, index) in graph.edges"
          :key="index"
          :edge="edge"
          :graphViewerId="id"
          :schematicPanelStoreCtx="schematicPanelStoreCtx"
        />
        <SiGraphNode
          v-for="(node, index) in graph.nodes"
          :key="index"
          :node="node"
          :graphViewerId="id"
          :storesCtx="storesCtx"
          @selectNode="selectNode"
        />
      </div>

      <SiBackground :resolution="canvas.resolution" style="default" />
    </div>
  </div>
</template>

<script lang="ts">
/**
 * Graph Viewer
 *
 * - Input: graph data (nodes and edges)
 * - Input: viewer configuration (viewer options)
 *
 * - Viewer options:
 * -- Canvas resolution
 * -- Graph rendering style (nodes and edges)
 * -- Background rendering style
 * -- Keyboard shortcuts
 * -- Additional viewer options
 *
 * - Renders graph (nodes and edges)
 * - Renders background (grid)
 *
 * - Handles graph manipulations (nodes and edges: transforms and connections)
 * - Handles viewport manipulations (pan and zoom)
 * - Handles context menus (node right click & background right click menus)
 **/

import Vue, { PropType } from "vue";
import { mapState } from "vuex";

import { PanelEventBus } from "@/atoms/PanelEventBus";
import { SchematicPanelStore } from "@/store/modules/schematicPanel";
import { SessionStore } from "@/store/modules/session";

import { Edge, IEdge, IVertex, EdgeKind } from "@/api/sdf/model/edge";

import { NodePositionUpdateEvent } from "@/organisims/SchematicViewer/Node.vue";
import {
  EdgePostionUpdateEvent,
  EdgeTemporary,
} from "@/organisims/SchematicViewer/Edge.vue";

import { SetNodePositionPayload } from "@/store/modules/schematicPanel";
import { ConnectionCreateReply } from "@/api/sdf/dal/schematicDal";
import {
  ISchematic,
  Schematic,
  ISchematicNode,
} from "@/api/sdf/model/schematic";

import { InstanceStoreContext } from "@/store";

import { SiCg } from "@/api/sicg";
import { Cg2dCoordinate, CgResolution } from "@/api/sicg";
import _ from "lodash";

import { INodeUpdatePositionReply } from "@/api/sdf/dal/editorDal";
import {
  Connection,
  ConnectionNodeReference,
  ConnectionKind,
} from "@/api/sdf/dal/schematicDal";

import {
  NodeUpdatePositionePayload,
  ConnectionCreatePayload,
  TransientEdgeRemovalEvent,
} from "@/store/modules/editor";

import SiBackground from "@/atoms/SiBackground.vue";
import SiGraphNode from "./SchematicViewer/Node.vue";
import SiGraphEdge from "./SchematicViewer/Edge.vue";

import {
  ShortcutRegistrationEvent,
  ShortcutContext,
  MouseRegistrationEvent,
  MouseContext,
  ShortcutActions,
  ShortcutUpdateEvent,
} from "@/organisims/ShortcutsEventBroker.vue";
import { INode } from "@/api/sdf/model/node";
export type StoreCtx = InstanceStoreContext<SchematicPanelStore>;

export interface StoresCtx {
  [storeId: string]: StoreCtx;
}

export interface Graph {
  nodes: ISchematic["nodes"];
  edges: ISchematic["edges"];
}

interface Selection {
  element: HTMLElement | null;
  offset: {
    x: number;
    y: number;
  };
  position: Cg2dCoordinate;
  id: string | null;
}

interface IData {
  id: string;
  canvas: {
    element: Element | Vue | HTMLElement | null;
    resolution: CgResolution;
    id: string;
  };
  viewer: {
    shortcutsEnabled: boolean;
    panningMode: boolean;
    isPanning: boolean;
    draggingMode: boolean;
    isNodeConnectionMode: boolean;
    isDragging: boolean;
    isActive: boolean;
    isNodeCreate: boolean;
    mouseIsDown: boolean;
  };
  viewport: {
    element: Element | Vue | HTMLElement | null;
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
        x: number | null;
        y: number | null;
      };
    };
    mouse: {
      position: {
        x: number;
        y: number;
      };
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

type IPanelLayoutUpdated = boolean;

export default Vue.extend({
  name: "SchematicViewer",
  components: {
    SiGraphEdge,
    SiGraphNode,
    SiBackground,
  },
  props: {
    graph: {
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
    let id = _.uniqueId("graphViewer:");
    return {
      id: id,
      canvas: {
        element: null,
        resolution: {
          x: 600,
          y: 600,
        },
        id: id + "." + "canvas",
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
        mouse: {
          position: {
            x: 0,
            y: 0,
          },
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
          id: "transientConnection",
          sourceSocketId: "",
          destinationSocketId: "",
          elementId: id + "." + "transientConnectionSvg",
          edge: null,
        },
      },
    };
  },
  mounted(): void {
    this.registerEvents();
    this.setCanvasSize();
    this.setCanvasPositionToViewportCenter();

    //@ts-ignore
    this.canvas.element = this.$refs.canvas;
    //@ts-ignore
    this.viewport.element = this.$refs.viewport;
  },
  beforeDestroy() {
    this.deRegisterEvents();
  },
  beforeUpdate: function() {
    this.setCanvasPositionToViewportCenter();
  },
  computed: {
    selectedNode(): ISchematicNode | null {
      return this.storesCtx.schematicPanelStoreCtx.state.selectedNode;
    },
    ...mapState({
      currentSystem: (state: any): SessionStore["currentSystem"] =>
        state.session.currentSystem,
      editMode(): boolean {
        return this.$store.getters["editor/inEditable"];
      },
    }),
  },
  methods: {
    registerEvents(): void {
      PanelEventBus.$on("panel-viewport-update", this.redraw);
      PanelEventBus.$on("panel-viewport-edge-remove", this.removeTemporaryEdge);
      PanelEventBus.$on(
        "shortcuts-update-" + this.id,
        this.handleShortcutUpdate,
      );
    },
    deRegisterEvents(): void {
      PanelEventBus.$off("panel-viewport-update", this.redraw);
      PanelEventBus.$off(
        "panel-viewport-edge-remove",
        this.removeTransientEdge,
      );
      PanelEventBus.$off(
        "shortcuts-update-" + this.id,
        this.handleShortcutUpdate,
      );
    },
    async selectNode(node: INode) {
      await this.schematicPanelStoreCtx.dispatch("nodeSelect", node);
    },
    async clearNodeSelection() {
      await this.schematicPanelStoreCtx.dispatch("nodeSelectionClear");
    },
    activateShortcuts(): void {
      this.viewer.shortcutsEnabled = true;

      let ctx: ShortcutContext = {
        id: this.id,
        isActive: true,
      };
      let event: ShortcutRegistrationEvent = {
        context: ctx,
      };

      PanelEventBus.$emit("shortcuts-registration-update", event);
    },
    deactivateShortcuts(): void {
      this.viewer.shortcutsEnabled = false;

      let ctx: ShortcutContext = {
        id: this.id,
        isActive: false,
      };
      let event: ShortcutRegistrationEvent = {
        context: ctx,
      };

      PanelEventBus.$emit("shortcuts-registration-update", event);
    },
    handleShortcutUpdate(e: ShortcutUpdateEvent) {
      if (e.action == ShortcutActions.StartPan) {
        this.activatePanningMode();
      } else if (e.action == ShortcutActions.EndPan) {
        this.deactivatePanningMode();
      }
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
    },
    deactivatePanningMode(): void {
      this.viewer.panningMode = false;
    },
    setIsNodeCreate(): void {
      this.viewer.isNodeCreate = true;
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
        // Deletect node if clicking on "canvas"
        // @ts-ignore
        if (e.target.id == "transientConnectionSvg") {
          this.clearNodeSelection();
        }

        let ctx: MouseContext = {
          id: this.id,
          isActive: true,
        };
        let event: MouseRegistrationEvent = {
          context: ctx,
        };
        PanelEventBus.$emit("mouse-registration-update", event);

        // This block should go somewhere else...
        let canvas = this.canvas.element as HTMLElement;
        let canvasRect = canvas.getBoundingClientRect();
        this.viewport.pan.originalPosition = {
          x: canvasRect.left,
          y: canvasRect.top,
        };

        // This block should go somewhere else...
        let mouseEvent = e as MouseEvent;
        this.viewport.mouse.position = {
          x: mouseEvent.clientX,
          y: mouseEvent.clientY,
        };
        // console.log(
        //   "mouseClicked with position (x,y): ",
        //   this.viewport.mouse.position.x,
        //   this.viewport.mouse.position.y,
        // );

        // Pan viewer
        if (this.viewer.panningMode) {
          this.viewer.isPanning = true;
        } else {
          this.viewer.draggingMode = true;

          // Select element for current selection

          // Select a node
          if (e.target && e.target instanceof HTMLDivElement) {
            if (e.target.classList.contains("node")) {
              if (this.selectedNode?.node.id) {
                this.selection.id = this.selectedNode.node.id;
                let id = this.id + "." + this.selectedNode.node.id;
                this.selection.element = document.getElementById(id);
              } else {
                this.selection.element = null;
              }
            } else {
              this.selection.element = null;
            }
          }

          // Draw an edge
          if (e.target) {
            //  @ts-ignore
            if (e.target.classList.contains("socket")) {
              /**
               * - Socket Connection -
               *
               * V1 Connection
               * create a new transient edge (p1,p2) and snap p2(xy) to cursor until mouse up.
               * - user clicks on source socket
               * - user drags connection edge to destination socket (on mouse move)
               * - user connects connection edge by draging p2 over the destination socket. On mouse up if p2 is over the
               *   destination socket, a new connection is made.
               *
               * V2 Disconnection
               * - to disconnect a conneciton, user clicks on a socket and drags the connection edge away from any node
               *   and mouse up while the connection edge p2 isn't over anohter socket.
               *
               */

              // @ts-ignore
              this.connection.transientConnection.sourceSocketId = e.target.id;

              let selectionObjectOffset = SiCg.cgGetMousePositionInElementSpace(
                e as MouseEvent,
                this.canvas.element as HTMLElement,
              );

              // Ccreate an edge
              let transientConnection = document.createElementNS(
                "http://www.w3.org/2000/svg",
                "line",
              );

              this.connection.transientConnection.id =
                this.connection.transientConnection.sourceSocketId +
                "." +
                "transient";
              transientConnection.id = this.connection.transientConnection.id;

              transientConnection.setAttribute(
                "x1",
                String(selectionObjectOffset.x),
              );
              transientConnection.setAttribute(
                "y1",
                String(selectionObjectOffset.y),
              );
              transientConnection.setAttribute(
                "x2",
                String(selectionObjectOffset.x),
              );
              transientConnection.setAttribute(
                "y2",
                String(selectionObjectOffset.y),
              );
              transientConnection.setAttribute("stroke", "rgb(71,99,113)");
              transientConnection.setAttribute("stroke-width", "2");

              // Add the edge to the svg
              let transientConnectionSvg = document.getElementById(
                this.connection.transientConnection.elementId,
              );

              transientConnectionSvg?.appendChild(transientConnection);

              // Sets state to update the edge when the mouse moves
              this.viewer.isNodeConnectionMode = true;
              this.selection.element = document.getElementById(
                this.connection.transientConnection.id,
              );
            }
          }

          // Set selection
          if (e instanceof MouseEvent && this.selection.element) {
            let selectionOffsetLeft = this.selection.element.offsetLeft;
            let selectionOffsetTop = this.selection.element.offsetTop;

            let mousePositionX = e.clientX;
            let mousePositionY = e.clientY;

            this.selection.offset.x = mousePositionX - selectionOffsetLeft;
            this.selection.offset.y = mousePositionY - selectionOffsetTop;
          }
        }
      }
    },
    mouseMove(e: MouseEvent): void {
      if (this.viewer.isActive) {
        if (this.viewer.panningMode) {
          this.viewer.isPanning = true;
          this.panViewport(e);
        } else if (
          this.viewer.isNodeConnectionMode &&
          this.selection.element &&
          this.editMode
        ) {
          let newPosition = SiCg.cgGetMousePositionInElementSpace(
            e as MouseEvent,
            this.canvas.element as HTMLElement,
          );

          this.selection.element.setAttribute("x2", String(newPosition.x));
          this.selection.element.setAttribute("y2", String(newPosition.y));

          //@ts-ignore
          if (e.target.classList.contains("socket")) {
            this.connection.transientConnection.destinationSocketId =
              //@ts-ignore
              e.target.id;
          }
        } else {
          if (this.viewer.isNodeCreate && this.selectedNode) {
            if (this.selectedNode.node) {
              this.selection.id = this.selectedNode.node.id;
              let id = this.id + "." + this.selection.id;
              this.selection.element = document.getElementById(id);
              this.viewer.draggingMode = true;
            }
          }

          // Refactor node drag using viewer mode and flags...
          if (this.selection.element !== null) {
            if (this.viewer.mouseIsDown || this.viewer.isNodeCreate) {
              if (
                this.selection.element.classList.contains("node") &&
                this.editMode
              ) {
                this.viewer.isDragging = true;

                let mousePositionX = e.clientX;
                let mousePositionY = e.clientY;

                // Need to account for zoom factor? (1 / this.zoom.factor)
                let newPositionX = mousePositionX - this.selection.offset.x;
                let newPositionY = mousePositionY - this.selection.offset.y;

                let newPosition: Cg2dCoordinate = {
                  x: newPositionX,
                  y: newPositionY,
                };

                let setNodePositionPayload: SetNodePositionPayload = {
                  nodeId: this.selection.id as string,
                  context: "AAA",
                  position: newPosition,
                };
                this.storesCtx["schematicPanelStoreCtx"].dispatch(
                  "setNodePosition",
                  setNodePositionPayload,
                );

                let position: Cg2dCoordinate = {
                  x: newPositionX,
                  y: newPositionY,
                };

                let edgePositionUpdate: EdgePostionUpdateEvent = {
                  nodeId: this.selection.id as string,
                  nodePosition: position,
                };

                let eventId =
                  "panel-viewport-edge-update" +
                  "." +
                  this.id +
                  "." +
                  this.selection.id;

                PanelEventBus.$emit(eventId, edgePositionUpdate);

                this.selection.position = position;
              }
            }
          }
        }
      }
    },
    mouseUp(e: MouseEvent): void {
      if (this.viewer.isActive) {
        if (this.viewer.isDragging == true) {
          this.viewer.isDragging = false;
          this.viewer.draggingMode = false;
          this.viewer.isNodeCreate = false;

          this.setNodePosition(this.selection.position);

          if (this.selection.id) {
            let nodePositionUpdate: NodePositionUpdateEvent = {
              position: this.selection.position,
              nodeId: this.selection.id,
            };
            PanelEventBus.$emit(
              "panel-viewport-node-update",
              nodePositionUpdate,
            );

            let edgePositionUpdate: EdgePostionUpdateEvent = {
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

        // TODO: DESELECT SELECTED NODE

        if (this.viewer.isPanning == true) {
          this.viewer.isPanning = false;

          let ctx: MouseContext = {
            id: this.id,
            isActive: false,
          };
          let event: MouseRegistrationEvent = {
            context: ctx,
          };
          PanelEventBus.$emit("mouse-registration-update", event);
        }

        if (this.viewer.isNodeConnectionMode) {
          this.viewer.isNodeConnectionMode = false;
          this.selection.element = null;
          this.selection.id = null;
          let transientConnection = document.getElementById(
            this.connection.transientConnection.id,
          );

          if (
            this.connection.transientConnection.destinationSocketId !=
            this.connection.transientConnection.sourceSocketId
          ) {
            // Source socket (we draw the connection from this socket to the destination socket).
            let sourceElementString = this.connection.transientConnection.sourceSocketId.split(
              ".",
            );

            let sourceNode = this.connection.transientConnection.sourceSocketId.split(
              ".",
            );
            let source: ConnectionNodeReference = {
              nodeId: sourceNode[1],
              socketId: sourceNode[2],
              nodeKind: this.storesCtx.schematicPanelStoreCtx.state.schematic
                ?.nodes[sourceNode[1]].node.objectType as string,
            };

            let destinationNode = this.connection.transientConnection.destinationSocketId.split(
              ".",
            );
            let destination: ConnectionNodeReference = {
              nodeId: destinationNode[1],
              socketId: destinationNode[2],
              nodeKind: this.storesCtx.schematicPanelStoreCtx.state.schematic
                ?.nodes[destinationNode[1]].node.objectType as string,
            };

            // console.log(
            //   "sourceSocketId:",
            //   this.connection.transientConnection.sourceSocketId,
            // );
            // console.log(
            //   "destinationSocketId:",
            //   this.connection.transientConnection.destinationSocketId,
            // );

            if (!this.edgeExistsOnGraph(source, destination, "configures")) {
              this.connection.transientConnection.edge = this.newTemporaryEdge(
                "aa",
                source.nodeId,
                destination.nodeId,
              );
              this.removeTransientEdge();
              this.createConnection(source, destination, "configures");
            }

            this.removeTransientEdge();

            let edgePositionUpdate: EdgePostionUpdateEvent = {
              sourceNodeId: sourceNode[1],
              destinationNodeId: destinationNode[1],
            };
            PanelEventBus.$emit(
              "panel-viewport-edge-update",
              edgePositionUpdate,
            );
          }
        }
      }
      this.viewer.mouseIsDown = false;
      this.viewer.panningMode = false;
      this.viewer.isPanning = false;
    },
    panViewport(e: MouseEvent) {
      let mousePosition = {
        x: e.clientX,
        y: e.clientY,
      };

      if (
        this.viewport.pan.position.x != null &&
        this.viewport.pan.position.y != null &&
        this.viewport.pan.originalPosition.x != null &&
        this.viewport.pan.originalPosition.y != null
      ) {
        let mouseMovement = {
          x: mousePosition.x - this.viewport.mouse.position.x,
          y: mousePosition.y - this.viewport.mouse.position.y,
        };

        this.viewport.mouse.position = {
          x: mousePosition.x,
          y: mousePosition.y,
        };

        this.viewport.pan.translation = {
          x: this.viewport.pan.position.x + mouseMovement.x,
          y: this.viewport.pan.position.y + mouseMovement.y,
        };

        this.viewport.pan.position = {
          x: this.viewport.pan.translation.x,
          y: this.viewport.pan.translation.y,
        };

        let position: Cg2dCoordinate = {
          x: this.viewport.pan.translation.x,
          y: this.viewport.pan.translation.y,
        };

        let canvas = this.canvas.element as HTMLElement;
        let canvasRect = canvas.getBoundingClientRect();

        // console.log(
        //   this.viewport.pan.originalPosition.x,
        //   canvasRect.left,
        //   this.viewport.pan.position.x,
        //   mouseMovement.x,
        //   position.x,
        // );

        // console.log("canvas original position (x,y): ", this.viewport.pan.originalPosition.x, this.viewport.pan.originalPosition.y)
        // console.log("mouse movement (x,y): ", mouseMovement.x, mouseMovement.y)
        // console.log("translation (x,y): ", this.viewport.pan.translation.x, this.viewport.pan.translation.y)
        canvas.setAttribute(
          "style",
          "left:" +
            String(position.x) +
            "px;" +
            "top:" +
            String(position.y) +
            "px;",
        );

        this.viewport.pan.position.x = position.x;
        this.viewport.pan.position.y = position.y;
      }
    },
    setCanvasSize() {
      let canvas = this.$refs.canvas as HTMLElement;
      SiCg.cgSetElementSize(
        canvas,
        this.canvas.resolution.x,
        this.canvas.resolution.y,
      );
    },
    setCanvasPositionToViewportCenter() {
      let canvas = this.$refs.canvas as HTMLElement;
      let viewport = this.$refs.viewport as HTMLElement;
      // SiCg.cgSetElementPositionToViewportCenter(canvas, viewport);
    },
    redraw(event: IPanelLayoutUpdated | UIEvent) {
      this.$forceUpdate();
    },
    newTemporaryEdge(
      edgeId: string,
      sourceNodeId: string,
      destinationNodeId: string,
    ): EdgeTemporary {
      // console.log("newTemporaryEdge:sourceNodeId: ", sourceNodeId);
      // console.log("newTemporaryEdge:destinationNodeId: ", destinationNodeId);

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
        let transientConnection = document.getElementById(
          this.connection.transientConnection.id,
        );
        if (transientConnection != null) {
          transientConnection.remove();
        }
      }
    },
    async setNodePosition(position: Cg2dCoordinate) {
      if (this.selectedNode && this.selectedNode.node) {
        if (this.selectedNode.node.id) {
          let payload: NodeUpdatePositionePayload = {
            nodeId: this.selectedNode?.node.id,
            contextId: "AAA",
            position: position,
          };

          let reply: INodeUpdatePositionReply = await this.$store.dispatch(
            "editor/nodeSetPosition",
            payload,
          );
          if (reply.error) {
            PanelEventBus.$emit("editor-error-message", reply.error.message);
          }
        }
      }
    },
    async createConnection(
      source: ConnectionNodeReference,
      destination: ConnectionNodeReference,
      kind: String,
    ) {
      if (this.currentSystem) {
        let connection: Connection = {
          kind: kind,
          source: source,
          destination: destination,
          systemId: this.currentSystem.id,
        };

        let payload: ConnectionCreatePayload = {
          connection: connection,
        };

        // console.log("createConnection:source: ", source.nodeId);
        // console.log("createConnection:destination: ", destination.nodeId);

        let reply: ConnectionCreateReply = await this.$store.dispatch(
          "editor/connectionCreate",
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
      if (!this.graph) {
        throw new Error("graph must be set for edgeExists check, bug!");
      }

      return Object.values(this.graph.edges).some(function(edge) {
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
