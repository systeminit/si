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
import { componentsMetadata$ } from "@/observable/component";
import { ComponentMetadata } from "@/service/component/get_components_metadata";

import { EditorContext, SchematicKind } from "@/api/sdf/dal/schematic";
import { SceneManager } from "./Viewer/scene";
import { SelectionManager } from "./Viewer/interaction/selection";
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
  dataManager?: SchematicDataManager;
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
      type: Object as PropType<Schematic | null>,
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
    deploymentNodePin: {
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
    async deploymentNodePin(ctx) {
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
      if (this.dataManager) {
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
        const data = await Rx.firstValueFrom(nodeSelection$);
        nodeSelection$.next(data);
      }
    },
    async schematicData(schematic) {
      if (this.dataManager) {
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
    this.dataManager.selectedDeploymentNodeId = this.deploymentNodePin;
    this.dataManager.editorContext$.next(this.editorContext);
    this.dataManager.schematicKind$.next(this.schematicKind);

    dataManager.schematicData$.pipe(untilUnmounted).subscribe({
      next: async (d) => {
        if (d) await this.loadSchematicData(d);
      },
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

    nodeSelection$.pipe(untilUnmounted).subscribe({
      next: async (selections) => {
        console.log(this.schematicKind);
        if (
          !this.isComponentPanelPinned &&
          this.dataManager &&
          this.schematicData
        ) {
          const deploymentSelection = selections.find(
            (sel) => sel.parentDeploymentNodeId === undefined,
          );
          const deploymentNodeId = deploymentSelection?.nodes?.find(
            (_) => true,
          )?.id;
          const isSame =
            this.dataManager.selectedDeploymentNodeId === deploymentNodeId;

          if (!isSame) {
            this.dataManager.selectedDeploymentNodeId = deploymentNodeId;

            switch (this.schematicKind) {
              case SchematicKind.Deployment:
                break;
              case SchematicKind.Component:
                // The deployment node selected defines which nodes appear in the Component panel
                await this.loadSchematicData(this.schematicData);
                break;
            }
          }
        }

        this.syncSelection(selections);
      },
    });

    this.renderer?.stage?.addChild(this.sceneManager.scene as PIXI.Container);

    // Initializes selectedDeploymentNodeId to the global state
    Rx.firstValueFrom(nodeSelection$).then((selections) => {
      nodeSelection$.next(selections);
    });

    componentsMetadata$.pipe(untilUnmounted).subscribe((metadatas) => {
      if (metadatas) this.updateMetadata(metadatas);
    });

    if (this.schematicData) {
      this.loadSchematicData(this.schematicData).then(() => {
        this.renderer?.renderStage();

        // Note: Horrible hack. I have no idea why, but without this the first load doesn't show the qualification/resource sync icons
        setTimeout(async () => {
          const metadatas = await Rx.firstValueFrom(componentsMetadata$);
          if (metadatas) this.updateMetadata(metadatas);
        }, 100);
      });
    } else {
      this.renderer?.renderStage();
    }
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
      const parentDeploymentNodeId =
        this.schematicKind !== SchematicKind.Deployment
          ? this.dataManager?.selectedDeploymentNodeId
          : undefined;

      let foundOurSelection = false;
      for (const selection of selections) {
        const isOurSelection =
          selection.parentDeploymentNodeId === parentDeploymentNodeId;

        const nodes = this.sceneManager?.group?.nodes?.children;

        // If storing nodes from other panels we won't have our version, so let's store the original one
        // If we change to that panel we will just retrigger the selection, therefore we will actually find it
        let node: OBJ.Node | null = selection.nodes[0] ?? null;

        if (isOurSelection) {
          foundOurSelection = true;

          node =
            nodes
              ?.map((n) => n as OBJ.Node)
              ?.find((n) => n.id === selection.nodes[0]?.id) ?? null;
        }

        if (!node) return;
        this.selection = node ? [node] : null;
        this.interactionManager?.selectionManager?.select({
          parentDeploymentNodeId: selection.parentDeploymentNodeId,
          nodes: node ? [node] : [],
        });
        this.renderer?.renderStage();
      }

      if (!foundOurSelection) {
        this.interactionManager?.selectionManager?.select({
          parentDeploymentNodeId,
          nodes: [],
        });
      }
    },

    async loadSchematicData(schematic: Schematic): Promise<void> {
      const selectionManager = this.interactionManager?.selectionManager;
      if (
        this.schematicKind &&
        this.sceneManager &&
        this.dataManager &&
        selectionManager
      ) {
        const parentDeploymentNodeId =
          this.schematicKind !== SchematicKind.Deployment
            ? this.dataManager?.selectedDeploymentNodeId
            : undefined;
        await this.sceneManager.loadSceneData(
          schematic,
          selectionManager as SelectionManager,
          this.schematicKind,
          parentDeploymentNodeId,
        );
        this.renderer?.renderStage();
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
        node.positions.length > 0 &&
        this.schematicKind
      ) {
        let schemaVariant;
        try {
          schemaVariant = await variantById(node.schemaVariantId);
        } catch {
          // Since we are using the `listComponentsIdentification` hack to obtain the schemas as the new backend route isn't
          // there yet, we only get the schema metadata of components already created.
          // So in schemas where no components have been created for, we can't really find it's metadata, but as the only
          // reason we need the metadata here is to find it's sockets. And since we haven't created anything yet, it can't
          // connect or be connected to anything, so we just mock a schema here with no sockets and wait for the visibility
          // update, after creation, to give us the propper schema.
          //
          // This logic should go away after the backend provides us the complete set of SchematicSchemaVariants
          schemaVariant = {
            id: -20,
            name: "v0",
            schemaName: "mistery",
            inputSockets: [],
            outputSockets: [],
          };
        }

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
