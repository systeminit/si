<template>
  <div :id="container.id" ref="container" class="w-full h-full">
    <div v-if="debug" class="flex flex-row">
      <div class="ml-2 font-medium text-yellow-200">{{ state.value }}</div>
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
import * as ST from "./state";
import { untilUnmounted } from "vuse-rx";
import { Interpreter } from "xstate";

import { ViewerStateMachine, ViewerEventKind } from "./state";
import { useMachine } from "@xstate/vue";
import { componentsMetadata$ } from "@/observable/component";
import { ComponentMetadata } from "@/service/component/get_components_metadata";

import { EditorContext, SchematicKind } from "@/api/sdf/dal/schematic";
import { SceneManager } from "./Viewer/scene";
import { InteractionManager } from "./Viewer/interaction";
import { Renderer } from "./Viewer/renderer";
import { SchematicDataManager } from "./data";
import { Schematic, SchematicNode, variantById } from "@/api/sdf/dal/schematic";
import { nodeSelection$, SelectedNode } from "./state";

import * as VE from "./event";

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
    element?: HTMLCanvasElement;
    isPanning: boolean;
    mousePosition?: { x: number; y: number } | null;
  };
  container: {
    id: string;
    element?: HTMLCanvasElement;
  };
  renderer?: Renderer;
  sceneManager?: SceneManager;
  spaceBarPressed: boolean;
  resizeObserver: ResizeObserver;
  interactionManager?: InteractionManager;
  debug: boolean;
}

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
      type: Object as PropType<Schematic>,
      required: true,
    },
    editorContext: {
      type: Object as PropType<EditorContext | null>,
      required: true,
    },
    schematicKind: {
      type: String as PropType<SchematicKind>,
      required: true,
    },
    deploymentNodeSelected: {
      type: Number,
      required: false,
      default: undefined,
    },
  },
  setup(props) {
    const { state, send, service } = useMachine(props.viewerState.machine);

    const dataManager = new SchematicDataManager();
    dataManager.editorContext$.next(props.editorContext);
    dataManager.schematicKind$.next(props.schematicKind);

    return {
      state,
      send,
      dataManager,
      service: service as Interpreter<unknown>,
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
        id,
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
      spaceBarPressed: false,
      resizeObserver: resizeObserver,
      interactionManager: undefined,
      debug: true,
    };
  },
  watch: {
    async deploymentNodeSelected(ctx) {
      this.dataManager.selectedDeploymentNodeId = ctx;
      await this.loadSchematicData(this.schematicData);
    },
    editorContext(ctx) {
      this.dataManager.editorContext$.next(ctx);
    },
    async schematicKind(ctx) {
      if (this.interactionManager) {
        this.dataManager.schematicKind$.next(ctx);
        await this.loadSchematicData(this.schematicData);

        // Fixes some inconsistencies where if you change panels while dragging the dragging would stick forever
        ST.deactivateDragging(
          this.interactionManager.stateService as Interpreter<unknown>,
        );
      }
    },
    async schematicData(schematic) {
      this.dataManager.schematicData$.next(schematic);
    },
  },
  mounted(): void {
    this.canvas.element = this.$refs.canvas as HTMLCanvasElement;
    this.container.element = this.$refs.container as HTMLCanvasElement;

    this.dataManager.selectedDeploymentNodeId = this.deploymentNodeSelected;
    this.dataManager.schematicData$
      .pipe(untilUnmounted)
      .subscribe(async (data) => {
        if (data) await this.loadSchematicData(data);
      });

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

    this.sceneManager = new SceneManager(this.renderer as Renderer);

    const interactionManager = new InteractionManager(
      this.sceneManager as SceneManager,
      this.dataManager,
      this.service,
      this.renderer as Renderer,
    );
    this.interactionManager = interactionManager;
    this.sceneManager.subscribeToInteractionEvents(interactionManager);

    nodeSelection$.pipe(untilUnmounted).subscribe((selections) => {
      this.syncSelection(selections);
    });

    this.renderer?.stage?.addChild(this.sceneManager.scene as PIXI.Container);

    componentsMetadata$.pipe(untilUnmounted).subscribe((metadatas) => {
      if (metadatas) this.updateMetadata(metadatas);
    });

    // Load local metadata
    Rx.firstValueFrom(componentsMetadata$).then((metadatas) => {
      if (metadatas) this.updateMetadata(metadatas);
    });

    this.loadSchematicData(this.schematicData).then(() => {
      this.renderer?.renderStage();

      // Note: Horrible hack. I have no idea why, but without this the first load doesn't show the qualification/resource sync icons
      setTimeout(async () => {
        const metadatas = await Rx.firstValueFrom(componentsMetadata$);
        if (metadatas) this.updateMetadata(metadatas);
      }, 100);
    });
  },
  beforeUnmount(): void {
    document.removeEventListener("keydown", this.handleKeyDown);
    document.removeEventListener("keyup", this.handleKeyUp);

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

    syncSelection(selections: SelectedNode[]) {
      const parentDeploymentNodeId = this.dataManager.selectedDeploymentNodeId;

      const nodes = this.sceneManager?.group?.nodes?.children as
        | OBJ.Node[]
        | undefined;

      const selected = selections.find(
        (s) => s.parentDeploymentNodeId === parentDeploymentNodeId,
      );
      for (const node of nodes ?? []) {
        if (selected?.nodeIds?.includes(node.id)) {
          node.select();
        } else {
          node.deselect();
        }
      }

      this.renderer?.renderStage();
    },

    async loadSchematicData(schematic: Schematic): Promise<void> {
      if (this.sceneManager) {
        const parentDeploymentNodeId =
          this.dataManager.selectedDeploymentNodeId;

        await this.sceneManager.loadSceneData(
          schematic,
          this.schematicKind,
          parentDeploymentNodeId,
        );

        this.syncSelection(await Rx.firstValueFrom(nodeSelection$));
      }

      const metadatas = await Rx.firstValueFrom(componentsMetadata$);
      if (metadatas) this.updateMetadata(metadatas);
    },

    handleViewerEvent(e: VE.ViewerEvent | null): void {
      if (e && e.kind == VE.ViewerEventKind.NODE_ADD) {
        this.nodeAdd(e.data.node, e.data.schemaId);
      }
    },

    updateMetadata(metadatas: ComponentMetadata[]) {
      if (!this.sceneManager?.group) return;

      for (const metadata of metadatas) {
        for (const n of this.sceneManager.group.nodes.children) {
          const node = n as OBJ.Node;
          if (metadata.componentId === node.nodeKind?.componentId) {
            node.setQualificationStatus(metadata.qualified);
            node.setResourceStatus(metadata.resourceHealth);
            break;
          }
        }
      }
      this.renderer?.renderStage();
    },

    async nodeAdd(node: SchematicNode, schemaId: number): Promise<void> {
      this.activateComponent();

      if (
        this.component.isActive &&
        this.interactionManager &&
        node.positions.length > 0
      ) {
        const schemaVariant = await variantById(node.schemaVariantId);

        // Fake nodes will always only have one position as they don't exist in the db yet
        const nodeObj = new OBJ.Node(
          node,
          schemaVariant,
          {
            x: node.positions[0].x,
            y: node.positions[0].y,
          },
          this.schematicKind,
        );
        this.send(ViewerEventKind.ACTIVATE_NODEADD);
        this.interactionManager.nodeAddManager.addNode(nodeObj, schemaId);
      }
    },
  },
});
</script>
