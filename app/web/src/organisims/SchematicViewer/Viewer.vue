<template>
  <div :id="container.id" ref="container" class="w-full h-full">
    <div v-if="debug" class="flex flex-row">
      <div class="ml-2 font-medium text-yellow-200">{{ state.value }}</div>

      <div v-if="selection" class="ml-2 font-medium text-blue-200">
        <div v-for="(s, i) in selection" :key="i">
          {{ s.name }}
        </div>
      </div>
    </div>
    <canvas
      :id="canvas.id"
      ref="canvas"
      @wheel="handleMouseWheel"
      @mouseenter="mouseEnter()"
      @mouseleave="mouseLeave()"
    />
  </div>
</template>

<script lang="ts">
import { ref, Ref, defineComponent, PropType } from "vue";
import _ from "lodash";
import * as Rx from "rxjs";
import * as PIXI from "pixi.js";
import * as OBJ from "./Viewer/obj";
import * as ST from "./state";
import { untilUnmounted } from "vuse-rx";
import { Interpreter } from "xstate";

import { ViewerStateMachine, ViewerEventKind } from "./state";
import { useMachine } from "@xstate/vue";

import {
  EditorContext,
  SchematicKind,
  nodeKindFromSchematicKind,
} from "@/api/sdf/dal/schematic";
import { SceneManager } from "./Viewer/scene";
import { SelectionManager } from "./Viewer/interaction/selection";
import { InteractionManager } from "./Viewer/interaction";
import { Renderer } from "./Viewer/renderer";
import { SchematicDataManager } from "./data";
import {
  deploymentSelection$,
  componentSelection$,
  SelectedNode,
} from "./state";

import * as MODEL from "./model";

import * as VE from "./event";

import { schematicData$ } from "./data";

interface KeyboardKey {
  long: string;
  short: string;
}

const spacebarKey: KeyboardKey = {
  long: "Spacebar",
  short: " ",
};

interface Data {
  component: {
    id: string;
    isActive: boolean;
  };
  canvas: {
    id: string;
    element: HTMLCanvasElement | undefined;
    isPanning: boolean;
    mousePosition: { x: number; y: number } | null | undefined;
  };
  container: {
    id: string;
    element: HTMLCanvasElement | undefined;
  };
  renderer: Renderer | undefined;
  sceneManager: SceneManager | undefined;
  dataManager: SchematicDataManager | undefined;
  spaceBarPressed: boolean;
  resizeObserver: ResizeObserver;
  interactionManager: InteractionManager | undefined;
  debug: boolean;
}

const selectionObserver = (
  schematicKind: SchematicKind,
): Rx.ReplaySubject<SelectedNode[]> => {
  switch (schematicKind) {
    case SchematicKind.Deployment:
      return deploymentSelection$;
    case SchematicKind.Component:
      return componentSelection$;
  }
  throw Error(`invalid schematic kind ${schematicKind}`);
};

export default defineComponent({
  name: "Viewer",
  props: {
    schematicViewerId: {
      type: String,
      required: true,
    },
    viewerState: {
      type: Object as PropType<ViewerStateMachine>,
      required: true,
    },
    viewerEvent$: {
      type: Object as
        | PropType<Rx.ReplaySubject<VE.ViewerEvent | null>>
        | undefined,
      required: false,
      default: undefined,
    },
    schematicData: {
      type: Object as PropType<MODEL.Schematic | null>,
      required: false,
      default: undefined,
    },
    editorContext: {
      type: Object as PropType<EditorContext | null>,
      required: true,
    },
    schematicKind: {
      type: String as PropType<SchematicKind | null>,
      required: true,
    },
    isComponentPanelPinned: {
      type: Boolean,
      required: true,
    },
    componentPanelPin: {
      type: Number,
      required: false,
      default: undefined,
    },
  },
  setup(props) {
    const { state, send, service } = useMachine(props.viewerState.machine);

    const selection: Ref<OBJ.Node[] | null> = ref(null);
    return {
      state,
      send,
      service,
      selection,
    };
  },
  data(): Data {
    const id = _.uniqueId();
    const canvasId =
      this.schematicViewerId + ":" + this.$options.name + "-" + id;
    const containerId = this.schematicViewerId + "-" + "container";

    // Watch for element resize. Not completely smooth, will need to revisit.
    const resizeObserver = new ResizeObserver((entries) => {
      this.resizeCanvas(
        entries[0].contentRect.width,
        entries[0].contentRect.height,
      );
    });
    return {
      component: {
        id: id,
        isActive: false,
      },
      canvas: {
        id: canvasId,
        element: undefined,
        isPanning: false,
        mousePosition: undefined,
      },
      container: {
        id: containerId,
        element: undefined,
      },
      renderer: undefined,
      sceneManager: undefined,
      dataManager: undefined,
      spaceBarPressed: false,
      resizeObserver: resizeObserver,
      interactionManager: undefined,
      debug: true,
    };
  },
  watch: {
    async componentPanelPin(ctx) {
      if (this.dataManager && this.schematicData) {
        this.dataManager.selectedDeploymentNodeId = ctx;
        await this.loadSchematicData(this.schematicData);
      }
    },
    isComponentPanelPinned(ctx) {
      if (this.dataManager) {
        this.dataManager.isComponentPanelPinned = ctx;
      }
    },
    editorContext(ctx) {
      if (this.dataManager && this.sceneManager && this.editorContext) {
        this.dataManager.editorContext$.next(ctx);
      }
    },
    async schematicKind(ctx) {
      if (this.dataManager && this.schematicData && this.interactionManager) {
        this.dataManager.schematicKind$.next(ctx);
        await this.loadSchematicData(this.schematicData);

        // Fixes some inconsistencies where if you change panels while dragging the dragging would stick forever
        ST.deactivateDragging(
          this.interactionManager.stateService as Interpreter<unknown>,
        );

        // We re-sync the last selection as now we can retrieve the selected nodes of this panel from the sceneManager
        const observable = selectionObserver(ctx);
        const data = await Rx.firstValueFrom(observable);
        observable.next(data);

        const parentDeploymentNodeId =
          this.schematicKind !== SchematicKind.Deployment
            ? this.dataManager.selectedDeploymentNodeId
            : null;
        console.debug(
          `Schematic Kind changed: ${ctx} ${parentDeploymentNodeId}`,
        );
      }
    },
    async schematicData(schematic) {
      if (this.dataManager && this.schematicData && this.schematicKind) {
        this.dataManager.schematicData$.next(schematic);
      }
    },
  },
  mounted(): void {
    this.canvas.element = this.$refs.canvas as HTMLCanvasElement;
    this.container.element = this.$refs.container as HTMLCanvasElement;

    document.addEventListener("keydown", this.handleKeyDown);
    document.addEventListener("keyup", this.handleKeyUp);

    if (this.viewerEvent$) {
      this.viewerEvent$.pipe(untilUnmounted).subscribe({
        next: (v) => this.handleViewerEvent(v),
      });
    }

    // this.$once("hook:beforeDestroy", () => {
    //   this.container.element.removeEventListener("keydown", this.handleKeyDown);
    // });

    this.resizeObserver.observe(this.container.element);

    this.renderer = new Renderer({
      view: this.canvas.element,
      resolution: window.devicePixelRatio || 1,
      width: this.container.element.offsetWidth,
      height: this.container.element.offsetHeight,
      autoDensity: true,
      antialias: true,
      backgroundColor: 0x282828,
    });

    const dataManager = new SchematicDataManager();
    this.dataManager = dataManager;
    this.dataManager.isComponentPanelPinned = this.isComponentPanelPinned;
    this.dataManager.editorContext$.next(this.editorContext);
    this.dataManager.schematicKind$.next(this.schematicKind);

    // Initializes selectedDeploymentNodeId to the global state
    Rx.firstValueFrom(deploymentSelection$).then((selections) => {
      const ourSelection = selections?.find(
        (sel) => sel.parentDeploymentNodeId === null,
      );
      const ourNode = ourSelection?.nodes?.find((_) => true);
      if (this.dataManager) {
        this.dataManager.selectedDeploymentNodeId = ourNode?.id ?? null;
      }
    });

    dataManager.schematicData$.pipe(untilUnmounted).subscribe({
      next: async (d) => {
        if (d) await this.loadSchematicData(d);
      },
    });

    // Global events
    schematicData$.pipe(untilUnmounted).subscribe({
      next: (d) => this.dataManager?.schematicData$?.next(d),
    });

    this.sceneManager = new SceneManager(this.renderer as Renderer);

    const interactionManager = new InteractionManager(
      this.sceneManager as SceneManager,
      dataManager,
      this.service,
      this.renderer as Renderer,
    );
    this.interactionManager = interactionManager;
    this.sceneManager.subscribeToInteractionEvents(interactionManager);

    deploymentSelection$.pipe(untilUnmounted).subscribe({
      next: async (selections) => {
        // Edge case where the visibility changed, and every state was cleared
        // except for this.selection
        if (
          !selections.length &&
          !this.interactionManager?.selectionManager?.selection?.length
        ) {
          this.selection = null;
        }

        const ourSelection = selections.find(
          (sel) => sel.parentDeploymentNodeId === null,
        );
        const ourNode = ourSelection?.nodes?.find((_) => true);

        if (!this.isComponentPanelPinned && this.dataManager) {
          this.dataManager.selectedDeploymentNodeId = ourNode?.id ?? null;
        }

        this.syncSelection(
          selections,
          ourSelection?.parentDeploymentNodeId ?? null,
        );
        switch (this.schematicKind) {
          case SchematicKind.Deployment:
            break;
          case SchematicKind.Component:
            // The deployment node selected defines which nodes appear in the Component panel
            if (!this.isComponentPanelPinned && this.schematicData) {
              await this.loadSchematicData(this.schematicData);
            }
            break;
        }
      },
    });

    componentSelection$.pipe(untilUnmounted).subscribe({
      next: (selections) => {
        // Edge case where the visibility changed, and every state was cleared
        // except for this.selection
        if (
          !selections.length &&
          !this.interactionManager?.selectionManager?.selection?.length
        ) {
          this.selection = null;
        }

        const ourSelection = selections.find(
          (sel) =>
            sel.parentDeploymentNodeId ===
            this.dataManager?.selectedDeploymentNodeId,
        );

        this.syncSelection(
          selections,
          ourSelection?.parentDeploymentNodeId ?? null,
        );
      },
    });

    this.renderer?.stage?.addChild(this.sceneManager.scene as PIXI.Container);

    if (this.schematicData) {
      this.loadSchematicData(this.schematicData).then(() =>
        this.renderer?.renderStage(),
      );
    } else {
      this.renderer?.renderStage();
    }
  },
  beforeUnmount(): void {
    if (this.container.element) {
      this.resizeObserver.unobserve(this.container.element);
    }
  },
  methods: {
    resizeCanvas(width: number, height: number): void {
      if (this.renderer && this.container.element) {
        this.renderer?.resize(width, height);
        this.renderer?.renderStage();
      }
    },

    handleKeyDown(e: KeyboardEvent): void {
      if (e.key === spacebarKey.long || e.key === spacebarKey.short) {
        e.preventDefault();
        if (this.component.isActive) {
          this.send(ViewerEventKind.ACTIVATE_PANNING);
        }
      }
    },

    handleKeyUp(e: KeyboardEvent): void {
      if (e.key === spacebarKey.long || e.key === spacebarKey.short) {
        e.preventDefault();
        this.send(ViewerEventKind.DEACTIVATE_PANNING);
      }
    },

    activateComponent(): void {
      this.component.isActive = true;
    },

    deactivateComponent(): void {
      this.component.isActive = false;
    },

    mouseEnter(): void {
      this.activateComponent();
    },
    mouseLeave(): void {
      this.deactivateComponent();
    },

    handleMouseWheel(e: WheelEvent): void {
      e.preventDefault();
      // implement zoom on alt/option key
      if (this.interactionManager) {
        this.send(ViewerEventKind.ACTIVATE_ZOOMING);
        this.send(ViewerEventKind.INITIATE_ZOOMING);
        this.send(ViewerEventKind.ZOOMING);
        this.interactionManager.zoomingManager.zoom(e);
        this.renderer?.renderStage();
        this.send(ViewerEventKind.DEACTIVATE_ZOOMING);
      }
    },

    syncSelection(
      selections: SelectedNode[],
      parentDeploymentNodeId: number | null,
    ) {
      for (const selection of selections) {
        const isOurSelection =
          selection.parentDeploymentNodeId === parentDeploymentNodeId;

        if (isOurSelection) {
          const nodes = this.sceneManager?.group?.nodes?.children;
          // Finds our panel's node
          let node = nodes
            ?.map((n) => n as OBJ.Node)
            ?.find((n) => n.id === selection.nodes[0]?.id);

          // Deployment node can appear in both panels, so we have to filter the selection of it when it happens in the other panel kind
          if (
            (this.schematicKind !== SchematicKind.Deployment &&
              parentDeploymentNodeId === null) ||
            (this.schematicKind === SchematicKind.Deployment &&
              parentDeploymentNodeId !== null)
          ) {
            node = undefined;
          }

          if (!node) {
            // If storing nodes from other panels we won't have our version, so let's store the original one
            // If we change to that panel we will just retrigger the selection, therefore we will actually find it
            node = selection.nodes[0];
          }

          this.selection = node ? [node] : null;
          this.interactionManager?.selectionManager?.select({
            parentDeploymentNodeId: selection.parentDeploymentNodeId,
            nodes: node ? [node] : [],
          });
          this.renderer?.renderStage();
        }
      }
    },

    async loadSchematicData(schematic: MODEL.Schematic): Promise<void> {
      const selectionManager = this.interactionManager?.selectionManager;
      if (
        this.schematicKind &&
        this.sceneManager &&
        this.dataManager &&
        selectionManager
      ) {
        // Deep cloning, very hackish, but bypassess all proxies
        const filteredSchematic: MODEL.Schematic = {
          nodes: schematic.nodes,
          connections: schematic.connections,
          lastUpdated: schematic.lastUpdated,
          checksum: schematic.checksum,
        };
        const parentDeploymentNodeId =
          this.schematicKind !== SchematicKind.Deployment
            ? this.dataManager.selectedDeploymentNodeId
            : null;

        // We want to ignore component data in deployment panel, and vice versa
        // The selected deployment node appears in both panels
        filteredSchematic.nodes = filteredSchematic.nodes.filter(
          (node) =>
            node.kind.kind === nodeKindFromSchematicKind(this.schematicKind) ||
            node.id === parentDeploymentNodeId,
        );
        // Find component nodes connected to selected deployment node
        const nodeIds = filteredSchematic.connections
          .filter((conn) => conn.destination.nodeId === parentDeploymentNodeId)
          .map((conn) => conn.source.nodeId);
        if (parentDeploymentNodeId) {
          nodeIds.push(parentDeploymentNodeId);
        }

        switch (this.schematicKind) {
          case SchematicKind.Deployment:
            break;
          case SchematicKind.Component:
            // Filters component nodes that are children of selected deployment node
            filteredSchematic.nodes = filteredSchematic.nodes.filter((node) =>
              nodeIds.includes(node.id),
            );
            break;
        }

        // We need to remove connections from nodes that don't appear in our panel
        filteredSchematic.connections = filteredSchematic.connections.filter(
          (conn) => {
            return (
              filteredSchematic.nodes.find(
                (node) => node.id === conn.destination.nodeId,
              ) &&
              filteredSchematic.nodes.find(
                (node) => node.id === conn.source.nodeId,
              )
            );
          },
        );

        this.sceneManager.loadSceneData(
          filteredSchematic,
          selectionManager as SelectionManager,
          this.schematicKind,
          parentDeploymentNodeId,
        );
        this.renderer?.renderStage();
      }
    },

    handleViewerEvent(e: VE.ViewerEvent | null): void {
      if (e && e.kind == VE.ViewerEventKind.NODE_ADD) {
        this.nodeAdd(e.data.node, e.data.schemaId);
      }
    },

    nodeAdd(node: MODEL.Node, schemaId: number): void {
      this.activateComponent();
      if (
        this.component.isActive &&
        this.interactionManager &&
        node.position.length > 0
      ) {
        // Fake nodes will always only have one position as they don't exist in the db yet
        const nodeObj = new OBJ.Node(node, {
          x: parseFloat(node.position[0].x as string),
          y: parseFloat(node.position[0].y as string),
        });
        this.send(ViewerEventKind.ACTIVATE_NODEADD);
        this.interactionManager.nodeAddManager.addNode(nodeObj, schemaId);
      }
    },
  },
});
</script>
