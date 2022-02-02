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
import { refFrom } from "vuse-rx";

import { ViewerStateMachine, ViewerEventKind } from "./state";
import { useMachine } from "@xstate/vue";

import { EditorContext } from "@/api/sdf/dal/schematic";
import { SceneManager } from "./Viewer/scene";
import { InteractionManager } from "./Viewer/interaction";
import { Renderer } from "./Viewer/renderer";
import { SchematicDataManager } from "./data";

// import { Schematic } from "./model";
import * as MODEL from "./model";

import * as VE from "./event";

import * as s from "./state";
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
  },
  setup(props) {
    const { state, send, service } = useMachine(props.viewerState.machine);
    const selection = refFrom(s.selection$); //pipe and additional operations.
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
    schematicData(schematic) {
      if (this.sceneManager && this.schematicData) {
        this.dataManager?.schematicData$.next(schematic);
      }
    },
  },
  mounted(): void {
    this.canvas.element = this.$refs.canvas as HTMLCanvasElement;
    this.container.element = this.$refs.container as HTMLCanvasElement;

    document.addEventListener("keydown", this.handleKeyDown);
    document.addEventListener("keyup", this.handleKeyUp);

    if (this.viewerEvent$) {
      this.viewerEvent$.subscribe({
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

    this.dataManager.editorContext$.next(this.editorContext);

    dataManager.schematicData$.subscribe({
      next: (d) => this.loadSchematicData(d),
    });

    // Global events
    schematicData$.subscribe({
      next: (d) => this.dataManager?.schematicData$.next(d),
    });

    this.sceneManager = new SceneManager(this.renderer as Renderer);

    const interactionManager = new InteractionManager(
      this.sceneManager as SceneManager,
      dataManager,
      this.service,
      this.renderer as Renderer,
      this.editorContext,
    );
    this.interactionManager = interactionManager;

    this.sceneManager.subscribeToInteractionEvents(interactionManager);

    this.renderer.stage.sortableChildren = true;

    this.renderer.stage.addChild(this.sceneManager.scene as PIXI.Container);

    if (this.schematicData) {
      this.sceneManager.loadSceneData(this.schematicData);
    }

    this.renderer.render(this.renderer.stage);
  },
  beforeUnmount(): void {
    if (this.container.element) {
      this.resizeObserver.unobserve(this.container.element);
    }
  },
  methods: {
    resizeCanvas(width: number, height: number): void {
      if (this.renderer && this.container.element) {
        this.renderer.resize(width, height);
        this.renderer.render(this.renderer.stage);
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
        this.interactionManager.renderer.renderStage();
        this.send(ViewerEventKind.DEACTIVATE_ZOOMING);
      }
    },

    loadSchematicData(schematic: MODEL.Schematic | null): void {
      if (schematic && this.sceneManager) {
        this.sceneManager.loadSceneData(schematic);
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
          this.send(ViewerEventKind.ACTIVATE_NODEADD);
          this.interactionManager.nodeAddManager.addNode(node, schemaId);
        }
      }
    },
  },
});
</script>
