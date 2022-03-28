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
import { defineComponent, PropType } from "vue";
import _ from "lodash";
import * as Rx from "rxjs";
import * as PIXI from "pixi.js";
import * as OBJ from "./Viewer/obj";
import { refFrom } from "vuse-rx";

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
import { GlobalErrorService } from "@/service/global_error";
import { SchematicDataManager } from "./data";
import { deploymentSelection$, componentSelection$ } from "./state";

// import { Schematic } from "./model";
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
): Rx.ReplaySubject<Array<OBJ.Node> | null> => {
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
  },
  setup(props) {
    const { state, send, service } = useMachine(props.viewerState.machine);

    // Chooses which observable will define our selected node
    const selection = refFrom(
      Rx.combineLatest([deploymentSelection$, componentSelection$]).pipe(
        Rx.map(([deploymentSelection, componentSelection]) => {
          switch (props.schematicKind) {
            case SchematicKind.Deployment:
              return deploymentSelection;
            case SchematicKind.Component:
              return componentSelection;
          }
          return null;
        }),
      ),
      null,
    );

    return {
      state,
      send,
      service,
      selection,
      subscribers: [] as Array<Rx.Subscription>,
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
    editorContext(ctx) {
      if (this.dataManager && this.sceneManager && this.editorContext) {
        this.dataManager.editorContext$.next(ctx);
      }
    },
    async schematicKind(ctx) {
      if (this.dataManager && this.schematicData && this.interactionManager) {
        this.dataManager.schematicKind$.next(ctx);
        await this.loadSchematicData(this.schematicData);

        // We resend the last selection state to update ourselves
        console.debug("Schematic Kind changed: " + ctx);
        const nodes = await Rx.firstValueFrom(selectionObserver(ctx));
        selectionObserver(ctx).next(nodes);
      }
    },
    schematicData(schematic) {
      if (this.dataManager && this.schematicData) {
        this.dataManager.schematicData$.next(schematic);
      }
    },
  },
  unmounted(): void {
    for (const subscriber of this.subscribers) {
      subscriber.unsubscribe();
    }
    this.subscribers = [];
  },
  mounted(): void {
    this.canvas.element = this.$refs.canvas as HTMLCanvasElement;
    this.container.element = this.$refs.container as HTMLCanvasElement;

    document.addEventListener("keydown", this.handleKeyDown);
    document.addEventListener("keyup", this.handleKeyUp);

    if (this.viewerEvent$) {
      this.subscribers.push(
        this.viewerEvent$.subscribe({
          next: (v) => this.handleViewerEvent(v),
        }),
      );
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
    this.dataManager.editorContext$.next(this.editorContext);
    this.dataManager.schematicKind$.next(this.schematicKind);

    this.subscribers.push(
      dataManager.schematicData$.subscribe({
        next: async (d) => await this.loadSchematicData(d),
      }),
    );

    // Global events
    this.subscribers.push(
      schematicData$.subscribe({
        next: (d) => this.dataManager?.schematicData$?.next(d),
      }),
    );

    this.sceneManager = new SceneManager(this.renderer as Renderer);

    const interactionManager = new InteractionManager(
      this.sceneManager as SceneManager,
      dataManager,
      this.service,
      this.renderer as Renderer,
    );
    this.interactionManager = interactionManager;
    this.subscribers.push(
      this.sceneManager.subscribeToInteractionEvents(interactionManager),
    );

    const syncSelection = (
      selection: Array<OBJ.Node> | null,
      schematicKind: SchematicKind,
    ) => {
      const manager = this.interactionManager?.selectionManager;
      const nodes = this.sceneManager?.group?.nodes?.children;
      if (!selection && (manager?.selection ?? [])[0]) {
        // If the other panel deselected the node we have to update our selection state
        this.interactionManager?.selectionManager?.clearSelection();
        this.renderer?.renderStage();
      } else if (selection && !selection[0]) {
        // If there is as selection but it's an empty array, there is a bug somewhere
        const sel = JSON.stringify(selection);
        const message = `Selection is broken ${sel} in panel ${this.component.id}`;
        const [statusCode, code] = [500, 42];

        GlobalErrorService.set({ error: { statusCode, message, code } });
      } else if (selection) {
        const node = nodes
          ?.map((n) => n as OBJ.Node)
          ?.find((n) => n.id === selection[0].id);

        if (!node) {
          // This generally is a bug, but there are regular cases where it happen, like with vue's hot-reload
          // When we caught it happening the viewer was always completely empty, so we just check for that edge case
          if (!nodes || nodes.length === 0) {
            console.warn("Vue hot-reload broke current state, reseting it");
            selectionObserver(schematicKind).next(null);
            return;
          }

          // If there is as selection but the node related to it is not found, there is a bug somewhere
          const message = `Node ${selection[0].id} not found in panel ${this.component.id}`;
          const [statusCode, code] = [500, 42];
          const error = { statusCode, message, code };

          GlobalErrorService.set({ error });
        } else if (node.id !== (manager?.selection ?? [])[0]?.id) {
          // If the other panel selected the node we have to update our selection state
          this.interactionManager?.selectionManager?.select(node);
          this.renderer?.renderStage();
        }
      }
    };

    let lastDeploymentSelection: OBJ.Node | null = null;
    this.subscribers.push(
      deploymentSelection$.subscribe({
        next: async (selection) => {
          if (selection && lastDeploymentSelection === selection[0]) return;
          lastDeploymentSelection = selection ? selection[0] : null;

          switch (this.schematicKind) {
            case SchematicKind.Deployment:
              // We need to sync ourselves with the other panel if it's also Deployment
              syncSelection(selection, this.schematicKind);
              break;
            case SchematicKind.Component:
              // The deployment node selected defines which nodes appear in the Component panel
              if (this.schematicData) {
                await this.loadSchematicData(this.schematicData);
              }
              break;
          }
        },
      }),
    );

    let lastComponentSelection: OBJ.Node | null = null;
    this.subscribers.push(
      componentSelection$.subscribe({
        next: (selection) => {
          if (selection && lastComponentSelection === selection[0]) return;
          lastComponentSelection = selection ? selection[0] : null;

          switch (this.schematicKind) {
            case SchematicKind.Deployment:
              break;
            case SchematicKind.Component:
              // We need to sync ourselves with the other panel if it's also Component
              syncSelection(selection, this.schematicKind);
              break;
          }
        },
      }),
    );

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

    async loadSchematicData(schematic: MODEL.Schematic | null): Promise<void> {
      if (this.schematicKind && schematic && this.sceneManager) {
        // Deep cloning, very hackish, but bypassess all proxies
        const filteredSchematic: MODEL.Schematic = JSON.parse(
          JSON.stringify(schematic),
        );
        // We want to ignore component data in deployment panel, and vice versa
        filteredSchematic.nodes = filteredSchematic.nodes.filter(
          (node) =>
            node.kind.kind === nodeKindFromSchematicKind(this.schematicKind),
        );

        const deploymentNodes = await Rx.firstValueFrom(deploymentSelection$);
        // Find component nodes connected to selected deployment node
        const nodeIds = filteredSchematic.connections
          .filter(
            (conn) =>
              conn.destination.nodeId === (deploymentNodes ?? [])[0]?.id,
          )
          .map((conn) => conn.source.nodeId);

        const componentNodes = await Rx.firstValueFrom(componentSelection$);
        const selectionManager = this.interactionManager?.selectionManager;
        if (selectionManager) {
          switch (this.schematicKind) {
            case SchematicKind.Deployment:
              break;
            case SchematicKind.Component:
              // Filters component nodes that are children of selected deployment node
              filteredSchematic.nodes = filteredSchematic.nodes.filter((node) =>
                nodeIds.includes(node.id),
              );
              // If selected node is not in display anymore de-select it
              if (componentNodes && !nodeIds.includes(componentNodes[0]?.id)) {
                selectionManager.clearSelection(componentSelection$);
              }
              break;
          }

          this.sceneManager.loadSceneData(
            filteredSchematic,
            selectionManager as SelectionManager,
          );
        }
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
      if (this.component.isActive) {
        if (this.interactionManager) {
          const nodeObj = new OBJ.Node(node);
          this.send(ViewerEventKind.ACTIVATE_NODEADD);
          this.interactionManager.nodeAddManager.addNode(nodeObj, schemaId);
        }
      }
    },
  },
});
</script>
